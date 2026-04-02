//! World-building via itihas, sankhya, varna, and pramana
//!
//! Bridges the AGNOS world-building stack with kiran's ECS:
//! - **itihas** — World history (civilizations, eras, events, historical figures)
//! - **sankhya** — Historical mathematics (ancient calendars, numeral systems)
//! - **varna** — Multilingual language engine (phonemes, scripts, grammar, lexicons)
//! - **pramana** — Statistics and probability (distributions, Bayesian, Monte Carlo)
//!
//! Core types:
//! - [`CultureProfile`] component for civilization/language identity
//! - [`StochasticSource`] component for probability-driven behavior

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// itihas — world history
// ---------------------------------------------------------------------------

/// Calendar system metadata.
pub use itihas::calendar as history_calendar;
/// Causal chains between events.
pub use itihas::causality;
/// Civilizations and societies.
pub use itihas::civilization;
/// Historical eras and periods.
pub use itihas::era;
/// Historical events.
pub use itihas::event as history_event;
/// Historical figures.
pub use itihas::figure;

pub use itihas::ItihasError;

// ---------------------------------------------------------------------------
// sankhya — historical mathematics
// ---------------------------------------------------------------------------

/// Babylonian mathematics (sexagesimal).
pub use sankhya::babylonian;
/// Chinese calendar.
pub use sankhya::chinese;
/// Egyptian calendar.
pub use sankhya::egyptian;
/// Epoch and Julian day conversions.
pub use sankhya::epoch;
/// Greek mathematics.
pub use sankhya::greek;
/// Gregorian calendar utilities.
pub use sankhya::gregorian;
/// Hebrew lunisolar calendar.
pub use sankhya::hebrew;
/// Islamic (Hijri) calendar.
pub use sankhya::islamic;
/// Mayan calendar (Long Count, Tzolkin, Haab).
pub use sankhya::mayan;
/// Persian (Jalaali) calendar.
pub use sankhya::persian;
/// Roman numeral system.
pub use sankhya::roman;
/// Vedic mathematics.
pub use sankhya::vedic;

pub use sankhya::SankhyaError;

// ---------------------------------------------------------------------------
// varna — multilingual language engine
// ---------------------------------------------------------------------------

/// Dialect overlays and regional variants.
pub use varna::dialect;
/// Grammar typology (isolating, agglutinative, fusional).
pub use varna::grammar;
/// Lexicon entries and vocabulary.
pub use varna::lexicon;
/// Phoneme inventories (IPA, features, manner/place).
pub use varna::phoneme as lang_phoneme;
/// Language registry (ISO 639 codes).
pub use varna::registry;
/// Writing systems (alphabets, syllabaries, logographic).
pub use varna::script;

pub use varna::VarnaError;

// ---------------------------------------------------------------------------
// pramana — statistics and probability
// ---------------------------------------------------------------------------

/// Bayesian inference (prior/posterior updates).
pub use pramana::bayesian as stats_bayesian;
/// Combinatorics (permutations, combinations).
pub use pramana::combinatorics;
/// Descriptive statistics (mean, variance, percentiles).
pub use pramana::descriptive;
/// Probability distributions (Normal, Uniform, Poisson, etc.).
pub use pramana::distribution;
/// Hypothesis testing (t-test, chi-squared, ANOVA).
pub use pramana::hypothesis;
/// Markov chains and HMMs.
pub use pramana::markov;
/// Monte Carlo simulation.
pub use pramana::monte_carlo;
/// Regression models (linear, polynomial).
pub use pramana::regression;
/// Random number generation.
pub use pramana::rng;
/// Time series analysis.
pub use pramana::timeseries;

pub use pramana::PramanaError;

// ---------------------------------------------------------------------------
// Culture profile component
// ---------------------------------------------------------------------------

/// Cultural identity for an entity (NPC, settlement, faction).
///
/// Links an entity to a civilization template, language, and calendar system
/// for world-building and dialogue generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultureProfile {
    /// Civilization or culture name.
    pub civilization: String,
    /// Primary language (ISO 639 code or custom).
    pub language: String,
    /// Calendar system in use.
    pub calendar: String,
    /// Cultural era or period.
    pub era: String,
    /// Whether this profile is active.
    pub active: bool,
}

