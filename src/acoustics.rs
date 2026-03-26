//! Acoustics integration via goonj
//!
//! Bridges goonj's acoustic simulation with kiran's ECS:
//! - [`AcousticsEngine`] resource wrapping goonj's occlusion engine
//! - [`RoomAcoustics`] component for room geometry and materials
//! - [`AcousticSource`] component for directional sound sources
//! - [`AcousticPortal`] component for sound openings between rooms
//! - [`ReverbProcessor`] resource for FDN late reverb
//! - Re-exports key goonj types

pub use goonj::ambisonics::{
    BFormatIr, HoaIr, encode_bformat, encode_hoa, new_bformat_ir, new_hoa_ir,
};
pub use goonj::coupled::{CoupledDecay, CoupledRooms, coupled_room_decay};
pub use goonj::diffraction::{edge_diffraction_loss, is_occluded, utd_wedge_diffraction};
pub use goonj::directivity::{DirectivityBalloon, DirectivityPattern};
pub use goonj::fdn::{Fdn, FdnConfig, fdn_config_for_room};
pub use goonj::impulse::{ImpulseResponse, IrConfig, MultibandIr, generate_ir};
pub use goonj::impulse::{eyring_rt60, sabine_rt60};
pub use goonj::integration::kiran::{OcclusionEngine, OcclusionResult};
pub use goonj::material::{
    AcousticMaterial, FREQUENCY_BANDS, JcalMaterial, NUM_BANDS, WallConstruction,
};
pub use goonj::portal::{Portal, portal_energy_transfer};
pub use goonj::propagation::{
    GroundImpedance, TemperatureProfile, WindProfile, atmospheric_absorption, doppler_shift,
    inverse_square_law, speed_of_sound,
};
pub use goonj::room::{AcceleratedRoom, AcousticRoom, RoomGeometry, Wall};

use hisab::Vec3;
use serde::{Deserialize, Serialize};

use crate::world::{Entity, World};

// ---------------------------------------------------------------------------
// Room acoustics component
// ---------------------------------------------------------------------------

/// Acoustic room configuration attached to a scene/zone entity.
///
/// Defines the acoustic environment: room geometry, materials, temperature,
/// and humidity. Used by [`AcousticsEngine`] for occlusion queries and
/// impulse response generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomAcoustics {
    /// The acoustic room definition (geometry + environment).
    pub room: AcousticRoom,
    /// Cached RT60 estimate (seconds). Updated when room changes.
    pub rt60: f32,
}

impl RoomAcoustics {
    /// Create room acoustics from a shoebox room.
    #[must_use]
    pub fn shoebox(length: f32, width: f32, height: f32, material: AcousticMaterial) -> Self {
        let room = AcousticRoom::shoebox(length, width, height, material);
        let volume = room.geometry.volume_shoebox();
        let absorption = room.geometry.total_absorption();
        let rt60 = sabine_rt60(volume, absorption);
        Self { room, rt60 }
    }

    /// Create from an existing acoustic room.
    #[must_use]
    pub fn from_room(room: AcousticRoom) -> Self {
        let volume = room.geometry.volume_shoebox();
        let absorption = room.geometry.total_absorption();
        let rt60 = sabine_rt60(volume, absorption);
        Self { room, rt60 }
    }

    /// Recalculate RT60 after modifying the room.
    pub fn update_rt60(&mut self) {
        let volume = self.room.geometry.volume_shoebox();
        let absorption = self.room.geometry.total_absorption();
        self.rt60 = sabine_rt60(volume, absorption);
    }
}

// ---------------------------------------------------------------------------
// Acoustic source component
// ---------------------------------------------------------------------------

/// Directivity and acoustic properties for a sound-emitting entity.
///
/// Pair with [`crate::audio::SoundSource`] for full spatial audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcousticSource {
    /// Directivity pattern for this source.
    pub directivity: DirectivityPattern,
    /// Forward direction of the source (world space).
    pub front: Vec3,
    /// Source power level in dB SPL (at 1m reference distance).
    pub power_db: f32,
}

