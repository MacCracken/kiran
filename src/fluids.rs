//! Fluid dynamics integration via pravash
//!
//! Bridges pravash's fluid simulation with kiran's ECS:
//! - [`FluidEmitter`] component for entities that spawn fluid particles
//! - [`FluidBody`] component for entities affected by fluid forces
//! - Re-exports key pravash types

#[cfg(feature = "fluids")]
pub use pravash::shallow::ShallowWater;
#[cfg(feature = "fluids")]
pub use pravash::sph::SphSolver;
pub use pravash::{FluidConfig, FluidMaterial, FluidParticle};

use serde::{Deserialize, Serialize};

use crate::world::World;

// ---------------------------------------------------------------------------
// ECS components
// ---------------------------------------------------------------------------

/// A fluid emitter component — spawns fluid particles from this entity's position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidEmitter {
    /// Material of the emitted fluid.
    pub material: FluidMaterial,
    /// Particles per second.
    pub rate: f32,
    /// Initial velocity of emitted particles.
    pub velocity: [f32; 3],
    /// Spread angle in radians (0 = focused jet, PI = hemisphere).
    pub spread: f32,
    /// Maximum particles this emitter can have alive.
    pub max_particles: usize,
    /// Whether the emitter is active.
    pub active: bool,
    /// Accumulator for fractional particle emission.
    #[serde(skip)]
    pub accumulator: f32,
}

impl FluidEmitter {
    pub fn new(material: FluidMaterial) -> Self {
        Self {
            material,
            rate: 100.0,
            velocity: [0.0, -1.0, 0.0],
            spread: 0.1,
            max_particles: 1000,
            active: true,
            accumulator: 0.0,
        }
    }

    pub fn with_rate(mut self, rate: f32) -> Self {
        self.rate = rate;
        self
    }

    pub fn with_velocity(mut self, velocity: [f32; 3]) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    pub fn with_max_particles(mut self, max: usize) -> Self {
        self.max_particles = max;
        self
    }

    /// Calculate how many particles to emit this frame.
    #[must_use]
    pub fn particles_to_emit(&mut self, dt: f32) -> u32 {
        if !self.active {
            return 0;
        }
        self.accumulator += self.rate * dt;
        let count = self.accumulator as u32;
        self.accumulator -= count as f32;
        count
    }
}

/// Marks an entity as affected by fluid forces (buoyancy, drag).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidBody {
    /// Volume of the body in m³ (for buoyancy calculation).
    pub volume: f32,
    /// Drag coefficient (0 = no drag, 1 = high drag).
    pub drag_coefficient: f32,
    /// Whether this body is currently submerged.
    #[serde(skip)]
    pub submerged: bool,
    /// Current buoyancy force applied to this body.
    #[serde(skip)]
    pub buoyancy_force: [f32; 3],
}

impl FluidBody {
    pub fn new(volume: f32) -> Self {
        Self {
            volume,
            drag_coefficient: 0.5,
            submerged: false,
            buoyancy_force: [0.0; 3],
        }
    }

    pub fn with_drag(mut self, drag: f32) -> Self {
        self.drag_coefficient = drag;
        self
    }

    /// Calculate buoyancy force (Archimedes' principle).
    /// Returns force vector [fx, fy, fz].
    #[must_use]
    pub fn compute_buoyancy(
        &self,
        fluid_density: f64,
        gravity: [f64; 3],
        submersion_fraction: f32,
    ) -> [f32; 3] {
        let displaced_volume = self.volume * submersion_fraction;
        let buoyancy_mag = fluid_density as f32 * displaced_volume;
        [
            -gravity[0] as f32 * buoyancy_mag,
            -gravity[1] as f32 * buoyancy_mag,
            -gravity[2] as f32 * buoyancy_mag,
        ]
    }
}

/// Fluid simulation resource — wraps pravash SphSolver + particles for the world.
pub struct FluidSimulation {
    /// SPH solver (spatial hash, neighbor search).
    pub solver: SphSolver,
    /// Simulation config.
    pub config: FluidConfig,
    /// Particle storage (owned by the simulation).
    pub particles: Vec<FluidParticle>,
    /// Fluid viscosity.
    pub viscosity: f64,
    /// Whether the simulation is running.
    pub active: bool,
}

impl FluidSimulation {
    /// Create a new fluid simulation.
    pub fn new(config: FluidConfig, material: FluidMaterial) -> Self {
        Self {
            solver: SphSolver::new(),
            config,
            particles: Vec::new(),
            viscosity: material.viscosity,
            active: true,
        }
    }

    /// Create a water simulation with default config.
    pub fn water_2d() -> Self {
        Self::new(FluidConfig::water_2d(), FluidMaterial::WATER)
    }

    /// Step the simulation forward.
    pub fn step(&mut self) {
        if !self.active || self.particles.is_empty() {
            return;
        }
        let _ = self
            .solver
            .step(&mut self.particles, &self.config, self.viscosity);
    }

    /// Number of particles in the simulation.
    #[must_use]
    #[inline]
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Add a particle to the simulation.
    pub fn add_particle(&mut self, particle: FluidParticle) {
        self.particles.push(particle);
    }

