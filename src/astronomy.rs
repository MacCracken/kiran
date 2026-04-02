//! Astronomy and weather via falak, jyotish, tara, brahmanda, and badal
//!
//! Bridges the AGNOS astronomy stack with kiran's ECS:
//! - **falak** — Orbital mechanics (Kepler, transfers, perturbations, n-body)
//! - **jyotish** — Astronomical computation (planetary positions, eclipses, calendars)
//! - **tara** — Stellar astrophysics (classification, evolution, nucleosynthesis)
//! - **brahmanda** — Cosmology (expansion, cosmic web, dark matter halos)
//! - **badal** — Weather and atmosphere (clouds, precipitation, storms, stability)
//!
//! Core types:
//! - [`CelestialBody`] component for orbital entities
//! - [`WeatherZone`] component for atmospheric simulation

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// falak — orbital mechanics
// ---------------------------------------------------------------------------

/// Ephemeris computation.
pub use falak::ephemeris;
/// Reference frames and coordinate transforms.
pub use falak::frame;
/// Kepler's equation solvers.
pub use falak::kepler;
/// Orbital maneuvers (delta-v, burns).
pub use falak::maneuver;
/// N-body gravitational simulation.
pub use falak::nbody;
/// Orbital elements and Keplerian motion.
pub use falak::orbit;
/// Gravitational perturbations (J2, third-body).
pub use falak::perturbation;
/// Orbit propagation.
pub use falak::propagate;
/// Orbital transfers (Hohmann, bi-elliptic).
pub use falak::transfer;

pub use falak::FalakError;

// ---------------------------------------------------------------------------
// jyotish — astronomical computation
// ---------------------------------------------------------------------------

/// Calendar systems and Julian date conversion.
pub use jyotish::calendar;
/// Coordinate transforms (equatorial, ecliptic, horizontal).
pub use jyotish::coords;
/// Eclipse prediction (solar, lunar).
pub use jyotish::eclipse;
/// Lunar position and phases.
pub use jyotish::moon;
/// Nutation and precession corrections.
pub use jyotish::nutation;
/// Planetary positions (VSOP87, ELP2000).
pub use jyotish::planet;
/// Rise, set, and transit times.
pub use jyotish::riseset;
/// Star catalog and positions.
pub use jyotish::star as jyotish_star;
/// Solar position and phenomena.
pub use jyotish::sun;

pub use jyotish::JyotishError;

// ---------------------------------------------------------------------------
// tara — stellar astrophysics
// ---------------------------------------------------------------------------

/// Stellar atmospheres (opacity, limb darkening).
pub use tara::atmosphere as stellar_atmosphere;
/// Spectral classification (O, B, A, F, G, K, M).
pub use tara::classification;
/// Stellar evolution (HR diagram, tracks).
pub use tara::evolution;
/// Luminosity and magnitude calculations.
pub use tara::luminosity;
/// Nucleosynthesis (pp chain, CNO, s/r-process).
pub use tara::nucleosynthesis;
/// Stellar spectra and line profiles.
pub use tara::spectral as stellar_spectral;
/// Star properties and main-sequence models.
pub use tara::star;

pub use tara::error::TaraError;

// ---------------------------------------------------------------------------
// brahmanda — cosmology
// ---------------------------------------------------------------------------

/// Cosmic web topology (voids, filaments, nodes).
pub use brahmanda::cosmic_web;
/// Cosmological models (Friedmann, Planck parameters).
pub use brahmanda::cosmology;
/// Dark matter halo profiles.
pub use brahmanda::halo;
/// Galaxy morphology (Hubble sequence).
pub use brahmanda::morphology as galaxy_morphology;
/// Matter power spectrum.
pub use brahmanda::power_spectrum;

pub use brahmanda::BrahmandaError;

// ---------------------------------------------------------------------------
// badal — weather and atmosphere
// ---------------------------------------------------------------------------

/// Atmospheric state (pressure, temperature, humidity profiles).
pub use badal::atmosphere as weather_atmosphere;
/// Cloud types and formation.
pub use badal::cloud;
/// Mesoscale processes (sea breeze, mountain waves).
pub use badal::mesoscale;
/// Moisture and humidity calculations.
pub use badal::moisture;
/// Precipitation models (rain, snow, sleet, hail).
pub use badal::precipitation;
/// Atmospheric pressure systems.
pub use badal::pressure;
/// Solar/terrestrial radiation budget.
pub use badal::radiation;
/// Severe weather (thunderstorms, tornadoes, hurricanes).
pub use badal::severe;
/// Atmospheric stability indices.
pub use badal::stability as atmo_stability;
/// Wind patterns and profiles.
pub use badal::wind as weather_wind;

pub use badal::BadalError;

// ---------------------------------------------------------------------------
// Celestial body component
// ---------------------------------------------------------------------------