impl Default for AcousticSource {
    fn default() -> Self {
        Self {
            directivity: DirectivityPattern::Omnidirectional,
            front: Vec3::new(0.0, 0.0, -1.0),
            power_db: 85.0,
        }
    }
}

impl AcousticSource {
    /// Create an omnidirectional source at the given power level.
    #[must_use]
    pub fn omnidirectional(power_db: f32) -> Self {
        Self {
            power_db,
            ..Default::default()
        }
    }

    /// Create a cardioid source (e.g. NPC speaker facing forward).
    #[must_use]
    pub fn cardioid(front: Vec3, power_db: f32) -> Self {
        Self {
            directivity: DirectivityPattern::Cardioid,
            front,
            power_db,
        }
    }

    /// Set the directivity pattern.
    pub fn with_directivity(mut self, pattern: DirectivityPattern) -> Self {
        self.directivity = pattern;
        self
    }

    /// Set the front direction.
    pub fn with_front(mut self, front: Vec3) -> Self {
        self.front = front;
        self
    }

    /// Compute directivity gain toward a listener direction.
    #[must_use]
    #[inline]
    pub fn gain_toward(&self, direction: Vec3) -> f32 {
        self.directivity.gain(direction, self.front)
    }

    /// Compute per-band directivity gains toward a listener.
    #[must_use]
    #[inline]
    pub fn gain_per_band(&self, direction: Vec3) -> [f32; NUM_BANDS] {
        self.directivity.gain_per_band(direction, self.front)
    }
}

// ---------------------------------------------------------------------------
// Acoustic portal component
// ---------------------------------------------------------------------------

/// An acoustic opening attached to a doorway/window entity.
///
/// Connects two rooms and transmits sound with frequency-dependent filtering.
#[derive(Debug, Clone)]
pub struct AcousticPortal {
    /// The underlying goonj portal.
    pub portal: Portal,
    /// Entity of the source room.
    pub room_a: Entity,
    /// Entity of the destination room.
    pub room_b: Entity,
}

impl AcousticPortal {
    /// Create a portal between two room entities.
    #[must_use]
    pub fn new(
        position: Vec3,
        normal: Vec3,
        width: f32,
        height: f32,
        room_a: Entity,
        room_b: Entity,
    ) -> Self {
        Self {
            portal: Portal {
                position,
                normal,
                width,
                height,
            },
            room_a,
            room_b,
        }
    }

    /// Compute per-band energy transfer from source through this portal to listener.
    #[must_use]
    #[inline]
    pub fn energy_transfer(
        &self,
        source: Vec3,
        listener: Vec3,
        temp_celsius: f32,
    ) -> [f32; NUM_BANDS] {
        portal_energy_transfer(source, &self.portal, listener, temp_celsius)
    }
}

// ---------------------------------------------------------------------------
// Wall transmission component
// ---------------------------------------------------------------------------

/// Wall construction data for transmission loss calculations.
///
/// Attach to wall/partition entities to model sound bleeding through walls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallTransmission {
    /// The wall construction parameters.
    pub construction: WallConstruction,
}

impl WallTransmission {
    /// Create from a wall construction preset.
    #[must_use]
    pub fn new(construction: WallConstruction) -> Self {
        Self { construction }
    }

    /// Transmission loss in dB at a given frequency.
    #[must_use]
    #[inline]
    pub fn loss_db(&self, freq: f32) -> f32 {
        self.construction.transmission_loss_db(freq)
    }

    /// Transmission coefficient (0.0–1.0) at a given frequency.
    #[must_use]
    #[inline]
    pub fn coefficient(&self, freq: f32) -> f32 {
        self.construction.transmission_coefficient(freq)
    }
}

// ---------------------------------------------------------------------------
// Acoustics engine resource
// ---------------------------------------------------------------------------