    /// Clear all particles.
    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

/// Step the fluid simulation resource each frame.
pub fn step_fluid_simulation(world: &mut World) {
    if let Some(sim) = world.get_resource_mut::<FluidSimulation>() {
        sim.step();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fluid_emitter_new() {
        let emitter = FluidEmitter::new(FluidMaterial::WATER);
        assert!(emitter.active);
        assert_eq!(emitter.rate, 100.0);
        assert_eq!(emitter.max_particles, 1000);
    }

    #[test]
    fn fluid_emitter_builders() {
        let emitter = FluidEmitter::new(FluidMaterial::OIL)
            .with_rate(50.0)
            .with_velocity([1.0, 0.0, 0.0])
            .with_spread(0.5)
            .with_max_particles(500);
        assert_eq!(emitter.rate, 50.0);
        assert_eq!(emitter.velocity, [1.0, 0.0, 0.0]);
        assert_eq!(emitter.max_particles, 500);
    }

    #[test]
    fn fluid_emitter_particles_to_emit() {
        let mut emitter = FluidEmitter::new(FluidMaterial::WATER);
        emitter.rate = 60.0; // 60 particles/sec
        let count = emitter.particles_to_emit(1.0 / 60.0); // one frame at 60fps
        assert_eq!(count, 1);
    }

    #[test]
    fn fluid_emitter_inactive() {
        let mut emitter = FluidEmitter::new(FluidMaterial::WATER);
        emitter.active = false;
        assert_eq!(emitter.particles_to_emit(1.0), 0);
    }

    #[test]
    fn fluid_emitter_accumulates() {
        let mut emitter = FluidEmitter::new(FluidMaterial::WATER);
        emitter.rate = 30.0;
        // At 60fps, 0.5 particles per frame — first frame emits 0, second emits 1
        let c1 = emitter.particles_to_emit(1.0 / 60.0);
        let c2 = emitter.particles_to_emit(1.0 / 60.0);
        assert_eq!(c1 + c2, 1);
    }

    #[test]
    fn fluid_body_new() {
        let body = FluidBody::new(0.5);
        assert_eq!(body.volume, 0.5);
        assert!(!body.submerged);
    }

    #[test]
    fn fluid_body_buoyancy() {
        let body = FluidBody::new(1.0); // 1 m³
        let force = body.compute_buoyancy(1000.0, [0.0, -9.81, 0.0], 1.0);
        // Buoyancy = -gravity * density * volume = 9810 N upward
        assert!((force[1] - 9810.0).abs() < 1.0);
    }

    #[test]
    fn fluid_body_partial_submersion() {
        let body = FluidBody::new(1.0);
        let full = body.compute_buoyancy(1000.0, [0.0, -9.81, 0.0], 1.0);
        let half = body.compute_buoyancy(1000.0, [0.0, -9.81, 0.0], 0.5);
        assert!((half[1] - full[1] * 0.5).abs() < 1.0);
    }

    #[test]
    fn fluid_simulation_new() {
        let sim = FluidSimulation::water_2d();
        assert!(sim.active);
        assert_eq!(sim.particle_count(), 0);
    }

    #[test]
    fn fluid_simulation_add_particle() {
        let mut sim = FluidSimulation::water_2d();
        sim.add_particle(FluidParticle::new_2d(0.5, 0.5, 1.0));
        assert_eq!(sim.particle_count(), 1);
    }

    #[test]
    fn fluid_simulation_step() {
        let mut sim = FluidSimulation::water_2d();
        sim.add_particle(FluidParticle::new_2d(0.5, 0.8, 1.0));
        sim.step();
        // Particle should have moved (gravity)
        let p = &sim.particles[0];
        assert!(p.position[1] < 0.8); // fell due to gravity
    }

    #[test]
    fn fluid_simulation_inactive() {
        let mut sim = FluidSimulation::water_2d();
        sim.add_particle(FluidParticle::new_2d(0.5, 0.5, 1.0));
        sim.active = false;
        let pos_before = sim.particles[0].position;
        sim.step();
        assert_eq!(sim.particles[0].position, pos_before);
    }

    #[test]
    fn fluid_simulation_clear() {
        let mut sim = FluidSimulation::water_2d();
        sim.add_particle(FluidParticle::new_2d(0.5, 0.5, 1.0));
        sim.clear();
        assert_eq!(sim.particle_count(), 0);
    }

    #[test]
    fn fluid_emitter_as_component() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(e, FluidEmitter::new(FluidMaterial::WATER))
            .unwrap();
        assert!(world.has_component::<FluidEmitter>(e));
    }

    #[test]
    fn fluid_body_as_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, FluidBody::new(0.1)).unwrap();
        assert!(world.has_component::<FluidBody>(e));
    }

    #[test]
    fn fluid_simulation_as_resource() {
        let mut world = World::new();
        world.insert_resource(FluidSimulation::water_2d());
        let sim = world.get_resource::<FluidSimulation>().unwrap();
        assert_eq!(sim.particle_count(), 0);
    }

    #[test]
    fn step_fluid_system() {
        let mut world = World::new();
        let mut sim = FluidSimulation::water_2d();
        sim.add_particle(FluidParticle::new_2d(0.5, 0.8, 1.0));
        world.insert_resource(sim);

        step_fluid_simulation(&mut world);

        let sim = world.get_resource::<FluidSimulation>().unwrap();
        assert!(sim.particles[0].position[1] < 0.8);
    }

    #[test]
    fn fluid_material_presets() {
        let water = FluidMaterial::WATER;
        let air = FluidMaterial::AIR;
        let honey = FluidMaterial::HONEY;
        let lava = FluidMaterial::LAVA;
        assert!(water.density > air.density);
        assert!(honey.viscosity > water.viscosity);
        assert!(lava.density > water.density);
    }

    #[test]
    fn fluid_emitter_serde() {
        let emitter = FluidEmitter::new(FluidMaterial::WATER).with_rate(200.0);
        let json = serde_json::to_string(&emitter).unwrap();
        let decoded: FluidEmitter = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.rate, 200.0);
    }

    #[test]
    fn fluid_body_serde() {
        let body = FluidBody::new(2.5).with_drag(0.8);
        let json = serde_json::to_string(&body).unwrap();
        let decoded: FluidBody = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.volume, 2.5);
        assert_eq!(decoded.drag_coefficient, 0.8);
    }
}
