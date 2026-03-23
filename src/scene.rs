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
    /// Reusable entity templates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prefabs: Vec<PrefabDef>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light_intensity: Option<f32>,
    /// Arbitrary string tags for gameplay logic.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Optional material definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
    /// Child entities (scene hierarchy).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<EntityDef>,
    /// Optional prefab template name to inherit from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefab: Option<String>,
    /// Optional sound source definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<SoundDef>,
    /// Optional physics body definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physics: Option<PhysicsDef>,
}

/// Physics body definition in a scene file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsDef {
    /// Body type: "dynamic", "static", or "kinematic".
    pub body_type: String,
    /// Collider shape and dimensions.
    pub collider: ColliderDef,
}

/// Collider definition in a scene file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColliderDef {
    /// Shape type: "ball", "box", or "capsule".
    pub shape: String,
    /// Radius for ball/capsule shapes.
    #[serde(default)]
    pub radius: Option<f64>,
    /// Half-extents for box shapes [hx, hy, hz].
    #[serde(default)]
    pub half_extents: Option<[f64; 3]>,
    /// Half-height for capsule shapes.
    #[serde(default)]
    pub half_height: Option<f64>,
}

/// Sound source definition in a scene file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoundDef {
    /// Path to the audio file.
    pub source: String,
    /// Playback volume (0.0–1.0).
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Whether the sound is spatial.
    #[serde(default = "default_true")]
    pub spatial: bool,
    /// Whether the sound loops.
    #[serde(default)]
    pub looping: bool,
}

fn default_volume() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

/// A reusable entity template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrefabDef {
    /// Template name (referenced by EntityDef::prefab).
    pub name: String,
    /// Default position.
    #[serde(default)]
    pub position: [f32; 3],
    /// Default light intensity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub light_intensity: Option<f32>,
    /// Default tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Default material.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<Material>,
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

/// Parent component — points to this entity's parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

/// Children component — ordered list of child entities.
#[derive(Debug, Clone, PartialEq)]
pub struct Children(pub Vec<Entity>);

/// Material definition attached to an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    /// Base color as RGBA (0.0–1.0).
    #[serde(default = "Material::default_color")]
    pub color: [f32; 4],
    /// Optional texture file path.
    #[serde(default)]
    pub texture: Option<String>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            texture: None,
        }
    }
}

impl Material {
    fn default_color() -> [f32; 4] {
        [1.0, 1.0, 1.0, 1.0]
    }
}

// ---------------------------------------------------------------------------
// Hierarchy helpers
// ---------------------------------------------------------------------------

/// Set `child`'s parent to `parent`, updating both Parent and Children components.
pub fn set_parent(world: &mut World, child: Entity, parent: Entity) -> Result<(), KiranError> {
    // Remove from old parent's children list
    if let Some(old_parent) = world.get_component::<Parent>(child).map(|p| p.0)
        && let Some(children) = world.get_component_mut::<Children>(old_parent)
    {
        children.0.retain(|&e| e != child);
    }

    // Set new parent
    world.insert_component(child, Parent(parent))?;

    // Add to new parent's children list
    if let Some(children) = world.get_component_mut::<Children>(parent) {
        if !children.0.contains(&child) {
            children.0.push(child);
        }
    } else {
        world.insert_component(parent, Children(vec![child]))?;
    }

    Ok(())
}