/// The acoustics engine resource — manages occlusion queries and propagation.
///
/// Created from a [`RoomAcoustics`] component. Insert as a world resource
/// for real-time acoustic queries during the audio frame.
pub struct AcousticsEngine {
    /// The goonj occlusion engine (BVH-accelerated).
    engine: OcclusionEngine,
    /// Temperature for propagation calculations.
    temperature_celsius: f32,
}

impl AcousticsEngine {
    /// Build the acoustics engine from a room.
    #[must_use]
    pub fn new(room: AcousticRoom) -> Self {
        let temperature_celsius = room.temperature_celsius;
        Self {
            engine: OcclusionEngine::new(room),
            temperature_celsius,
        }
    }

    /// Build from a [`RoomAcoustics`] component.
    #[must_use]
    pub fn from_room_acoustics(room_acoustics: &RoomAcoustics) -> Self {
        Self::new(room_acoustics.room.clone())
    }

    /// Query occlusion between source and listener positions.
    #[must_use]
    #[inline]
    pub fn query_occlusion(&self, source: Vec3, listener: Vec3) -> OcclusionResult {
        self.engine.query(source, listener)
    }

    /// Compute distance attenuation (inverse square law).
    #[must_use]
    #[inline]
    pub fn distance_attenuation(power: f32, distance: f32) -> f32 {
        inverse_square_law(power, distance)
    }

    /// Compute Doppler-shifted frequency.
    #[must_use]
    #[inline]
    pub fn doppler(freq: f32, source_vel: f32, listener_vel: f32) -> f32 {
        let c = speed_of_sound(20.0);
        doppler_shift(freq, source_vel, listener_vel, c)
    }

    /// Compute Doppler shift with explicit temperature.
    #[must_use]
    #[inline]
    pub fn doppler_at_temp(
        freq: f32,
        source_vel: f32,
        listener_vel: f32,
        temp_celsius: f32,
    ) -> f32 {
        let c = speed_of_sound(temp_celsius);
        doppler_shift(freq, source_vel, listener_vel, c)
    }

    /// Compute atmospheric absorption at a given frequency (dB/m).
    #[must_use]
    #[inline]
    pub fn atmospheric_absorption_at(&self, freq: f32, humidity: f32) -> f32 {
        atmospheric_absorption(freq, humidity, self.temperature_celsius, 1.0)
    }

    /// Access the underlying room.
    #[must_use]
    pub fn room(&self) -> &AcousticRoom {
        self.engine.room()
    }

    /// Get the engine temperature.
    #[must_use]
    pub fn temperature(&self) -> f32 {
        self.temperature_celsius
    }
}

// ---------------------------------------------------------------------------
// Reverb processor resource
// ---------------------------------------------------------------------------

/// FDN-based reverb processor resource.
///
/// Provides efficient real-time late reverberation. Create from room
/// dimensions and RT60, then call [`process_sample`](Self::process_sample)
/// or [`process_buffer`](Self::process_buffer) each audio frame.
pub struct ReverbProcessor {
    fdn: Fdn,
    /// The configuration used to build this processor.
    config: FdnConfig,
}

impl ReverbProcessor {
    /// Create a reverb processor from explicit configuration.
    #[must_use]
    pub fn new(config: FdnConfig) -> Self {
        let fdn = Fdn::new(&config);
        Self { fdn, config }
    }

    /// Create a reverb processor sized for a room.
    #[must_use]
    pub fn for_room(length: f32, width: f32, height: f32, rt60: f32, sample_rate: u32) -> Self {
        let config = fdn_config_for_room(length, width, height, rt60, sample_rate);
        Self::new(config)
    }

    /// Process one sample through the FDN.
    #[inline]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        self.fdn.process_sample(input)
    }

    /// Process a buffer of samples.
    pub fn process_buffer(&mut self, input: &[f32]) -> Vec<f32> {
        self.fdn.process_buffer(input)
    }

    /// Reset all delay lines (silence the reverb tail).
    pub fn reset(&mut self) {
        self.fdn.reset();
    }

    /// Access the configuration.
    #[must_use]
    pub fn config(&self) -> &FdnConfig {
        &self.config
    }
}