/// Celestial body for orbital simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestialBody {
    /// Name of the body.
    pub name: String,
    /// Mass (kg).
    pub mass: f64,
    /// Orbital semi-major axis (m).
    pub semi_major_axis: f64,
    /// Orbital eccentricity (0 = circular, <1 = elliptical).
    pub eccentricity: f64,
    /// Orbital inclination (radians).
    pub inclination: f64,
    /// Current true anomaly (radians).
    pub true_anomaly: f64,
    /// Whether this body is active in simulation.
    pub active: bool,
}

impl CelestialBody {
    /// Create a new celestial body in a circular orbit.
    pub fn circular(name: impl Into<String>, mass: f64, radius: f64) -> Self {
        Self {
            name: name.into(),
            mass,
            semi_major_axis: radius,
            eccentricity: 0.0,
            inclination: 0.0,
            true_anomaly: 0.0,
            active: true,
        }
    }

    /// Create a new celestial body in an elliptical orbit.
    pub fn elliptical(
        name: impl Into<String>,
        mass: f64,
        semi_major_axis: f64,
        eccentricity: f64,
    ) -> Self {
        Self {
            name: name.into(),
            mass,
            semi_major_axis,
            eccentricity,
            inclination: 0.0,
            true_anomaly: 0.0,
            active: true,
        }
    }

    /// Set the inclination.
    pub fn with_inclination(mut self, inc: f64) -> Self {
        self.inclination = inc;
        self
    }
}

// ---------------------------------------------------------------------------
// Weather zone component
// ---------------------------------------------------------------------------

/// Weather zone for atmospheric simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherZone {
    /// Temperature (K).
    pub temperature: f64,
    /// Atmospheric pressure (Pa).
    pub pressure: f64,
    /// Relative humidity (0.0–1.0).
    pub humidity: f64,
    /// Wind speed (m/s).
    pub wind_speed: f64,
    /// Wind direction (radians, 0 = north).
    pub wind_direction: f64,
    /// Cloud cover (0.0–1.0).
    pub cloud_cover: f64,
    /// Current weather condition.
    pub condition: WeatherCondition,
    /// Whether this zone is active.
    pub active: bool,
}

/// Weather condition for a zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum WeatherCondition {
    /// Clear sky.
    #[default]
    Clear,
    /// Partly cloudy.
    PartlyCloudy,
    /// Overcast.
    Overcast,
    /// Rain.
    Rain,
    /// Snow.
    Snow,
    /// Thunderstorm.
    Thunderstorm,
    /// Fog.
    Fog,
}

impl WeatherZone {
    /// Create a clear-sky weather zone at standard conditions.
    pub fn standard() -> Self {
        Self {
            temperature: 288.15, // 15°C
            pressure: 101325.0,  // 1 atm
            humidity: 0.5,
            wind_speed: 0.0,
            wind_direction: 0.0,
            cloud_cover: 0.0,
            condition: WeatherCondition::Clear,
            active: true,
        }
    }

    /// Set temperature.
    pub fn with_temperature(mut self, t: f64) -> Self {
        self.temperature = t;
        self
    }

    /// Set humidity.
    pub fn with_humidity(mut self, h: f64) -> Self {
        self.humidity = h.clamp(0.0, 1.0);
        self
    }

    /// Set wind.
    pub fn with_wind(mut self, speed: f64, direction: f64) -> Self {
        self.wind_speed = speed;
        self.wind_direction = direction;
        self
    }

    /// Set the weather condition.
    pub fn with_condition(mut self, condition: WeatherCondition) -> Self {
        self.condition = condition;
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn celestial_circular() {
        let body = CelestialBody::circular("Earth", 5.972e24, 1.496e11);
        assert_eq!(body.eccentricity, 0.0);
        assert_eq!(body.name, "Earth");
    }

    #[test]
    fn celestial_elliptical() {
        let body =
            CelestialBody::elliptical("Mars", 6.417e23, 2.279e11, 0.0934).with_inclination(0.032);
        assert!(body.eccentricity > 0.0);
        assert!(body.inclination > 0.0);
    }

    #[test]
    fn weather_standard() {
        let w = WeatherZone::standard();
        assert_eq!(w.condition, WeatherCondition::Clear);
        assert!((w.temperature - 288.15).abs() < 0.01);
    }

    #[test]
    fn weather_builder() {
        let w = WeatherZone::standard()
            .with_temperature(260.0)
            .with_humidity(0.9)
            .with_wind(15.0, std::f64::consts::PI)
            .with_condition(WeatherCondition::Snow);
        assert_eq!(w.condition, WeatherCondition::Snow);
        assert_eq!(w.wind_speed, 15.0);
    }

    #[test]
    fn celestial_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world
            .insert_component(e, CelestialBody::circular("Moon", 7.342e22, 3.844e8))
            .unwrap();
        assert!(world.has_component::<CelestialBody>(e));
    }

    #[test]
    fn weather_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world.insert_component(e, WeatherZone::standard()).unwrap();
        let w = world.get_component::<WeatherZone>(e).unwrap();
        assert_eq!(w.condition, WeatherCondition::Clear);
    }
}
