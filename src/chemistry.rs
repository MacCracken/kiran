//! Chemistry and materials via kimiya, khanij, tanmatra, and kana
//!
//! Bridges the AGNOS chemistry stack with kiran's ECS:
//! - **kimiya** — Chemistry (elements, molecules, reactions, kinetics, thermochemistry)
//! - **khanij** — Geology (minerals, rocks, soil, crystals, tectonics, weathering)
//! - **tanmatra** — Atomic physics (nuclei, particles, decay, quantum numbers)
//! - **kana** — Quantum mechanics (state vectors, operators, circuits, entanglement)
//!
//! Core types:
//! - [`ChemicalBody`] component for reactive materials
//! - [`GeologicalBody`] component for geological simulation
//! - [`RadioactiveSource`] component for decay simulation

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// kimiya — chemistry
// ---------------------------------------------------------------------------

/// Electrochemistry (Nernst equation, galvanic cells).
pub use kimiya::electrochemistry;
/// Periodic table elements and properties.
pub use kimiya::element;
/// Gas laws (ideal, van der Waals).
pub use kimiya::gas;
/// Reaction kinetics (rate laws, Arrhenius).
pub use kimiya::kinetics;
/// Molecular structure and composition.
pub use kimiya::molecule;
/// Nuclear chemistry.
pub use kimiya::nuclear;
/// Phase equilibria.
pub use kimiya::phase as chem_phase;
/// Chemical reactions and balancing.
pub use kimiya::reaction;
/// Solution chemistry (concentration, pH, buffers).
pub use kimiya::solution;
/// Spectroscopy (absorption, emission).
pub use kimiya::spectroscopy;
/// Stoichiometry calculations.
pub use kimiya::stoichiometry;
/// Thermochemistry (enthalpy, Hess's law).
pub use kimiya::thermochem;

pub use kimiya::KimiyaError;

// ---------------------------------------------------------------------------
// khanij — geology and mineralogy
// ---------------------------------------------------------------------------

/// Crystal systems and structures.
pub use khanij::crystal;
/// Crystallography (lattice, symmetry).
pub use khanij::crystallography;
/// Radiometric dating.
pub use khanij::dating;
/// Mineral properties and identification.
pub use khanij::mineral;
/// Ore deposits and resources.
pub use khanij::ore;
/// Rock types and composition.
pub use khanij::rock;
/// Sedimentary processes.
pub use khanij::sediment;
/// Soil profiles and classification.
pub use khanij::soil as geo_soil;
/// Stratigraphy (rock layers).
pub use khanij::stratigraphy;
/// Tectonic processes.
pub use khanij::tectonics;
/// Geological timescale.
pub use khanij::timescale;
/// Volcanic processes.
pub use khanij::volcanic;
/// Weathering and erosion.
pub use khanij::weathering;

pub use khanij::KhanijError;

// ---------------------------------------------------------------------------
// tanmatra — atomic and subatomic physics
// ---------------------------------------------------------------------------

/// Atomic structure (orbitals, quantum numbers, electron config).
pub use tanmatra::atomic;
/// Physical constants (Planck, Boltzmann, fine structure).
pub use tanmatra::constants as atomic_constants;
/// Radioactive decay modes and half-lives.
pub use tanmatra::decay;
/// Nuclear structure and properties.
pub use tanmatra::nucleus;
/// Fundamental particles (quarks, leptons, bosons).
pub use tanmatra::particle;
/// Nuclear and particle reactions.
pub use tanmatra::reaction as nuclear_reaction;
/// Relativistic mechanics (four-momentum, Lorentz).
pub use tanmatra::relativity;
/// Particle scattering (cross sections, form factors).
pub use tanmatra::scattering as particle_scattering;

pub use tanmatra::prelude::TanmatraError;

// ---------------------------------------------------------------------------
// kana — quantum mechanics
// ---------------------------------------------------------------------------

/// Quantum circuits and gates.
pub use kana::circuit as quantum_circuit;
/// Quantum dynamics (Hamiltonian evolution).
pub use kana::dynamics as quantum_dynamics;
/// Quantum entanglement (Bell states, concurrence).
pub use kana::entanglement;
/// Quantum operators (unitary, Hermitian).
pub use kana::operator;
/// Quantum state vectors.
pub use kana::state;

