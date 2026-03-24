//! Asset pipeline — registry, handles, and hot reload integration.
//!
//! Provides a typed asset registry that maps file paths to handles,
//! integrates with [`FileWatcher`](crate::reload::FileWatcher) for hot reload.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::reload::FileWatcher;

/// A typed handle to a loaded asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetHandle(pub u64);

impl AssetHandle {
    /// A null/invalid handle.
    pub const NONE: Self = Self(0);

    /// Check if this handle is valid (non-zero).
    pub fn is_valid(self) -> bool {
        self.0 != 0
    }
}

/// Metadata about a loaded asset.
#[derive(Debug, Clone)]
pub struct AssetEntry {
    pub handle: AssetHandle,
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub size_bytes: usize,
}

/// Known asset types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Sound,
    Scene,
    Shader,
    Model,
    Script,
    Other,
}

impl AssetType {
    /// Infer asset type from file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "tga" => Self::Texture,
            "wav" | "ogg" | "mp3" | "flac" => Self::Sound,
            "toml" => Self::Scene,
            "wgsl" | "glsl" | "hlsl" => Self::Shader,
            "gltf" | "glb" | "obj" | "fbx" => Self::Model,
            "wasm" => Self::Script,
            _ => Self::Other,
        }
    }
}

/// Asset registry — maps paths to typed handles with hot reload support.
pub struct AssetRegistry {
    /// Path → handle mapping.
    path_to_handle: HashMap<PathBuf, AssetHandle>,
    /// Handle → entry mapping.
    entries: HashMap<AssetHandle, AssetEntry>,
    /// Next handle ID.
    next_id: u64,
    /// File watcher for hot reload.
    watcher: FileWatcher,
    /// Handles that need reloading (changed on disk).
    dirty: Vec<AssetHandle>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            path_to_handle: HashMap::new(),
            entries: HashMap::new(),
            next_id: 1, // 0 is NONE
            watcher: FileWatcher::new(),
            dirty: Vec::new(),
        }
    }

    /// Register an asset and start watching it for changes.
    pub fn register(&mut self, path: impl AsRef<Path>, asset_type: AssetType) -> AssetHandle {
        let path = path.as_ref().to_path_buf();

        // Return existing handle if already registered
        if let Some(&handle) = self.path_to_handle.get(&path) {
            return handle;
        }

        let handle = AssetHandle(self.next_id);
        self.next_id += 1;

        let size_bytes = std::fs::metadata(&path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);

        self.entries.insert(
            handle,
            AssetEntry {
                handle,
                path: path.clone(),
                asset_type,
                size_bytes,
            },
        );
        self.path_to_handle.insert(path.clone(), handle);

        // Start watching (ignore errors for missing files)
        let _ = self.watcher.watch(&path);

        handle
    }

    /// Register an asset, inferring type from file extension.
    pub fn register_auto(&mut self, path: impl AsRef<Path>) -> AssetHandle {
        let path = path.as_ref();
        let asset_type = path
            .extension()
            .and_then(|e| e.to_str())
            .map(AssetType::from_extension)
            .unwrap_or(AssetType::Other);
        self.register(path, asset_type)
    }

    /// Get the handle for a path.
    pub fn handle_for(&self, path: impl AsRef<Path>) -> Option<AssetHandle> {
        self.path_to_handle.get(path.as_ref()).copied()
    }

    /// Get the entry for a handle.
    pub fn get(&self, handle: AssetHandle) -> Option<&AssetEntry> {
        self.entries.get(&handle)
    }

    /// Get the path for a handle.
    pub fn path_for(&self, handle: AssetHandle) -> Option<&Path> {
        self.entries.get(&handle).map(|e| e.path.as_path())
    }

    /// Number of registered assets.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Poll for changed assets. Returns handles that need reloading.
    pub fn poll_changes(&mut self) -> Vec<AssetHandle> {
        let changed_paths = self.watcher.poll_changes();
        self.dirty.clear();
        for path in changed_paths {
            if let Some(&handle) = self.path_to_handle.get(&path) {
                self.dirty.push(handle);
            }
        }
        self.dirty.clone()
    }

    /// Check if a handle needs reloading.
    pub fn is_dirty(&self, handle: AssetHandle) -> bool {
        self.dirty.contains(&handle)
    }

    /// Total size of all registered assets in bytes.
    pub fn total_size_bytes(&self) -> usize {
        self.entries.values().map(|e| e.size_bytes).sum()
    }

    /// List all registered assets of a given type.
    pub fn assets_of_type(&self, asset_type: AssetType) -> Vec<AssetHandle> {
        self.entries
            .values()
            .filter(|e| e.asset_type == asset_type)
            .map(|e| e.handle)
            .collect()
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_handle_none() {
        assert!(!AssetHandle::NONE.is_valid());
        assert!(AssetHandle(1).is_valid());
    }

    #[test]
    fn asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("png"), AssetType::Texture);
        assert_eq!(AssetType::from_extension("JPG"), AssetType::Texture);
        assert_eq!(AssetType::from_extension("wav"), AssetType::Sound);
        assert_eq!(AssetType::from_extension("toml"), AssetType::Scene);
        assert_eq!(AssetType::from_extension("wgsl"), AssetType::Shader);
        assert_eq!(AssetType::from_extension("glb"), AssetType::Model);
        assert_eq!(AssetType::from_extension("wasm"), AssetType::Script);
        assert_eq!(AssetType::from_extension("xyz"), AssetType::Other);
    }

    #[test]
    fn registry_register() {
        let mut reg = AssetRegistry::new();
        let h = reg.register("textures/brick.png", AssetType::Texture);
        assert!(h.is_valid());
        assert_eq!(reg.count(), 1);

        let entry = reg.get(h).unwrap();
        assert_eq!(entry.asset_type, AssetType::Texture);
    }

    #[test]
    fn registry_register_auto() {
        let mut reg = AssetRegistry::new();
        let h = reg.register_auto("sounds/explosion.wav");
        let entry = reg.get(h).unwrap();
        assert_eq!(entry.asset_type, AssetType::Sound);
    }

    #[test]
    fn registry_duplicate_returns_same_handle() {
        let mut reg = AssetRegistry::new();
        let h1 = reg.register("test.png", AssetType::Texture);
        let h2 = reg.register("test.png", AssetType::Texture);
        assert_eq!(h1, h2);
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn registry_handle_for() {
        let mut reg = AssetRegistry::new();
        let h = reg.register("test.toml", AssetType::Scene);
        assert_eq!(reg.handle_for("test.toml"), Some(h));
        assert_eq!(reg.handle_for("missing.toml"), None);
    }

    #[test]
    fn registry_path_for() {
        let mut reg = AssetRegistry::new();
        let h = reg.register("models/char.glb", AssetType::Model);
        assert_eq!(
            reg.path_for(h).unwrap().to_str().unwrap(),
            "models/char.glb"
        );
    }

    #[test]
    fn registry_assets_of_type() {
        let mut reg = AssetRegistry::new();
        reg.register("a.png", AssetType::Texture);
        reg.register("b.png", AssetType::Texture);
        reg.register("c.wav", AssetType::Sound);

        let textures = reg.assets_of_type(AssetType::Texture);
        assert_eq!(textures.len(), 2);
        let sounds = reg.assets_of_type(AssetType::Sound);
        assert_eq!(sounds.len(), 1);
    }

    #[test]
    fn registry_poll_no_changes() {
        let mut reg = AssetRegistry::new();
        let changes = reg.poll_changes();
        assert!(changes.is_empty());
    }

    #[test]
    fn asset_handle_serde() {
        let h = AssetHandle(42);
        let json = serde_json::to_string(&h).unwrap();
        let decoded: AssetHandle = serde_json::from_str(&json).unwrap();
        assert_eq!(h, decoded);
    }

    #[test]
    fn asset_type_serde() {
        for t in [
            AssetType::Texture,
            AssetType::Sound,
            AssetType::Scene,
            AssetType::Shader,
            AssetType::Model,
            AssetType::Script,
            AssetType::Other,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let decoded: AssetType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, decoded);
        }
    }

    #[test]
    fn registry_as_world_resource() {
        let mut world = crate::World::new();
        let mut reg = AssetRegistry::new();
        reg.register("game.toml", AssetType::Scene);
        world.insert_resource(reg);

        let reg = world.get_resource::<AssetRegistry>().unwrap();
        assert_eq!(reg.count(), 1);
    }
}
