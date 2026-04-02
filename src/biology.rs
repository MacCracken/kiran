//! Biology simulation via sharira, jivanu, rasayan, and vanaspati
//!
//! Bridges the AGNOS biology stack with kiran's ECS:
//! - **sharira** — Physiology (skeleton, muscles, gait, IK, biomechanics)
//! - **jivanu** — Microbiology (growth, metabolism, genetics, epidemiology)
//! - **rasayan** — Biochemistry (enzymes, metabolic pathways, signaling)
//! - **vanaspati** — Botany (plant growth, photosynthesis, seasonal cycles, ecology)
//!
//! Core types:
//! - [`Physiology`] component for skeletal/muscular creatures
//! - [`Microbe`] component for microbial populations
//! - [`MetabolicProfile`] component for cellular biochemistry
//! - [`PlantState`] component for botanical simulation

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// sharira — physiology
// ---------------------------------------------------------------------------

/// Allometric scaling (mass-to-dimension).
pub use sharira::allometry;
/// Biomechanical analysis.
pub use sharira::biomechanics;
/// Body composition and structure.
pub use sharira::body;
/// Muscle fatigue modeling.
pub use sharira::fatigue as phys_fatigue;
/// Gait cycles and locomotion patterns.
pub use sharira::gait;
/// Inverse kinematics solvers.
pub use sharira::ik;
/// Joint types and constraints.
pub use sharira::joint;
/// Forward kinematics.
pub use sharira::kinematics;
/// Body morphology and proportions.
pub use sharira::morphology;
/// Muscle force and activation.
pub use sharira::muscle;
/// Pose representation (joint angles/transforms).
pub use sharira::pose;
/// Preset body configurations.
pub use sharira::preset as body_preset;
/// Skeletal system (bones, hierarchy).
pub use sharira::skeleton;

pub use sharira::ShariraError;

// ---------------------------------------------------------------------------
// jivanu — microbiology
// ---------------------------------------------------------------------------

/// Biofilm formation and dynamics.
pub use jivanu::biofilm;
/// Epidemiology (SIR, SEIR models).
pub use jivanu::epidemiology;
/// Genetics (mutation, selection, gene transfer).
pub use jivanu::genetics;
/// Microbial growth models (exponential, logistic, Monod).
pub use jivanu::growth;
/// Microbial metabolism.
pub use jivanu::metabolism as micro_metabolism;
/// Antimicrobial resistance.
pub use jivanu::resistance;
/// Taxonomic classification.
pub use jivanu::taxonomy;

pub use jivanu::JivanuError;

// ---------------------------------------------------------------------------
// rasayan — biochemistry
// ---------------------------------------------------------------------------

/// Calcium signaling.
pub use rasayan::calcium;
/// Enzyme kinetics (Michaelis-Menten, Hill equation, inhibition).
pub use rasayan::enzyme;
/// Electron transport chain.
pub use rasayan::etc;
/// Glycolysis pathway.
pub use rasayan::glycolysis;
/// Hormonal signaling.
pub use rasayan::hormonal;
/// Membrane transport and ion channels.
pub use rasayan::membrane;
/// Metabolic state (ATP, NAD, glucose, O2).
pub use rasayan::metabolism as cell_metabolism;
/// Neurotransmitter synthesis and degradation.
pub use rasayan::neurotransmitter as biochem_nt;
/// Metabolic pathways.
pub use rasayan::pathway;
/// Protein structure and domains.
pub use rasayan::protein;
/// Signal transduction (second messengers, GPCR cascades).
pub use rasayan::signal;
/// TCA cycle.
pub use rasayan::tca;

pub use rasayan::RasayanError;

// ---------------------------------------------------------------------------
// vanaspati — botany
// ---------------------------------------------------------------------------

/// Biomass pools (leaf, stem, root, reproductive).
pub use vanaspati::biomass;
/// Fire ecology (bark protection, resprouting, serotiny).
pub use vanaspati::fire;
/// Plant growth models (logistic, ontogenetic stages).
pub use vanaspati::growth as plant_growth;
/// Herbivory effects.
pub use vanaspati::herbivory;
/// Plant mortality (drought, frost, fire, competition).
pub use vanaspati::mortality;
/// Mycorrhizal symbiosis.
pub use vanaspati::mycorrhiza;
/// Nitrogen cycling.
pub use vanaspati::nitrogen;
/// Plant functional types and presets.
pub use vanaspati::pft;
/// Phenology (growing degree days, lifecycle events).
pub use vanaspati::phenology;
/// Photosynthesis (C3, C4, CAM pathways).
pub use vanaspati::photosynthesis;
/// Plant reproduction and seed dispersal.
pub use vanaspati::reproduction;
/// Root systems and water uptake.
pub use vanaspati::root;
/// Seasonal dynamics and day length.
pub use vanaspati::season;
/// Ecological succession.
pub use vanaspati::succession;
/// Soil water balance modeling.
pub use vanaspati::water as soil_water;

