//! Advanced dynamics via bijli, dravya, ushma, and pavan
//!
//! Bridges the AGNOS dynamics stack with kiran's ECS:
//! - **bijli** — Electromagnetism (fields, charges, waves, circuits, FDTD)
//! - **dravya** — Material science (stress, strain, elasticity, fatigue, fracture)
//! - **ushma** — Thermodynamics (heat transfer, phase transitions, equations of state)
//! - **pavan** — Aerodynamics (atmosphere, airfoils, forces, wind, panel/VLM methods)
//!
//! Core types:
//! - [`EmField`] component for electromagnetic field sources
//! - [`MaterialBody`] component for stress/strain simulation
//! - [`ThermalBody`] component for heat transfer simulation
//! - [`AeroSurface`] component for aerodynamic force computation

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// bijli — electromagnetism
// ---------------------------------------------------------------------------

/// Point charges and Coulomb interactions.
pub use bijli::charge;
/// Electrical circuits (Ohm's law, impedance, RC/RL/RLC).
pub use bijli::circuit;
/// Electric and magnetic field calculations.
pub use bijli::field;
/// Dielectric and magnetic material properties.
pub use bijli::material as em_material;
/// Maxwell's equations solvers.
pub use bijli::maxwell;
/// Polarization (Jones vectors, Mueller matrices).
pub use bijli::polarization;
/// Scattering (Mie, Rayleigh).
pub use bijli::scattering;
/// EM wave propagation.
pub use bijli::wave as em_wave;

pub use bijli::BijliError;

// ---------------------------------------------------------------------------
// dravya — material science
// ---------------------------------------------------------------------------

/// Composite laminate theory and failure criteria.
pub use dravya::composite;
/// Constitutive models (bilinear, Ramberg-Osgood, Neo-Hookean, Johnson-Cook).
pub use dravya::constitutive;
/// Elastic moduli and Hooke's law.
pub use dravya::elastic;
/// Fatigue models (Basquin, Coffin-Manson, Miner's rule).
pub use dravya::fatigue;
/// Fracture mechanics (stress intensity, Paris law).
pub use dravya::fracture;
/// Material definitions and mechanical properties.
pub use dravya::material as mech_material;
/// Strain tensor (Voigt notation).
pub use dravya::strain;
/// Stress tensor (Voigt notation).
pub use dravya::stress;
/// Yield criteria (von Mises, Tresca, Drucker-Prager).
pub use dravya::yield_criteria;

pub use dravya::DravyaError;

// ---------------------------------------------------------------------------
// ushma — thermodynamics
// ---------------------------------------------------------------------------

/// Chemical thermodynamics (Hess's law, equilibrium).
pub use ushma::chem as thermo_chem;
/// Thermodynamic cycles (Otto, Diesel, Brayton, Rankine).
pub use ushma::cycle;
/// Entropy, free energy, thermodynamic potentials.
pub use ushma::entropy;
/// Thermal material properties (conductivity, specific heat, density).
pub use ushma::material as thermal_material;
/// Phase transitions (Clausius-Clapeyron).
pub use ushma::phase;
/// Statistical thermodynamics (partition functions).
pub use ushma::stat as thermo_stat;
/// Equations of state (ideal gas, van der Waals, Peng-Robinson).
pub use ushma::state as thermo_state;
/// Heat transfer (conduction, convection, radiation).
pub use ushma::transfer;

pub use ushma::UshmaError;

// ---------------------------------------------------------------------------
// pavan — aerodynamics
// ---------------------------------------------------------------------------

/// NACA airfoil generation.
pub use pavan::airfoil;
/// ISA standard atmosphere.
pub use pavan::atmosphere as aero_atmosphere;
/// Boundary layer analysis.
pub use pavan::boundary;
/// Lift/drag coefficient models.
pub use pavan::coefficients;
/// Aerodynamic forces (lift, drag, moment).
pub use pavan::forces as aero_forces;
/// 2D panel method (Hess-Smith).
pub use pavan::panel;
/// Flight stability analysis.
pub use pavan::stability;
/// Aerodynamic body definition.
pub use pavan::vehicle;
/// Vortex Lattice Method (3D finite wings).
pub use pavan::vlm;
/// Wind field modeling.
pub use pavan::wind;

