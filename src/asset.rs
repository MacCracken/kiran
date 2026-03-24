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
#[non_exhaustive]
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

// ---------------------------------------------------------------------------
// Async asset loading
// ---------------------------------------------------------------------------

/// Load status for an async asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoadStatus {
    /// Not yet started.
    Pending,
    /// Currently loading.
    Loading,
    /// Loaded successfully.
    Ready,
    /// Load failed.
    Failed,
}

/// An async load request — tracks background file reads.
#[derive(Debug)]
pub struct AsyncLoadRequest {
    pub handle: AssetHandle,
    pub path: PathBuf,
    pub status: LoadStatus,
    /// The loaded bytes (populated when status == Ready).
    pub data: Option<Vec<u8>>,
    /// Error message if status == Failed.
    pub error: Option<String>,
}

/// Async asset loader — queues file reads on a background thread.
pub struct AsyncAssetLoader {
    /// Pending load requests.
    requests: Vec<AsyncLoadRequest>,
    /// Completed requests ready to be consumed.
    completed: Vec<AsyncLoadRequest>,
}

impl Default for AsyncAssetLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncAssetLoader {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            completed: Vec::new(),
        }
    }

    /// Queue an asset for background loading.
    pub fn load(&mut self, handle: AssetHandle, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();
        tracing::debug!(handle = handle.0, path = %path.display(), "queued async load");
        self.requests.push(AsyncLoadRequest {
            handle,
            path,
            status: LoadStatus::Pending,
            data: None,
            error: None,
        });
    }

    /// Process pending loads (reads files synchronously in batches).
    ///
    /// For true async, this would dispatch to a thread pool. Currently reads
    /// up to `max_per_frame` files per call to avoid stalling the game loop.
    pub fn poll(&mut self, max_per_frame: usize) {
        let mut processed = 0;
        for request in &mut self.requests {
            if processed >= max_per_frame {
                break;
            }
            if request.status != LoadStatus::Pending {
                continue;
            }

            request.status = LoadStatus::Loading;
            match std::fs::read(&request.path) {
                Ok(data) => {
                    request.status = LoadStatus::Ready;
                    let size = data.len();
                    request.data = Some(data);
                    tracing::info!(
                        handle = request.handle.0,
                        path = %request.path.display(),
                        size,
                        "asset loaded"
                    );
                }
                Err(e) => {
                    request.status = LoadStatus::Failed;
                    request.error = Some(e.to_string());
                    tracing::warn!(
                        handle = request.handle.0,
                        path = %request.path.display(),
                        error = %e,
                        "asset load failed"
                    );
                }
            }
            processed += 1;
        }

        // Move completed requests out
        let mut i = 0;
        while i < self.requests.len() {
            if self.requests[i].status == LoadStatus::Ready
                || self.requests[i].status == LoadStatus::Failed
            {
                self.completed.push(self.requests.swap_remove(i));
            } else {
                i += 1;
            }
        }
    }

    /// Drain completed load results.
    pub fn drain_completed(&mut self) -> Vec<AsyncLoadRequest> {
        std::mem::take(&mut self.completed)
    }

    /// Number of pending (not yet started or in-progress) loads.
    #[must_use]
    #[inline]
    pub fn pending_count(&self) -> usize {
        self.requests.len()
    }

    /// Number of completed loads waiting to be consumed.
    #[must_use]
    #[inline]
    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    /// Whether there are any loads in progress or pending.
    #[must_use]
    #[inline]
    pub fn is_busy(&self) -> bool {
        !self.requests.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Asset preprocessing
// ---------------------------------------------------------------------------

/// Asset preprocessing step — transforms applied at build time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PreprocessStep {
    /// Compress a texture to a target format.
    CompressTexture { format: String, quality: u8 },
    /// Optimize a mesh (reorder indices for vertex cache).
    OptimizeMesh,
    /// Generate mipmaps for a texture.
    GenerateMipmaps,
    /// Strip unused animation channels.
    StripAnimations,
    /// Custom preprocessing command.
    Custom { command: String },
}

/// An asset preprocessing pipeline — a sequence of steps applied to assets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreprocessPipeline {
    /// Steps to apply, in order.
    pub steps: Vec<(AssetType, PreprocessStep)>,
}