pub use vanaspati::VanaspatiError;

// ---------------------------------------------------------------------------
// Physiology component
// ---------------------------------------------------------------------------

/// Physiological state for a creature entity.
///
/// Wraps sharira's skeletal/muscular systems for locomotion and IK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Physiology {
    /// Current gait type.
    pub gait_type: PhysiologyGait,
    /// Movement speed (m/s).
    pub speed: f32,
    /// Overall muscle fatigue (0.0 = fresh, 1.0 = exhausted).
    pub fatigue: f32,
    /// Body mass (kg).
    pub mass: f32,
    /// Whether this physiology is active.
    pub active: bool,
}

/// Gait type for the physiology component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum PhysiologyGait {
    /// Walking gait.
    #[default]
    Walk,
    /// Running gait.
    Run,
    /// Trotting gait.
    Trot,
    /// Galloping gait.
    Gallop,
    /// Crawling gait.
    Crawl,
    /// Swimming gait.
    Swim,
    /// Flying gait.
    Fly,
}

impl Physiology {
    /// Create a new physiology component.
    pub fn new(mass: f32) -> Self {
        Self {
            gait_type: PhysiologyGait::Walk,
            speed: 0.0,
            fatigue: 0.0,
            mass,
            active: true,
        }
    }

    /// Set the gait type.
    pub fn with_gait(mut self, gait: PhysiologyGait) -> Self {
        self.gait_type = gait;
        self
    }

    /// Set movement speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
}

// ---------------------------------------------------------------------------
// Microbe component
// ---------------------------------------------------------------------------

/// Microbial population attached to an entity (surface, region, host).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Microbe {
    /// Species or strain name.
    pub species: String,
    /// Population count.
    pub population: f64,
    /// Carrying capacity of the environment.
    pub carrying_capacity: f64,
    /// Growth rate (per hour).
    pub growth_rate: f64,
    /// Current growth phase.
    pub phase: MicrobePhase,
    /// Whether this population is active.
    pub active: bool,
}

/// Growth phase of a microbial population.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum MicrobePhase {
    /// Lag phase (adapting to environment).
    #[default]
    Lag,
    /// Exponential growth.
    Exponential,
    /// Stationary (at carrying capacity).
    Stationary,
    /// Death phase (declining).
    Death,
}

impl Microbe {
    /// Create a new microbial population.
    pub fn new(species: impl Into<String>, population: f64, carrying_capacity: f64) -> Self {
        Self {
            species: species.into(),
            population,
            carrying_capacity,
            growth_rate: 0.5,
            phase: MicrobePhase::Lag,
            active: true,
        }
    }

    /// Set the growth rate.
    pub fn with_growth_rate(mut self, rate: f64) -> Self {
        self.growth_rate = rate;
        self
    }
}

// ---------------------------------------------------------------------------
// Metabolism component
// ---------------------------------------------------------------------------

/// Cellular metabolic state for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicProfile {
    /// ATP level (normalized 0.0–1.0).
    pub atp: f64,
    /// Glucose availability (normalized 0.0–1.0).
    pub glucose: f64,
    /// Oxygen availability (normalized 0.0–1.0).
    pub oxygen: f64,
    /// Lactate accumulation (normalized 0.0–1.0).
    pub lactate: f64,
    /// Whether metabolism is active.
    pub active: bool,
}

impl MetabolicProfile {
    /// Create a new metabolic profile at homeostasis.
    pub fn new() -> Self {
        Self {
            atp: 1.0,
            glucose: 1.0,
            oxygen: 1.0,
            lactate: 0.0,
            active: true,
        }
    }

    /// Whether the cell is in an energy crisis (ATP critically low).
    #[must_use]
    #[inline]
    pub fn is_energy_crisis(&self) -> bool {
        self.atp < 0.2
    }

    /// Whether anaerobic conditions are present.
    #[must_use]
    #[inline]
    pub fn is_anaerobic(&self) -> bool {
        self.oxygen < 0.1
    }
}