impl CultureProfile {
    /// Create a new culture profile.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "world")] {
    /// use kiran::lore::CultureProfile;
    ///
    /// let culture = CultureProfile::new("Roman Empire", "la", "julian");
    /// assert_eq!(culture.civilization, "Roman Empire");
    /// assert!(culture.active);
    /// # }
    /// ```
    pub fn new(
        civilization: impl Into<String>,
        language: impl Into<String>,
        calendar: impl Into<String>,
    ) -> Self {
        let civilization = civilization.into();
        let language = language.into();
        let calendar = calendar.into();
        tracing::trace!(
            %civilization, %language, %calendar,
            "created culture profile"
        );
        Self {
            civilization,
            language,
            calendar,
            era: String::new(),
            active: true,
        }
    }

    /// Set the cultural era.
    pub fn with_era(mut self, era: impl Into<String>) -> Self {
        self.era = era.into();
        self
    }
}

// ---------------------------------------------------------------------------
// Stochastic source component
// ---------------------------------------------------------------------------

/// Probability-driven behavior source for an entity.
///
/// Attach to entities that need randomized behavior (loot tables,
/// encounter rates, weather variation, procedural generation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticSource {
    /// Distribution type identifier (e.g. "normal", "uniform", "poisson").
    pub distribution: String,
    /// Primary parameter (mean for normal, min for uniform, lambda for poisson).
    pub param_a: f64,
    /// Secondary parameter (std_dev for normal, max for uniform).
    pub param_b: f64,
    /// Last sampled value.
    #[serde(skip)]
    pub last_sample: f64,
    /// Whether this source is active.
    pub active: bool,
}

impl StochasticSource {
    /// Create a uniform distribution source.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "world")] {
    /// use kiran::lore::StochasticSource;
    ///
    /// let source = StochasticSource::uniform(0.0, 1.0);
    /// assert_eq!(source.distribution, "uniform");
    /// assert_eq!(source.param_a, 0.0);
    /// # }
    /// ```
    pub fn uniform(min: f64, max: f64) -> Self {
        tracing::trace!(
            distribution = "uniform",
            min,
            max,
            "created stochastic source"
        );
        Self {
            distribution: "uniform".into(),
            param_a: min,
            param_b: max,
            last_sample: 0.0,
            active: true,
        }
    }

    /// Create a normal distribution source.
    pub fn normal(mean: f64, std_dev: f64) -> Self {
        tracing::trace!(
            distribution = "normal",
            mean,
            std_dev,
            "created stochastic source"
        );
        Self {
            distribution: "normal".into(),
            param_a: mean,
            param_b: std_dev,
            last_sample: 0.0,
            active: true,
        }
    }

    /// Create a Poisson distribution source.
    pub fn poisson(lambda: f64) -> Self {
        Self {
            distribution: "poisson".into(),
            param_a: lambda,
            param_b: 0.0,
            last_sample: 0.0,
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
    fn culture_profile_builder() {
        let c = CultureProfile::new("Roman Empire", "la", "julian").with_era("Classical");
        assert_eq!(c.civilization, "Roman Empire");
        assert_eq!(c.language, "la");
        assert_eq!(c.era, "Classical");
    }

    #[test]
    fn stochastic_uniform() {
        let s = StochasticSource::uniform(0.0, 1.0);
        assert_eq!(s.distribution, "uniform");
        assert_eq!(s.param_a, 0.0);
        assert_eq!(s.param_b, 1.0);
    }

    #[test]
    fn stochastic_normal() {
        let s = StochasticSource::normal(100.0, 15.0);
        assert_eq!(s.distribution, "normal");
        assert_eq!(s.param_a, 100.0);
    }

    #[test]
    fn culture_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world
            .insert_component(e, CultureProfile::new("Norse", "no", "gregorian"))
            .unwrap();
        assert!(world.has_component::<CultureProfile>(e));
    }

    #[test]
    fn stochastic_as_component() {
        let mut world = crate::World::new();
        let e = world.spawn();
        world
            .insert_component(e, StochasticSource::poisson(5.0))
            .unwrap();
        let s = world.get_component::<StochasticSource>(e).unwrap();
        assert_eq!(s.param_a, 5.0);
    }
}