impl PreprocessPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a preprocessing step for a given asset type.
    pub fn add_step(&mut self, asset_type: AssetType, step: PreprocessStep) {
        self.steps.push((asset_type, step));
    }

    /// Get steps applicable to a given asset type.
    #[must_use]
    pub fn steps_for(&self, asset_type: AssetType) -> Vec<&PreprocessStep> {
        self.steps
            .iter()
            .filter(|(t, _)| *t == asset_type)
            .map(|(_, s)| s)
            .collect()
    }

    /// Process an asset's raw bytes through the pipeline.
    ///
    /// Returns the (potentially transformed) bytes and a log of applied steps.
    /// Currently a no-op pass-through — actual compression/optimization would
    /// call into image/mesh processing libraries.
    pub fn process(&self, asset_type: AssetType, data: Vec<u8>) -> (Vec<u8>, Vec<String>) {
        let steps = self.steps_for(asset_type);
        let mut log = Vec::new();

        if steps.is_empty() {
            return (data, log);
        }

        let result = data;
        for step in steps {
            match step {
                PreprocessStep::CompressTexture { format, quality } => {
                    log.push(format!("compress_texture({format}, q={quality})"));
                    // Actual compression would happen here
                }
                PreprocessStep::OptimizeMesh => {
                    log.push("optimize_mesh".to_string());
                    // Actual vertex cache optimization would happen here
                }
                PreprocessStep::GenerateMipmaps => {
                    log.push("generate_mipmaps".to_string());
                }
                PreprocessStep::StripAnimations => {
                    log.push("strip_animations".to_string());
                }
                PreprocessStep::Custom { command } => {
                    log.push(format!("custom({command})"));
                }
            }
        }

        (result, log)
    }

    /// Number of registered steps.
    #[must_use]
    #[inline]
    pub fn step_count(&self) -> usize {
        self.steps.len()
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

    // -- Async loader tests --

    #[test]
    fn async_loader_new() {
        let loader = AsyncAssetLoader::new();
        assert_eq!(loader.pending_count(), 0);
        assert_eq!(loader.completed_count(), 0);
        assert!(!loader.is_busy());
    }

    #[test]
    fn async_loader_queue() {
        let mut loader = AsyncAssetLoader::new();
        loader.load(AssetHandle(1), "test.png");
        assert_eq!(loader.pending_count(), 1);
        assert!(loader.is_busy());
    }

    #[test]
    fn async_loader_poll_missing_file() {
        let mut loader = AsyncAssetLoader::new();
        loader.load(AssetHandle(1), "/nonexistent/path/file.png");
        loader.poll(10);
        assert_eq!(loader.pending_count(), 0);
        let completed = loader.drain_completed();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].status, LoadStatus::Failed);
        assert!(completed[0].error.is_some());
    }

    #[test]
    fn async_loader_poll_limit() {
        let mut loader = AsyncAssetLoader::new();
        for i in 0..5 {
            loader.load(AssetHandle(i + 1), format!("/missing/{i}.png"));
        }
        loader.poll(2); // only process 2
        // 2 completed (failed), 3 still pending
        assert_eq!(loader.completed_count(), 2);
        assert_eq!(loader.pending_count(), 3);
    }

    #[test]
    fn async_loader_drain_clears() {
        let mut loader = AsyncAssetLoader::new();
        loader.load(AssetHandle(1), "/missing/a.png");
        loader.poll(10);
        assert_eq!(loader.completed_count(), 1);
        let _ = loader.drain_completed();
        assert_eq!(loader.completed_count(), 0);
    }

    // -- Preprocess pipeline tests --

    #[test]
    fn preprocess_pipeline_new() {
        let pipeline = PreprocessPipeline::new();
        assert_eq!(pipeline.step_count(), 0);
    }

    #[test]
    fn preprocess_pipeline_add_steps() {
        let mut pipeline = PreprocessPipeline::new();
        pipeline.add_step(
            AssetType::Texture,
            PreprocessStep::CompressTexture {
                format: "bc7".into(),
                quality: 80,
            },
        );
        pipeline.add_step(AssetType::Texture, PreprocessStep::GenerateMipmaps);
        pipeline.add_step(AssetType::Model, PreprocessStep::OptimizeMesh);
        assert_eq!(pipeline.step_count(), 3);
    }

    #[test]
    fn preprocess_pipeline_steps_for() {
        let mut pipeline = PreprocessPipeline::new();
        pipeline.add_step(AssetType::Texture, PreprocessStep::GenerateMipmaps);
        pipeline.add_step(AssetType::Model, PreprocessStep::OptimizeMesh);
        pipeline.add_step(
            AssetType::Texture,
            PreprocessStep::CompressTexture {
                format: "astc".into(),
                quality: 90,
            },
        );

        let tex_steps = pipeline.steps_for(AssetType::Texture);
        assert_eq!(tex_steps.len(), 2);
        let model_steps = pipeline.steps_for(AssetType::Model);
        assert_eq!(model_steps.len(), 1);
        let sound_steps = pipeline.steps_for(AssetType::Sound);
        assert!(sound_steps.is_empty());
    }

    #[test]
    fn preprocess_pipeline_process() {
        let mut pipeline = PreprocessPipeline::new();
        pipeline.add_step(AssetType::Texture, PreprocessStep::GenerateMipmaps);
        pipeline.add_step(
            AssetType::Texture,
            PreprocessStep::CompressTexture {
                format: "bc7".into(),
                quality: 80,
            },
        );

        let data = vec![1, 2, 3, 4];
        let (result, log) = pipeline.process(AssetType::Texture, data.clone());
        assert_eq!(result, data); // pass-through for now
        assert_eq!(log.len(), 2);
        assert!(log[0].contains("generate_mipmaps"));
        assert!(log[1].contains("compress_texture"));
    }

    #[test]
    fn preprocess_pipeline_process_no_steps() {
        let pipeline = PreprocessPipeline::new();
        let data = vec![1, 2, 3];
        let (result, log) = pipeline.process(AssetType::Sound, data.clone());
        assert_eq!(result, data);
        assert!(log.is_empty());
    }

    #[test]
    fn preprocess_pipeline_serde() {
        let mut pipeline = PreprocessPipeline::new();
        pipeline.add_step(AssetType::Texture, PreprocessStep::GenerateMipmaps);
        pipeline.add_step(AssetType::Model, PreprocessStep::OptimizeMesh);
        pipeline.add_step(
            AssetType::Texture,
            PreprocessStep::Custom {
                command: "optipng".into(),
            },
        );

        let json = serde_json::to_string(&pipeline).unwrap();
        let decoded: PreprocessPipeline = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.step_count(), 3);
    }

    #[test]
    fn load_status_variants() {
        assert_ne!(LoadStatus::Pending, LoadStatus::Loading);
        assert_ne!(LoadStatus::Ready, LoadStatus::Failed);
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