impl Default for MetabolicProfile {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Plant component
// ---------------------------------------------------------------------------

/// Botanical simulation state for a plant entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantState {
    /// Species or functional type name.
    pub species: String,
    /// Current height (m).
    pub height: f64,
    /// Current growth stage.
    pub stage: PlantStage,
    /// Leaf biomass (kg).
    pub leaf_mass: f64,
    /// Stem biomass (kg).
    pub stem_mass: f64,
    /// Root biomass (kg).
    pub root_mass: f64,
    /// Soil water content (0.0–1.0).
    pub soil_water: f64,
    /// Whether this plant is active.
    pub active: bool,
}

/// Growth stage for a plant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum PlantStage {
    /// Dormant seed.
    #[default]
    Seed,
    /// Germinating.
    Germination,
    /// Young seedling.
    Seedling,
    /// Vegetative growth.
    Vegetative,
    /// Producing flowers.
    Flowering,
    /// Producing fruit/seeds.
    Fruiting,
    /// Leaf drop and decline.
    Senescence,
    /// Winter dormancy.
    Dormant,
}

impl PlantState {
    /// Create a new plant at the seed stage.
    pub fn seed(species: impl Into<String>) -> Self {
        Self {
            species: species.into(),
            height: 0.0,
            stage: PlantStage::Seed,
            leaf_mass: 0.0,
            stem_mass: 0.0,
            root_mass: 0.0,
            soil_water: 0.5,
            active: true,
        }
    }

    /// Create a mature plant at the vegetative stage.
    pub fn mature(species: impl Into<String>, height: f64) -> Self {
        Self {
            species: species.into(),
            height,
            stage: PlantStage::Vegetative,
            leaf_mass: height * 0.1,
            stem_mass: height * 0.5,
            root_mass: height * 0.3,
            soil_water: 0.5,
            active: true,
        }
    }

    /// Total above-ground biomass (kg).
    #[must_use]
    #[inline]
    pub fn above_ground_mass(&self) -> f64 {
        self.leaf_mass + self.stem_mass
    }

    /// Total biomass (kg).
    #[must_use]
    #[inline]
    pub fn total_mass(&self) -> f64 {
        self.leaf_mass + self.stem_mass + self.root_mass
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physiology_builder() {
        let p = Physiology::new(70.0)
            .with_gait(PhysiologyGait::Run)
            .with_speed(3.0);
        assert_eq!(p.mass, 70.0);
        assert_eq!(p.gait_type, PhysiologyGait::Run);
        assert_eq!(p.speed, 3.0);
    }

    #[test]
    fn microbe_builder() {
        let m = Microbe::new("E. coli", 1000.0, 1e9).with_growth_rate(0.8);
        assert_eq!(m.species, "E. coli");
        assert_eq!(m.growth_rate, 0.8);
        assert_eq!(m.phase, MicrobePhase::Lag);
    }

    #[test]
    fn metabolic_homeostasis() {
        let m = MetabolicProfile::new();
        assert!(!m.is_energy_crisis());
        assert!(!m.is_anaerobic());
    }

    #[test]
    fn metabolic_crisis() {
        let m = MetabolicProfile {
            atp: 0.1,
            oxygen: 0.05,
            ..Default::default()
        };
        assert!(m.is_energy_crisis());
        assert!(m.is_anaerobic());
    }

    #[test]
    fn plant_seed() {
        let p = PlantState::seed("Oak");
        assert_eq!(p.stage, PlantStage::Seed);
        assert_eq!(p.height, 0.0);
        assert_eq!(p.total_mass(), 0.0);
    }

    #[test]
    fn plant_mature() {
        let p = PlantState::mature("Pine", 10.0);
        assert_eq!(p.stage, PlantStage::Vegetative);
        assert!(p.total_mass() > 0.0);
        assert!(p.above_ground_mass() > 0.0);
    }

    #[test]
    fn physiology_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world.insert_component(e, Physiology::new(80.0)).unwrap();
        assert!(world.has_component::<Physiology>(e));
    }

    #[test]
    fn microbe_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world
            .insert_component(e, Microbe::new("S. aureus", 500.0, 1e8))
            .unwrap();
        let m = world.get_component::<Microbe>(e).unwrap();
        assert_eq!(m.species, "S. aureus");
    }

    #[test]
    fn plant_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world.insert_component(e, PlantState::seed("Fern")).unwrap();
        assert!(world.has_component::<PlantState>(e));
    }
}