pub use pavan::PavanError;

// ---------------------------------------------------------------------------
// EM field component
// ---------------------------------------------------------------------------

/// Electromagnetic field source attached to an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmField {
    /// Electric field vector (V/m).
    pub electric: [f64; 3],
    /// Magnetic field vector (T).
    pub magnetic: [f64; 3],
    /// Whether this source is active.
    pub active: bool,
}

impl EmField {
    /// Create a new EM field source.
    pub fn new(electric: [f64; 3], magnetic: [f64; 3]) -> Self {
        Self {
            electric,
            magnetic,
            active: true,
        }
    }

    /// Create a purely electric field.
    pub fn electric_only(e: [f64; 3]) -> Self {
        Self::new(e, [0.0; 3])
    }

    /// Create a purely magnetic field.
    pub fn magnetic_only(b: [f64; 3]) -> Self {
        Self::new([0.0; 3], b)
    }
}

impl Default for EmField {
    fn default() -> Self {
        Self::new([0.0; 3], [0.0; 3])
    }
}

// ---------------------------------------------------------------------------
// Material body component
// ---------------------------------------------------------------------------

/// Material body for stress/strain simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialBody {
    /// Young's modulus (Pa).
    pub youngs_modulus: f64,
    /// Poisson's ratio.
    pub poisson_ratio: f64,
    /// Yield strength (Pa).
    pub yield_strength: f64,
    /// Density (kg/m³).
    pub density: f64,
    /// Accumulated fatigue damage (0.0 = pristine, 1.0 = failure).
    pub fatigue_damage: f64,
    /// Whether this body is active in simulation.
    pub active: bool,
}

impl MaterialBody {
    /// Create a material body from basic properties.
    pub fn new(youngs_modulus: f64, poisson_ratio: f64, yield_strength: f64, density: f64) -> Self {
        Self {
            youngs_modulus,
            poisson_ratio,
            yield_strength,
            density,
            fatigue_damage: 0.0,
            active: true,
        }
    }

    /// Create a steel material body.
    pub fn steel() -> Self {
        Self::new(200e9, 0.3, 250e6, 7850.0)
    }

    /// Create an aluminum material body.
    pub fn aluminum() -> Self {
        Self::new(69e9, 0.33, 276e6, 2700.0)
    }

    /// Whether this material has exceeded its yield strength.
    #[must_use]
    #[inline]
    pub fn is_yielded(&self, stress: f64) -> bool {
        stress >= self.yield_strength
    }

    /// Whether fatigue failure has occurred.
    #[must_use]
    #[inline]
    pub fn is_failed(&self) -> bool {
        self.fatigue_damage >= 1.0
    }
}

// ---------------------------------------------------------------------------
// Thermal body component
// ---------------------------------------------------------------------------

/// Thermal body for heat transfer simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalBody {
    /// Current temperature (K).
    pub temperature: f64,
    /// Thermal conductivity (W/m·K).
    pub conductivity: f64,
    /// Specific heat capacity (J/kg·K).
    pub specific_heat: f64,
    /// Mass (kg).
    pub mass: f64,
    /// Current phase of matter.
    pub phase: ThermalPhase,
    /// Whether this body is active in simulation.
    pub active: bool,
}

/// Phase of matter for a thermal body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ThermalPhase {
    /// Solid state.
    Solid,
    /// Liquid state.
    Liquid,
    /// Gaseous state.
    Gas,
}

impl ThermalBody {
    /// Create a new thermal body.
    pub fn new(temperature: f64, conductivity: f64, specific_heat: f64, mass: f64) -> Self {
        Self {
            temperature,
            conductivity,
            specific_heat,
            mass,
            phase: ThermalPhase::Solid,
            active: true,
        }
    }