pub use kana::KanaError;

// ---------------------------------------------------------------------------
// Chemical body component
// ---------------------------------------------------------------------------

/// Chemical body for reactive material simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemicalBody {
    /// Chemical formula or identifier.
    pub formula: String,
    /// Temperature (K).
    pub temperature: f64,
    /// Concentration (mol/L) if in solution.
    pub concentration: f64,
    /// pH value (0–14).
    pub ph: f64,
    /// Whether this body is active in simulation.
    pub active: bool,
}

impl ChemicalBody {
    /// Create a new chemical body.
    pub fn new(formula: impl Into<String>, temperature: f64) -> Self {
        Self {
            formula: formula.into(),
            temperature,
            concentration: 0.0,
            ph: 7.0,
            active: true,
        }
    }

    /// Set concentration.
    pub fn with_concentration(mut self, c: f64) -> Self {
        self.concentration = c;
        self
    }

    /// Set pH.
    pub fn with_ph(mut self, ph: f64) -> Self {
        self.ph = ph;
        self
    }
}

// ---------------------------------------------------------------------------
// Geological body component
// ---------------------------------------------------------------------------

/// Geological body for terrain/mineral simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeologicalBody {
    /// Rock or mineral type name.
    pub material_name: String,
    /// Hardness (Mohs scale, 1–10).
    pub hardness: f64,
    /// Density (kg/m³).
    pub density: f64,
    /// Weathering rate (0.0 = inert, 1.0 = rapidly weathering).
    pub weathering_rate: f64,
    /// Whether this body is active.
    pub active: bool,
}

impl GeologicalBody {
    /// Create a new geological body.
    pub fn new(material_name: impl Into<String>, hardness: f64, density: f64) -> Self {
        Self {
            material_name: material_name.into(),
            hardness,
            density,
            weathering_rate: 0.0,
            active: true,
        }
    }

    /// Set weathering rate.
    pub fn with_weathering(mut self, rate: f64) -> Self {
        self.weathering_rate = rate;
        self
    }
}

// ---------------------------------------------------------------------------
// Radioactive source component
// ---------------------------------------------------------------------------

/// Radioactive source for decay simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioactiveSource {
    /// Isotope identifier (e.g. "U-238", "C-14").
    pub isotope: String,
    /// Half-life in seconds.
    pub half_life: f64,
    /// Current activity (decays per second).
    pub activity: f64,
    /// Whether this source is active.
    pub active: bool,
}

impl RadioactiveSource {
    /// Create a new radioactive source.
    pub fn new(isotope: impl Into<String>, half_life: f64, activity: f64) -> Self {
        Self {
            isotope: isotope.into(),
            half_life,
            activity,
            active: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chemical_body_builder() {
        let c = ChemicalBody::new("NaCl", 298.0)
            .with_concentration(0.1)
            .with_ph(7.0);
        assert_eq!(c.formula, "NaCl");
        assert_eq!(c.concentration, 0.1);
    }

    #[test]
    fn geological_body_builder() {
        let g = GeologicalBody::new("Granite", 7.0, 2700.0).with_weathering(0.1);
        assert_eq!(g.hardness, 7.0);
        assert_eq!(g.weathering_rate, 0.1);
    }

    #[test]
    fn radioactive_source() {
        let r = RadioactiveSource::new("C-14", 5730.0 * 365.25 * 86400.0, 1e6);
        assert_eq!(r.isotope, "C-14");
        assert!(r.half_life > 0.0);
    }

    #[test]
    fn chemical_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world
            .insert_component(e, ChemicalBody::new("H2O", 373.0))
            .unwrap();
        assert!(world.has_component::<ChemicalBody>(e));
    }

    #[test]
    fn geological_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world
            .insert_component(e, GeologicalBody::new("Quartz", 7.0, 2650.0))
            .unwrap();
        let g = world.get_component::<GeologicalBody>(e).unwrap();
        assert_eq!(g.material_name, "Quartz");
    }
}
