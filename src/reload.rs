//! Scene hot reload
//!
//! Watches scene TOML files for changes and applies updates to the world
//! without restarting. Uses polling-based change detection.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::scene::{
    LightComponent, Material, Name, Position, SceneDefinition, Tags, load_scene, spawn_scene,
};
use crate::world::{KiranError, World};

// ---------------------------------------------------------------------------
// File watcher
// ---------------------------------------------------------------------------

/// Tracks file modification times for change detection.
#[derive(Debug, Default)]
pub struct FileWatcher {
    /// Path → last known modification time.
    watched: HashMap<PathBuf, SystemTime>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start watching a file. Returns Err if the file doesn't exist.
    pub fn watch(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref().to_path_buf();
        let modified = std::fs::metadata(&path)?.modified()?;
        self.watched.insert(path, modified);
        Ok(())
    }

    /// Stop watching a file.
    pub fn unwatch(&mut self, path: impl AsRef<Path>) {
        self.watched.remove(path.as_ref());
    }

    /// Check all watched files for changes. Returns paths that were modified.
    pub fn poll_changes(&mut self) -> Vec<PathBuf> {
        let mut changed = Vec::new();
        for (path, last_modified) in &mut self.watched {
            if let Ok(meta) = std::fs::metadata(path)
                && let Ok(modified) = meta.modified()
                && modified > *last_modified
            {
                *last_modified = modified;
                changed.push(path.clone());
            }
        }
        changed
    }

    /// Number of watched files.
    pub fn watch_count(&self) -> usize {
        self.watched.len()
    }
}

// ---------------------------------------------------------------------------
// Scene reload
// ---------------------------------------------------------------------------

/// Hot reload resource — manages scene file watching and live updates.
pub struct SceneReloader {
    watcher: FileWatcher,
    /// Maps file path → the entities spawned from it (for despawn on reload).
    scene_entities: HashMap<PathBuf, Vec<crate::world::Entity>>,
}

impl Default for SceneReloader {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneReloader {
    pub fn new() -> Self {
        Self {
            watcher: FileWatcher::new(),
            scene_entities: HashMap::new(),
        }
    }

    /// Load a scene and start watching it for changes.
    pub fn load_and_watch(
        &mut self,
        path: impl AsRef<Path>,
        world: &mut World,
    ) -> Result<Vec<crate::world::Entity>, KiranError> {
        let path = path.as_ref().to_path_buf();
        let toml_str = std::fs::read_to_string(&path)
            .map_err(|e| KiranError::Scene(format!("failed to read {}: {}", path.display(), e)))?;
        let scene = load_scene(&toml_str)?;
        let entities = spawn_scene(world, &scene)?;

        self.scene_entities.insert(path.clone(), entities.clone());

        // Ignore watch errors (e.g. file deleted between read and watch)
        let _ = self.watcher.watch(&path);

        Ok(entities)
    }

    /// Poll for changed scene files and reload them.
    /// Returns the number of scenes reloaded.
    pub fn poll_and_reload(&mut self, world: &mut World) -> usize {
        let changed = self.watcher.poll_changes();
        let mut reloaded = 0;

        for path in changed {
            if let Ok(toml_str) = std::fs::read_to_string(&path)
                && let Ok(scene) = load_scene(&toml_str)
            {
                // Despawn old entities
                if let Some(old_entities) = self.scene_entities.remove(&path) {
                    for entity in old_entities {
                        let _ = world.despawn(entity);
                    }
                }

                // Spawn new entities
                if let Ok(new_entities) = spawn_scene(world, &scene) {
                    self.scene_entities.insert(path, new_entities);
                    reloaded += 1;
                }
            }
        }

        reloaded
    }

    /// Get the file watcher.
    pub fn watcher(&self) -> &FileWatcher {
        &self.watcher
    }