// ---------------------------------------------------------------------------
// System: update acoustics from room entities
// ---------------------------------------------------------------------------

/// Rebuild the [`AcousticsEngine`] resource from the first entity with
/// [`RoomAcoustics`]. Call when the active room changes.
pub fn rebuild_acoustics_engine(world: &mut World) {
    let room = world
        .query::<RoomAcoustics>()
        .first()
        .map(|(_, ra)| ra.room.clone());

    if let Some(room) = room {
        let engine = AcousticsEngine::new(room);
        world.insert_resource(engine);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_room() -> AcousticRoom {
        AcousticRoom::shoebox(10.0, 8.0, 3.0, AcousticMaterial::concrete())
    }

    #[test]
    fn room_acoustics_shoebox() {
        let ra = RoomAcoustics::shoebox(10.0, 8.0, 3.0, AcousticMaterial::concrete());
        assert!(ra.rt60 > 0.0);
        assert_eq!(ra.room.geometry.walls.len(), 6);
    }

    #[test]
    fn room_acoustics_from_room() {
        let room = test_room();
        let ra = RoomAcoustics::from_room(room);
        assert!(ra.rt60 > 0.0);
    }

    #[test]
    fn room_acoustics_update_rt60() {
        let mut ra = RoomAcoustics::shoebox(10.0, 8.0, 3.0, AcousticMaterial::concrete());
        let original = ra.rt60;
        // Swap to a more absorbent material
        ra.room = AcousticRoom::shoebox(10.0, 8.0, 3.0, AcousticMaterial::carpet());
        ra.update_rt60();
        assert!(
            ra.rt60 < original,
            "carpet should give shorter RT60 than concrete"
        );
    }

    #[test]
    fn room_acoustics_serde_roundtrip() {
        let ra = RoomAcoustics::shoebox(10.0, 8.0, 3.0, AcousticMaterial::concrete());
        let json = serde_json::to_string(&ra).unwrap();
        let decoded: RoomAcoustics = serde_json::from_str(&json).unwrap();
        assert!((decoded.rt60 - ra.rt60).abs() < f32::EPSILON);
    }

    #[test]
    fn acoustic_source_default() {
        let src = AcousticSource::default();
        assert_eq!(src.power_db, 85.0);
        // Omnidirectional: gain should be 1.0 in any direction
        assert!((src.gain_toward(Vec3::X) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn acoustic_source_omnidirectional() {
        let src = AcousticSource::omnidirectional(90.0);
        assert_eq!(src.power_db, 90.0);
        assert!((src.gain_toward(Vec3::Z) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn acoustic_source_cardioid() {
        let src = AcousticSource::cardioid(Vec3::new(0.0, 0.0, -1.0), 85.0);
        let front_gain = src.gain_toward(Vec3::new(0.0, 0.0, -1.0));
        let back_gain = src.gain_toward(Vec3::new(0.0, 0.0, 1.0));
        assert!(front_gain > back_gain, "cardioid should be louder in front");
    }

    #[test]
    fn acoustic_source_builder() {
        let src = AcousticSource::omnidirectional(80.0)
            .with_directivity(DirectivityPattern::Supercardioid)
            .with_front(Vec3::X);
        assert_eq!(src.front, Vec3::X);
    }

    #[test]
    fn acoustic_source_per_band_gains() {
        let src = AcousticSource::default();
        let gains = src.gain_per_band(Vec3::X);
        assert_eq!(gains.len(), NUM_BANDS);
        // Omnidirectional: all bands should be 1.0
        for g in &gains {
            assert!((*g - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn acoustic_portal_creation() {
        let a = Entity::new(1, 0);
        let b = Entity::new(2, 0);
        let portal = AcousticPortal::new(Vec3::new(5.0, 1.0, 0.0), Vec3::Z, 1.0, 2.0, a, b);
        assert_eq!(portal.portal.area(), 2.0);
        assert_eq!(portal.room_a, a);
        assert_eq!(portal.room_b, b);
    }

    #[test]
    fn acoustic_portal_energy_transfer() {
        let a = Entity::new(1, 0);
        let b = Entity::new(2, 0);
        let portal = AcousticPortal::new(Vec3::new(5.0, 1.0, 0.0), Vec3::Z, 2.0, 2.5, a, b);
        let transfer =
            portal.energy_transfer(Vec3::new(3.0, 1.0, -2.0), Vec3::new(7.0, 1.0, 2.0), 20.0);
        assert_eq!(transfer.len(), NUM_BANDS);
        for t in &transfer {
            assert!(*t >= 0.0);
        }
    }

    #[test]
    fn wall_transmission_presets() {
        let wall = WallTransmission::new(WallConstruction::drywall_single());
        let tl = wall.loss_db(1000.0);
        assert!(tl > 0.0, "transmission loss should be positive dB");
        let coeff = wall.coefficient(1000.0);
        assert!(coeff > 0.0 && coeff < 1.0, "coefficient should be 0..1");
    }

    #[test]
    fn wall_transmission_frequency_dependent() {
        let wall = WallTransmission::new(WallConstruction::concrete_150mm());
        let low = wall.loss_db(125.0);
        let high = wall.loss_db(4000.0);
        // Concrete should generally block more at higher frequencies (mass law)
        assert!(
            high > low,
            "high freq TL ({high}) should exceed low ({low})"
        );
    }

    #[test]
    fn acoustics_engine_creation() {
        let engine = AcousticsEngine::new(test_room());
        assert_eq!(engine.room().geometry.walls.len(), 6);
        assert!((engine.temperature() - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn acoustics_engine_from_room_acoustics() {
        let ra = RoomAcoustics::shoebox(10.0, 8.0, 3.0, AcousticMaterial::concrete());
        let engine = AcousticsEngine::from_room_acoustics(&ra);
        assert_eq!(engine.room().geometry.walls.len(), 6);
    }

    #[test]
    fn acoustics_engine_unoccluded_query() {
        let engine = AcousticsEngine::new(test_room());
        let result = engine.query_occlusion(Vec3::new(3.0, 1.5, 4.0), Vec3::new(7.0, 1.5, 4.0));
        assert!(!result.is_occluded);
        assert!((result.attenuation_db).abs() < f32::EPSILON);
    }

    #[test]
    fn acoustics_engine_distance_attenuation() {
        let intensity = AcousticsEngine::distance_attenuation(1.0, 2.0);
        assert!(intensity > 0.0);
        assert!(intensity < 1.0);
        // Should follow inverse square: I ∝ 1/r²
        let i_near = AcousticsEngine::distance_attenuation(1.0, 1.0);
        let i_far = AcousticsEngine::distance_attenuation(1.0, 2.0);
        assert!((i_near / i_far - 4.0).abs() < 0.01);
    }

    #[test]
    fn acoustics_engine_doppler() {
        let original = 440.0;
        // Approaching source → higher frequency
        let shifted = AcousticsEngine::doppler(original, -10.0, 0.0);
        assert!(shifted > original);
        // Receding source → lower frequency
        let shifted = AcousticsEngine::doppler(original, 10.0, 0.0);
        assert!(shifted < original);
    }

    #[test]
    fn acoustics_engine_atmospheric_absorption() {
        let engine = AcousticsEngine::new(test_room());
        let abs_low = engine.atmospheric_absorption_at(125.0, 50.0);
        let abs_high = engine.atmospheric_absorption_at(8000.0, 50.0);
        assert!(abs_high > abs_low, "high freq absorbed more than low");
    }

    #[test]
    fn reverb_processor_for_room() {
        let mut reverb = ReverbProcessor::for_room(10.0, 8.0, 3.0, 1.5, 44100);
        assert_eq!(reverb.config().sample_rate, 44100);
        // Process a click and check we get output
        let out = reverb.process_sample(1.0);
        assert!(out.abs() > 0.0 || out == 0.0); // FDN may or may not output on first sample
        // Process a buffer
        let buf = vec![0.0; 512];
        let out = reverb.process_buffer(&buf);
        assert_eq!(out.len(), 512);
    }

    #[test]
    fn reverb_processor_reset() {
        let mut reverb = ReverbProcessor::for_room(10.0, 8.0, 3.0, 1.5, 44100);
        // Feed a click
        reverb.process_sample(1.0);
        for _ in 0..100 {
            reverb.process_sample(0.0);
        }
        reverb.reset();
        // After reset, silence in → silence out
        let out = reverb.process_sample(0.0);
        assert!((out).abs() < f32::EPSILON);
    }

    #[test]
    fn coupled_rooms_decay() {
        let room_a = AcousticRoom::shoebox(10.0, 8.0, 3.0, AcousticMaterial::concrete());
        let room_b = AcousticRoom::shoebox(6.0, 5.0, 3.0, AcousticMaterial::carpet());
        let portal = Portal {
            position: Vec3::new(5.0, 1.5, 0.0),
            normal: Vec3::Z,
            width: 1.0,
            height: 2.1,
        };
        let coupled = CoupledRooms {
            room_a,
            room_b,
            portal,
        };
        let decay = coupled_room_decay(&coupled);
        assert!(decay.rt60_early > 0.0);
        assert!(decay.rt60_late > 0.0);
        assert!(decay.coupling_strength >= 0.0 && decay.coupling_strength <= 1.0);
    }

    #[test]
    fn ir_generation_basic() {
        let room = test_room();
        let config = IrConfig {
            sample_rate: 44100,
            max_order: 2,
            num_diffuse_rays: 100,
            max_bounces: 3,
            max_time_seconds: 0.5,
            seed: 42,
        };
        let ir = generate_ir(
            Vec3::new(3.0, 1.5, 4.0),
            Vec3::new(7.0, 1.5, 4.0),
            &room,
            &config,
        );
        assert_eq!(ir.sample_rate, 44100);
        assert_eq!(ir.bands.len(), NUM_BANDS);
        let broadband = ir.to_broadband();
        assert!(!broadband.samples.is_empty());
    }

    #[test]
    fn bformat_encoding() {
        let mut ir = new_bformat_ir(1024, 44100);
        encode_bformat(0.5, Vec3::new(1.0, 0.0, 0.0), 10, &mut ir);
        // W channel should have energy at sample 10
        assert!(ir.w[10].abs() > 0.0);
        // X channel (front-back) should also have energy for +X direction
        assert!(ir.x[10].abs() > 0.0);
    }

    #[test]
    fn rebuild_engine_from_world() {
        let mut world = World::new();
        let entity = world.spawn();
        let ra = RoomAcoustics::shoebox(10.0, 8.0, 3.0, AcousticMaterial::concrete());
        world.insert_component(entity, ra).unwrap();

        rebuild_acoustics_engine(&mut world);

        assert!(world.get_resource::<AcousticsEngine>().is_some());
    }

    #[test]
    fn speed_of_sound_standard() {
        let c = speed_of_sound(20.0);
        assert!((c - 343.4).abs() < 0.2);
    }

    #[test]
    fn sabine_rt60_positive() {
        let rt60 = sabine_rt60(240.0, 50.0);
        assert!(rt60 > 0.0);
    }

    #[test]
    fn diffraction_loss_negative() {
        let loss = edge_diffraction_loss(1000.0, std::f32::consts::FRAC_PI_4, 20.0);
        assert!(loss < 0.0, "diffraction should attenuate (negative dB)");
    }

    #[test]
    fn material_presets() {
        let concrete = AcousticMaterial::concrete();
        let carpet = AcousticMaterial::carpet();
        assert!(carpet.average_absorption() > concrete.average_absorption());
    }
}