/// Remove `child` from its parent, clearing the Parent component.
pub fn remove_parent(world: &mut World, child: Entity) {
    if let Some(parent) = world.get_component::<Parent>(child).map(|p| p.0)
        && let Some(children) = world.get_component_mut::<Children>(parent)
    {
        children.0.retain(|&e| e != child);
    }
    world.remove_component::<Parent>(child);
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Parse a TOML string into a [`SceneDefinition`].
pub fn load_scene(toml_str: &str) -> Result<SceneDefinition, KiranError> {
    toml::from_str(toml_str).map_err(|e| KiranError::Scene(e.to_string()))
}

/// Spawn all entities from a scene definition into a world.
///
/// Returns the list of top-level spawned [`Entity`] handles.
/// Child entities are spawned recursively with Parent/Children components.
pub fn spawn_scene(world: &mut World, scene: &SceneDefinition) -> Result<Vec<Entity>, KiranError> {
    let mut spawned = Vec::with_capacity(scene.entities.len());

    for def in &scene.entities {
        let entity = spawn_entity_def(world, def, &scene.prefabs, None)?;
        spawned.push(entity);
    }

    Ok(spawned)
}

/// Spawn a single entity definition, resolving prefabs and recursing into children.
fn spawn_entity_def(
    world: &mut World,
    def: &EntityDef,
    prefabs: &[PrefabDef],
    parent: Option<Entity>,
) -> Result<Entity, KiranError> {
    let entity = world.spawn();

    // Resolve prefab defaults
    let prefab = def
        .prefab
        .as_ref()
        .and_then(|name| prefabs.iter().find(|p| &p.name == name));

    // Name — always from the entity def
    world.insert_component(entity, Name(def.name.clone()))?;

    // Position — entity overrides prefab
    let pos = if def.position != [0.0, 0.0, 0.0] {
        def.position
    } else {
        prefab.map_or([0.0, 0.0, 0.0], |p| p.position)
    };
    world.insert_component(entity, Position(Vec3::new(pos[0], pos[1], pos[2])))?;

    // Light — entity overrides prefab
    let light = def
        .light_intensity
        .or(prefab.and_then(|p| p.light_intensity));
    if let Some(intensity) = light {
        world.insert_component(entity, LightComponent { intensity })?;
    }

    // Tags — merge entity + prefab
    let mut tags = def.tags.clone();
    if let Some(p) = prefab {
        for tag in &p.tags {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }
    }
    if !tags.is_empty() {
        world.insert_component(entity, Tags(tags))?;
    }

    // Material — entity overrides prefab
    let mat = def
        .material
        .as_ref()
        .or(prefab.and_then(|p| p.material.as_ref()));
    if let Some(m) = mat {
        world.insert_component(entity, m.clone())?;
    }

    // Sound source
    if let Some(sound) = &def.sound {
        #[cfg(feature = "audio")]
        {
            world.insert_component(
                entity,
                crate::audio::SoundSource {
                    source: sound.source.clone(),
                    volume: sound.volume,
                    spatial: sound.spatial,
                    looping: sound.looping,
                    playing: false,
                    max_distance: 50.0,
                },
            )?;
        }
        let _ = sound; // suppress unused warning when audio feature is off
    }

    // Parent-child hierarchy
    if let Some(parent_entity) = parent {
        set_parent(world, entity, parent_entity)?;
    }

    // Recurse into children
    for child_def in &def.children {
        spawn_entity_def(world, child_def, prefabs, Some(entity))?;
    }

    Ok(entity)
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
            prefabs: vec![],
            entities: vec![EntityDef {
                name: "A".into(),
                position: [1.0, 2.0, 3.0],
                light_intensity: None,
                tags: vec![],
                material: None,
                children: vec![],
                prefab: None,
                sound: None,
                physics: None,
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

    // -- Hierarchy tests --

    #[test]
    fn set_parent_creates_relationship() {
        let mut world = World::new();
        let parent = world.spawn();
        let child = world.spawn();

        set_parent(&mut world, child, parent).unwrap();

        assert_eq!(world.get_component::<Parent>(child).unwrap().0, parent);
        let children = world.get_component::<Children>(parent).unwrap();
        assert_eq!(children.0, vec![child]);
    }

    #[test]
    fn set_parent_reparent() {
        let mut world = World::new();
        let p1 = world.spawn();
        let p2 = world.spawn();
        let child = world.spawn();

        set_parent(&mut world, child, p1).unwrap();
        set_parent(&mut world, child, p2).unwrap();

        // Old parent has no children
        let c1 = world.get_component::<Children>(p1).unwrap();
        assert!(c1.0.is_empty());

        // New parent has the child
        let c2 = world.get_component::<Children>(p2).unwrap();
        assert_eq!(c2.0, vec![child]);

        assert_eq!(world.get_component::<Parent>(child).unwrap().0, p2);
    }

    #[test]
    fn remove_parent_clears() {
        let mut world = World::new();
        let parent = world.spawn();
        let child = world.spawn();

        set_parent(&mut world, child, parent).unwrap();
        remove_parent(&mut world, child);

        assert!(world.get_component::<Parent>(child).is_none());
        let children = world.get_component::<Children>(parent).unwrap();
        assert!(children.0.is_empty());
    }

    #[test]
    fn multiple_children() {
        let mut world = World::new();
        let parent = world.spawn();
        let c1 = world.spawn();
        let c2 = world.spawn();
        let c3 = world.spawn();

        set_parent(&mut world, c1, parent).unwrap();
        set_parent(&mut world, c2, parent).unwrap();
        set_parent(&mut world, c3, parent).unwrap();

        let children = world.get_component::<Children>(parent).unwrap();
        assert_eq!(children.0.len(), 3);
    }

    // -- Material tests --

    #[test]
    fn scene_with_material() {
        let toml_str = r#"
name = "Material Test"
[[entities]]
name = "Cube"
position = [0.0, 0.0, 0.0]
[entities.material]
color = [1.0, 0.0, 0.0, 1.0]
texture = "textures/brick.png"
"#;
        let scene = load_scene(toml_str).unwrap();
        let mat = scene.entities[0].material.as_ref().unwrap();
        assert_eq!(mat.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(mat.texture.as_deref(), Some("textures/brick.png"));

        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();
        let spawned_mat = world.get_component::<Material>(entities[0]).unwrap();
        assert_eq!(spawned_mat.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn material_default() {
        let mat = Material::default();
        assert_eq!(mat.color, [1.0, 1.0, 1.0, 1.0]);
        assert!(mat.texture.is_none());
    }

    // -- Hierarchy in TOML --

    #[test]
    fn scene_with_children() {
        let toml_str = r#"
name = "Hierarchy"
[[entities]]
name = "Parent"
position = [0.0, 0.0, 0.0]
[[entities.children]]
name = "Child1"
position = [1.0, 0.0, 0.0]
[[entities.children]]
name = "Child2"
position = [2.0, 0.0, 0.0]
"#;
        let scene = load_scene(toml_str).unwrap();
        assert_eq!(scene.entities[0].children.len(), 2);

        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();

        // Only top-level entity returned
        assert_eq!(entities.len(), 1);
        // But 3 entities total (parent + 2 children)
        assert_eq!(world.entity_count(), 3);

        // Verify hierarchy
        let parent = entities[0];
        let children = world.get_component::<Children>(parent).unwrap();
        assert_eq!(children.0.len(), 2);

        let child1 = children.0[0];
        assert_eq!(world.get_component::<Parent>(child1).unwrap().0, parent);
        assert_eq!(world.get_component::<Name>(child1).unwrap().0, "Child1");
    }

    // -- Prefab tests --

    #[test]
    fn prefab_template() {
        let toml_str = r#"
name = "Prefab Test"

[[prefabs]]
name = "tree"
light_intensity = 0.1
tags = ["vegetation", "static"]
[prefabs.material]
color = [0.2, 0.8, 0.1, 1.0]

[[entities]]
name = "Oak"
position = [5.0, 0.0, 3.0]
prefab = "tree"
tags = ["large"]

[[entities]]
name = "Pine"
position = [10.0, 0.0, 7.0]
prefab = "tree"
"#;
        let scene = load_scene(toml_str).unwrap();
        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();

        // Oak inherits light from prefab
        let light = world.get_component::<LightComponent>(entities[0]).unwrap();
        assert_eq!(light.intensity, 0.1);

        // Oak has merged tags (own + prefab)
        let tags = world.get_component::<Tags>(entities[0]).unwrap();
        assert!(tags.0.contains(&"large".to_string()));
        assert!(tags.0.contains(&"vegetation".to_string()));

        // Oak inherits material from prefab
        let mat = world.get_component::<Material>(entities[0]).unwrap();
        assert_eq!(mat.color[1], 0.8);

        // Pine also gets prefab defaults
        let pine_light = world.get_component::<LightComponent>(entities[1]).unwrap();
        assert_eq!(pine_light.intensity, 0.1);
    }

    #[test]
    fn prefab_entity_overrides() {
        let toml_str = r#"
name = "Override"
[[prefabs]]
name = "base"
light_intensity = 1.0

[[entities]]
name = "Custom"
light_intensity = 5.0
prefab = "base"
"#;
        let scene = load_scene(toml_str).unwrap();
        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();

        // Entity's own value overrides prefab
        let light = world.get_component::<LightComponent>(entities[0]).unwrap();
        assert_eq!(light.intensity, 5.0);
    }

    #[test]
    fn unknown_prefab_ignored() {
        let toml_str = r#"
name = "Missing Prefab"
[[entities]]
name = "Orphan"
prefab = "nonexistent"
"#;
        let scene = load_scene(toml_str).unwrap();
        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();
        assert_eq!(entities.len(), 1);
        // No crash, just no prefab applied
        assert!(world.get_component::<LightComponent>(entities[0]).is_none());
    }

    #[test]
    fn prefab_with_children() {
        let toml_str = r#"
name = "Prefab + Hierarchy"
[[prefabs]]
name = "lamp"
light_intensity = 2.0
tags = ["light-source"]

[[entities]]
name = "StreetLamp"
position = [10.0, 0.0, 0.0]
prefab = "lamp"
[[entities.children]]
name = "LampBulb"
position = [0.0, 3.0, 0.0]
prefab = "lamp"
"#;
        let scene = load_scene(toml_str).unwrap();
        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();

        assert_eq!(entities.len(), 1); // only top-level
        assert_eq!(world.entity_count(), 2); // parent + child

        // Parent has prefab light
        let parent = entities[0];
        let light = world.get_component::<LightComponent>(parent).unwrap();
        assert_eq!(light.intensity, 2.0);

        // Child also has prefab light
        let children = world.get_component::<Children>(parent).unwrap();
        let child = children.0[0];
        let child_light = world.get_component::<LightComponent>(child).unwrap();
        assert_eq!(child_light.intensity, 2.0);

        // Child has parent set
        assert_eq!(world.get_component::<Parent>(child).unwrap().0, parent);
    }

    #[test]
    fn deep_hierarchy() {
        let toml_str = r#"
name = "Deep"
[[entities]]
name = "Root"
[[entities.children]]
name = "L1"
[[entities.children.children]]
name = "L2"
"#;
        let scene = load_scene(toml_str).unwrap();
        let mut world = World::new();
        let entities = spawn_scene(&mut world, &scene).unwrap();

        assert_eq!(entities.len(), 1);
        assert_eq!(world.entity_count(), 3);

        let root = entities[0];
        let l1 = world.get_component::<Children>(root).unwrap().0[0];
        let l2 = world.get_component::<Children>(l1).unwrap().0[0];
        assert_eq!(world.get_component::<Name>(l2).unwrap().0, "L2");
        assert_eq!(world.get_component::<Parent>(l2).unwrap().0, l1);
    }

    #[test]
    fn serialization_skips_empty_fields() {
        let scene = SceneDefinition {
            name: "Clean".into(),
            description: String::new(),
            prefabs: vec![],
            entities: vec![EntityDef {
                name: "Simple".into(),
                position: [0.0, 0.0, 0.0],
                light_intensity: None,
                tags: vec![],
                material: None,
                children: vec![],
                prefab: None,
                sound: None,
                physics: None,
            }],
        };
        let toml_str = toml::to_string(&scene).unwrap();
        // Empty optional fields should not appear
        assert!(!toml_str.contains("light_intensity"));
        assert!(!toml_str.contains("material"));
        assert!(!toml_str.contains("prefab"));
        assert!(!toml_str.contains("children"));
        assert!(!toml_str.contains("prefabs"));
        assert!(!toml_str.contains("sound"));
    }

    #[test]
    fn scene_with_sound() {
        let toml_str = r#"
name = "Sound Test"
[[entities]]
name = "Radio"
position = [5.0, 1.0, 0.0]
[entities.sound]
source = "sounds/music.ogg"
volume = 0.7
spatial = true
looping = true
"#;
        let scene = load_scene(toml_str).unwrap();
        let sound = scene.entities[0].sound.as_ref().unwrap();
        assert_eq!(sound.source, "sounds/music.ogg");
        assert_eq!(sound.volume, 0.7);
        assert!(sound.spatial);
        assert!(sound.looping);
    }

    #[test]
    fn sound_def_defaults() {
        let toml_str = r#"
name = "Defaults"
[[entities]]
name = "Beep"
[entities.sound]
source = "beep.wav"
"#;
        let scene = load_scene(toml_str).unwrap();
        let sound = scene.entities[0].sound.as_ref().unwrap();
        assert_eq!(sound.volume, 1.0);
        assert!(sound.spatial);
        assert!(!sound.looping);
    }

    // -- Physics in TOML tests --

    #[test]
    fn scene_with_physics() {
        let toml_str = r#"
name = "Physics Test"
[[entities]]
name = "Ball"
position = [0.0, 10.0, 0.0]
[entities.physics]
body_type = "dynamic"
[entities.physics.collider]
shape = "ball"
radius = 0.5

[[entities]]
name = "Floor"
position = [0.0, 0.0, 0.0]
[entities.physics]
body_type = "static"
[entities.physics.collider]
shape = "box"
half_extents = [50.0, 0.5, 50.0]
"#;
        let scene = load_scene(toml_str).unwrap();
        assert_eq!(scene.entities.len(), 2);

        let ball_phys = scene.entities[0].physics.as_ref().unwrap();
        assert_eq!(ball_phys.body_type, "dynamic");
        assert_eq!(ball_phys.collider.shape, "ball");
        assert_eq!(ball_phys.collider.radius, Some(0.5));

        let floor_phys = scene.entities[1].physics.as_ref().unwrap();
        assert_eq!(floor_phys.body_type, "static");
        assert_eq!(floor_phys.collider.shape, "box");
        assert_eq!(floor_phys.collider.half_extents, Some([50.0, 0.5, 50.0]));
    }

    #[test]
    fn scene_physics_capsule() {
        let toml_str = r#"
name = "Capsule"
[[entities]]
name = "Character"
[entities.physics]
body_type = "kinematic"
[entities.physics.collider]
shape = "capsule"
half_height = 0.8
radius = 0.3
"#;
        let scene = load_scene(toml_str).unwrap();
        let phys = scene.entities[0].physics.as_ref().unwrap();
        assert_eq!(phys.body_type, "kinematic");
        assert_eq!(phys.collider.half_height, Some(0.8));
        assert_eq!(phys.collider.radius, Some(0.3));
    }
}