    /// Check if a scene file is being watched.
    pub fn is_watching(&self, path: impl AsRef<Path>) -> bool {
        self.scene_entities.contains_key(path.as_ref())
    }
}

/// Apply a scene definition to the world, updating existing entities where possible.
/// Entities matched by name are updated in place; new entities are spawned; removed entities are despawned.
pub fn apply_scene_diff(
    world: &mut World,
    existing: &[crate::world::Entity],
    new_scene: &SceneDefinition,
) -> Result<Vec<crate::world::Entity>, KiranError> {
    use glam::Vec3;

    let mut result = Vec::new();

    // Build name → entity map from existing
    let mut name_map: HashMap<String, crate::world::Entity> = HashMap::new();
    for &entity in existing {
        if let Some(name) = world.get_component::<Name>(entity) {
            name_map.insert(name.0.clone(), entity);
        }
    }

    // Process new scene entities
    let mut seen_names = std::collections::HashSet::new();
    for def in &new_scene.entities {
        seen_names.insert(def.name.clone());

        if let Some(&entity) = name_map.get(&def.name) {
            // Update existing entity in place
            let new_pos = Position(Vec3::new(def.position[0], def.position[1], def.position[2]));
            if let Some(pos) = world.get_component_mut::<Position>(entity) {
                *pos = new_pos;
            }

            if let Some(intensity) = def.light_intensity {
                world.insert_component(entity, LightComponent { intensity })?;
            } else {
                world.remove_component::<LightComponent>(entity);
            }

            if !def.tags.is_empty() {
                world.insert_component(entity, Tags(def.tags.clone()))?;
            } else {
                world.remove_component::<Tags>(entity);
            }

            if let Some(m) = &def.material {
                world.insert_component(entity, m.clone())?;
            } else {
                world.remove_component::<Material>(entity);
            }

            result.push(entity);
        } else {
            // New entity — spawn it
            let entities = spawn_scene(
                world,
                &SceneDefinition {
                    name: new_scene.name.clone(),
                    description: String::new(),
                    prefabs: new_scene.prefabs.clone(),
                    entities: vec![def.clone()],
                },
            )?;
            result.extend(entities);
        }
    }

    // Despawn entities that no longer exist in the scene
    for (name, entity) in &name_map {
        if !seen_names.contains(name) {
            let _ = world.despawn(*entity);
        }
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn file_watcher_basics() {
        let mut watcher = FileWatcher::new();
        assert_eq!(watcher.watch_count(), 0);

        // Can't watch nonexistent file
        assert!(watcher.watch("/nonexistent/file.toml").is_err());
        assert_eq!(watcher.watch_count(), 0);
    }

    #[test]
    fn file_watcher_poll_no_changes() {
        let mut watcher = FileWatcher::new();
        // No files watched, no changes
        let changes = watcher.poll_changes();
        assert!(changes.is_empty());
    }

    #[test]
    fn scene_reloader_new() {
        let reloader = SceneReloader::new();
        assert_eq!(reloader.watcher().watch_count(), 0);
    }

    #[test]
    fn apply_scene_diff_updates_position() {
        let mut world = World::new();

        // Create initial scene
        let initial = load_scene(
            r#"
name = "Diff Test"
[[entities]]
name = "Player"
position = [0.0, 0.0, 0.0]
"#,
        )
        .unwrap();
        let entities = spawn_scene(&mut world, &initial).unwrap();

        // Apply diff with updated position
        let updated = load_scene(
            r#"
name = "Diff Test"
[[entities]]
name = "Player"
position = [10.0, 5.0, 3.0]
"#,
        )
        .unwrap();
        let result = apply_scene_diff(&mut world, &entities, &updated).unwrap();

        assert_eq!(result.len(), 1);
        let pos = world.get_component::<Position>(result[0]).unwrap();
        assert_eq!(pos.0, Vec3::new(10.0, 5.0, 3.0));
    }

    #[test]
    fn apply_scene_diff_adds_entity() {
        let mut world = World::new();

        let initial = load_scene(
            r#"
name = "Add Test"
[[entities]]
name = "A"
position = [0.0, 0.0, 0.0]
"#,
        )
        .unwrap();
        let entities = spawn_scene(&mut world, &initial).unwrap();

        let updated = load_scene(
            r#"
name = "Add Test"
[[entities]]
name = "A"
position = [0.0, 0.0, 0.0]
[[entities]]
name = "B"
position = [5.0, 0.0, 0.0]
"#,
        )
        .unwrap();
        let result = apply_scene_diff(&mut world, &entities, &updated).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn apply_scene_diff_removes_entity() {
        let mut world = World::new();

        let initial = load_scene(
            r#"
name = "Remove Test"
[[entities]]
name = "Keep"
[[entities]]
name = "Remove"
"#,
        )
        .unwrap();
        let entities = spawn_scene(&mut world, &initial).unwrap();
        assert_eq!(world.entity_count(), 2);

        let updated = load_scene(
            r#"
name = "Remove Test"
[[entities]]
name = "Keep"
"#,
        )
        .unwrap();
        let result = apply_scene_diff(&mut world, &entities, &updated).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(world.entity_count(), 1);
        let name = world.get_component::<Name>(result[0]).unwrap();
        assert_eq!(name.0, "Keep");
    }

    #[test]
    fn apply_scene_diff_updates_components() {
        let mut world = World::new();

        let initial = load_scene(
            r#"
name = "Component Update"
[[entities]]
name = "Light"
light_intensity = 1.0
tags = ["bright"]
"#,
        )
        .unwrap();
        let entities = spawn_scene(&mut world, &initial).unwrap();

        // Update: change light, remove tags
        let updated = load_scene(
            r#"
name = "Component Update"
[[entities]]
name = "Light"
light_intensity = 0.5
"#,
        )
        .unwrap();
        let result = apply_scene_diff(&mut world, &entities, &updated).unwrap();

        let light = world.get_component::<LightComponent>(result[0]).unwrap();
        assert_eq!(light.intensity, 0.5);
        assert!(world.get_component::<Tags>(result[0]).is_none());
    }

    #[test]
    fn apply_scene_diff_preserves_entity_identity() {
        let mut world = World::new();

        let initial = load_scene(
            r#"
name = "Identity"
[[entities]]
name = "Stable"
position = [1.0, 0.0, 0.0]
"#,
        )
        .unwrap();
        let entities = spawn_scene(&mut world, &initial).unwrap();
        let original_entity = entities[0];

        let updated = load_scene(
            r#"
name = "Identity"
[[entities]]
name = "Stable"
position = [99.0, 0.0, 0.0]
"#,
        )
        .unwrap();
        let result = apply_scene_diff(&mut world, &entities, &updated).unwrap();

        // Same entity handle — updated in place, not respawned
        assert_eq!(result[0], original_entity);
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn file_watcher_unwatch() {
        let mut watcher = FileWatcher::new();
        // Watch a real file
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        watcher.watch(&path).unwrap();
        assert_eq!(watcher.watch_count(), 1);

        watcher.unwatch(&path);
        assert_eq!(watcher.watch_count(), 0);
    }

    #[test]
    fn file_watcher_watch_real_file() {
        let mut watcher = FileWatcher::new();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        watcher.watch(&path).unwrap();
        assert_eq!(watcher.watch_count(), 1);

        // No changes since we just started watching
        let changes = watcher.poll_changes();
        assert!(changes.is_empty());
    }

    #[test]
    fn scene_reloader_load_nonexistent() {
        let mut reloader = SceneReloader::new();
        let mut world = World::new();
        let result = reloader.load_and_watch("/nonexistent/scene.toml", &mut world);
        assert!(result.is_err());
    }

    #[test]
    fn apply_scene_diff_no_op() {
        let mut world = World::new();
        let scene = load_scene(
            r#"
name = "NoOp"
[[entities]]
name = "Stable"
position = [1.0, 2.0, 3.0]
"#,
        )
        .unwrap();
        let entities = spawn_scene(&mut world, &scene).unwrap();

        // Apply same scene — no changes
        let result = apply_scene_diff(&mut world, &entities, &scene).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], entities[0]);
        assert_eq!(world.entity_count(), 1);

        let pos = world.get_component::<Position>(result[0]).unwrap();
        assert_eq!(pos.0, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn apply_scene_diff_empty_to_populated() {
        let mut world = World::new();
        let existing: Vec<crate::world::Entity> = vec![];

        let new_scene = load_scene(
            r#"
name = "Populated"
[[entities]]
name = "New1"
[[entities]]
name = "New2"
"#,
        )
        .unwrap();
        let result = apply_scene_diff(&mut world, &existing, &new_scene).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn apply_scene_diff_populated_to_empty() {
        let mut world = World::new();
        let scene = load_scene(
            r#"
name = "Clear"
[[entities]]
name = "Gone1"
[[entities]]
name = "Gone2"
"#,
        )
        .unwrap();
        let entities = spawn_scene(&mut world, &scene).unwrap();
        assert_eq!(world.entity_count(), 2);

        let empty_scene = load_scene(r#"name = "Clear""#).unwrap();
        let result = apply_scene_diff(&mut world, &entities, &empty_scene).unwrap();

        assert!(result.is_empty());
        assert_eq!(world.entity_count(), 0);
    }
}
