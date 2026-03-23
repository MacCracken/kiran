//! TOML scene format, loading, entity spawning
//!
//! Defines the scene file format and provides helpers to load scenes from
//! TOML strings and spawn their entities into a [`World`](crate::World).

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::world::{Entity, KiranError, World};

// ---------------------------------------------------------------------------
// Scene definitions (serde + TOML)
// ---------------------------------------------------------------------------

/// A full scene file, loaded from TOML.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneDefinition {
    /// Display name of the scene.
    pub name: String,
    /// Optional description.
    #[serde(default)]
    pub description: String,
    /// Entities defined in this scene.
    #[serde(default)]
    pub entities: Vec<EntityDef>,
}

/// Definition of a single entity inside a scene file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityDef {
    /// Entity name (used as the [`Name`] component).
    pub name: String,
    /// Position in 3D space.
    #[serde(default)]
    pub position: [f32; 3],
    /// Optional light intensity (attaches a [`LightComponent`] if present).
    #[serde(default)]
    pub light_intensity: Option<f32>,
    /// Arbitrary string tags for gameplay logic.
    #[serde(default)]
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// ECS components used by scenes
// ---------------------------------------------------------------------------

/// 3D position component.
#[derive(Debug, Clone, PartialEq)]
pub struct Position(pub Vec3);

/// Name component — a human-readable label for an entity.
#[derive(Debug, Clone, PartialEq)]
pub struct Name(pub String);

/// Light component attached to entities with emissive properties.
#[derive(Debug, Clone, PartialEq)]
pub struct LightComponent {
    pub intensity: f32,
}

/// Tags component — arbitrary string labels.
#[derive(Debug, Clone, PartialEq)]
pub struct Tags(pub Vec<String>);

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Parse a TOML string into a [`SceneDefinition`].
pub fn load_scene(toml_str: &str) -> Result<SceneDefinition, KiranError> {
    toml::from_str(toml_str).map_err(|e| KiranError::Scene(e.to_string()))
}

/// Spawn all entities from a scene definition into a world.
///
/// Returns the list of spawned [`Entity`] handles.
pub fn spawn_scene(world: &mut World, scene: &SceneDefinition) -> Result<Vec<Entity>, KiranError> {
    let mut spawned = Vec::with_capacity(scene.entities.len());

    for def in &scene.entities {
        let entity = world.spawn();

        world.insert_component(entity, Name(def.name.clone()))?;
        world.insert_component(
            entity,
            Position(Vec3::new(def.position[0], def.position[1], def.position[2])),
        )?;

        if let Some(intensity) = def.light_intensity {
            world.insert_component(entity, LightComponent { intensity })?;
        }

        if !def.tags.is_empty() {
            world.insert_component(entity, Tags(def.tags.clone()))?;
        }

        spawned.push(entity);
    }

    Ok(spawned)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SCENE: &str = r#"
name = "Test Level"
description = "A simple test scene"

[[entities]]
name = "Player"
position = [1.0, 2.0, 3.0]
tags = ["controllable", "hero"]

[[entities]]
name = "Sun"
position = [0.0, 100.0, 0.0]
light_intensity = 1.5

[[entities]]
name = "Rock"
position = [5.0, 0.0, -2.0]
"#;

    #[test]
    fn load_scene_basic() {
        let scene = load_scene(SAMPLE_SCENE).unwrap();
        assert_eq!(scene.name, "Test Level");
        assert_eq!(scene.entities.len(), 3);
    }

    #[test]
    fn load_scene_entity_fields() {
        let scene = load_scene(SAMPLE_SCENE).unwrap();
        let player = &scene.entities[0];
        assert_eq!(player.name, "Player");
        assert_eq!(player.position, [1.0, 2.0, 3.0]);
        assert_eq!(player.tags, vec!["controllable", "hero"]);
        assert!(player.light_intensity.is_none());

        let sun = &scene.entities[1];
        assert_eq!(sun.light_intensity, Some(1.5));
    }

    #[test]
    fn load_scene_invalid_toml() {
        let result = load_scene("not valid toml {{{{");
        assert!(result.is_err());
    }

    #[test]
    fn load_scene_missing_name() {
        let result = load_scene(r#"description = "no name field""#);
        assert!(result.is_err());
    }

    #[test]
    fn spawn_scene_entities() {
        let scene = load_scene(SAMPLE_SCENE).unwrap();
        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();

        assert_eq!(entities.len(), 3);
        assert_eq!(world.entity_count(), 3);
    }

    #[test]
    fn spawn_scene_components() {
        let scene = load_scene(SAMPLE_SCENE).unwrap();
        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();

        // Player
        let name = world.get_component::<Name>(entities[0]).unwrap();
        assert_eq!(name.0, "Player");

        let pos = world.get_component::<Position>(entities[0]).unwrap();
        assert_eq!(pos.0, Vec3::new(1.0, 2.0, 3.0));

        let tags = world.get_component::<Tags>(entities[0]).unwrap();
        assert_eq!(tags.0.len(), 2);

        // Sun has light
        let light = world.get_component::<LightComponent>(entities[1]).unwrap();
        assert_eq!(light.intensity, 1.5);

        // Rock has no light
        assert!(world.get_component::<LightComponent>(entities[2]).is_none());
    }

    #[test]
    fn spawn_empty_scene() {
        let scene = load_scene(r#"name = "Empty""#).unwrap();
        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();
        assert!(entities.is_empty());
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn scene_roundtrip_toml() {
        let scene = SceneDefinition {
            name: "Roundtrip".into(),
            description: String::new(),
            entities: vec![EntityDef {
                name: "A".into(),
                position: [1.0, 2.0, 3.0],
                light_intensity: None,
                tags: vec![],
            }],
        };
        let serialized = toml::to_string(&scene).unwrap();
        let deserialized = load_scene(&serialized).unwrap();
        assert_eq!(deserialized.name, "Roundtrip");
        assert_eq!(deserialized.entities.len(), 1);
    }

    #[test]
    fn spawn_scene_tags_absent() {
        let toml_str = r#"
name = "NoTags"
[[entities]]
name = "Plain"
position = [0.0, 0.0, 0.0]
"#;
        let scene = load_scene(toml_str).unwrap();
        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();
        assert!(world.get_component::<Tags>(entities[0]).is_none());
    }

    #[test]
    fn unicode_entity_names() {
        let toml_str = r#"
name = "ユニコード"
[[entities]]
name = "主人公"
position = [0.0, 0.0, 0.0]
tags = ["プレイヤー"]
"#;
        let scene = load_scene(toml_str).unwrap();
        assert_eq!(scene.name, "ユニコード");
        assert_eq!(scene.entities[0].name, "主人公");

        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();
        let name = world.get_component::<Name>(entities[0]).unwrap();
        assert_eq!(name.0, "主人公");
    }

    #[test]
    fn scene_default_position() {
        let toml_str = r#"
name = "Defaults"
[[entities]]
name = "Origin"
"#;
        let scene = load_scene(toml_str).unwrap();
        assert_eq!(scene.entities[0].position, [0.0, 0.0, 0.0]);
    }
}