    /// Set the phase.
    pub fn with_phase(mut self, phase: ThermalPhase) -> Self {
        self.phase = phase;
        self
    }

    /// Apply heat energy (J) and update temperature.
    pub fn apply_heat(&mut self, energy_joules: f64) {
        if self.mass > 0.0 && self.specific_heat > 0.0 {
            self.temperature += energy_joules / (self.mass * self.specific_heat);
        }
    }
}

// ---------------------------------------------------------------------------
// Aero surface component
// ---------------------------------------------------------------------------

/// Aerodynamic surface for force computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AeroSurface {
    /// Reference area (m²).
    pub reference_area: f64,
    /// Parasitic drag coefficient.
    pub cd0: f64,
    /// Wing aspect ratio.
    pub aspect_ratio: f64,
    /// Oswald efficiency factor.
    pub oswald_efficiency: f64,
    /// Current angle of attack (radians).
    pub angle_of_attack: f64,
    /// Whether this surface is active.
    pub active: bool,
}

impl AeroSurface {
    /// Create a new aerodynamic surface.
    pub fn new(reference_area: f64, cd0: f64, aspect_ratio: f64) -> Self {
        Self {
            reference_area,
            cd0,
            aspect_ratio,
            oswald_efficiency: 0.85,
            angle_of_attack: 0.0,
            active: true,
        }
    }

    /// Set the Oswald efficiency factor.
    pub fn with_oswald(mut self, e: f64) -> Self {
        self.oswald_efficiency = e;
        self
    }

    /// Set the angle of attack in radians.
    pub fn with_aoa(mut self, aoa: f64) -> Self {
        self.angle_of_attack = aoa;
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
    fn em_field_default() {
        let f = EmField::default();
        assert_eq!(f.electric, [0.0; 3]);
        assert_eq!(f.magnetic, [0.0; 3]);
        assert!(f.active);
    }

    #[test]
    fn em_field_electric_only() {
        let f = EmField::electric_only([1.0, 0.0, 0.0]);
        assert_eq!(f.electric[0], 1.0);
        assert_eq!(f.magnetic, [0.0; 3]);
    }

    #[test]
    fn material_body_steel() {
        let m = MaterialBody::steel();
        assert_eq!(m.youngs_modulus, 200e9);
        assert!(!m.is_failed());
    }

    #[test]
    fn material_body_yield() {
        let m = MaterialBody::steel();
        assert!(!m.is_yielded(100e6));
        assert!(m.is_yielded(300e6));
    }

    #[test]
    fn thermal_body_heat() {
        let mut t = ThermalBody::new(300.0, 50.0, 500.0, 1.0);
        t.apply_heat(500.0); // 500J into 1kg @ 500 J/kg·K = +1K
        assert!((t.temperature - 301.0).abs() < 1e-10);
    }

    #[test]
    fn thermal_body_phase() {
        let t = ThermalBody::new(373.0, 0.6, 4186.0, 1.0).with_phase(ThermalPhase::Liquid);
        assert_eq!(t.phase, ThermalPhase::Liquid);
    }

    #[test]
    fn aero_surface_builder() {
        let s = AeroSurface::new(16.0, 0.02, 8.0)
            .with_oswald(0.9)
            .with_aoa(0.1);
        assert_eq!(s.reference_area, 16.0);
        assert_eq!(s.oswald_efficiency, 0.9);
        assert_eq!(s.angle_of_attack, 0.1);
    }

    #[test]
    fn em_field_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world
            .insert_component(e, EmField::electric_only([5.0, 0.0, 0.0]))
            .unwrap();
        assert!(world.has_component::<EmField>(e));
    }

    #[test]
    fn material_body_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world.insert_component(e, MaterialBody::steel()).unwrap();
        let m = world.get_component::<MaterialBody>(e).unwrap();
        assert_eq!(m.density, 7850.0);
    }

    #[test]
    fn thermal_body_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world
            .insert_component(e, ThermalBody::new(293.0, 50.0, 500.0, 10.0))
            .unwrap();
        assert!(world.has_component::<ThermalBody>(e));
    }
}
