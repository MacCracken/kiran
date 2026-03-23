//! kiran-render — Rendering abstraction, headless mode
//!
//! Defines the rendering trait, camera with view/projection matrices,
//! sprite/mesh descriptors, and a headless [`NullRenderer`] for testing.

use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Rendering configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub fullscreen: bool,
    pub title: String,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            vsync: true,
            fullscreen: false,
            title: "Kiran".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

/// A 3D camera with view and projection matrices.
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: 60.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl Camera {
    /// Compute the view matrix (world -> camera).
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Compute the perspective projection matrix.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
    }

    /// Combined view-projection matrix.
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

// ---------------------------------------------------------------------------
// Draw descriptors
// ---------------------------------------------------------------------------

/// Describes a 2D sprite to render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteDesc {
    pub texture_id: u64,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub color: [f32; 4],
}

/// Describes a 3D mesh to render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDesc {
    pub mesh_id: u64,
    pub transform: [f32; 16], // column-major 4x4
    pub material_id: u64,
}

/// A draw command submitted to the renderer.
#[derive(Debug, Clone)]
pub enum DrawCommand {
    Clear([f32; 4]),
    Sprite(SpriteDesc),
    Mesh(MeshDesc),
    SetCamera(Camera),
}

// ---------------------------------------------------------------------------
// Renderer trait
// ---------------------------------------------------------------------------

/// Abstraction over a rendering backend.
pub trait Renderer: Send {
    /// Initialize the renderer with the given config.
    fn init(&mut self, config: &RenderConfig) -> Result<(), String>;

    /// Begin a new frame.
    fn begin_frame(&mut self) -> Result<(), String>;

    /// Submit a draw command.
    fn submit(&mut self, command: DrawCommand) -> Result<(), String>;

    /// End the current frame and present.
    fn end_frame(&mut self) -> Result<(), String>;

    /// Shut down the renderer and release resources.
    fn shutdown(&mut self) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// NullRenderer (headless)
// ---------------------------------------------------------------------------

/// A headless renderer that records draw commands for testing.
#[derive(Debug, Default)]
pub struct NullRenderer {
    pub initialized: bool,
    pub frame_count: u64,
    pub commands: Vec<DrawCommand>,
    in_frame: bool,
}

impl NullRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// How many commands were submitted in the last completed frame.
    pub fn last_frame_command_count(&self) -> usize {
        self.commands.len()
    }
}

impl Renderer for NullRenderer {
    fn init(&mut self, _config: &RenderConfig) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }

    fn begin_frame(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Renderer not initialized".into());
        }
        self.commands.clear();
        self.in_frame = true;
        Ok(())
    }

    fn submit(&mut self, command: DrawCommand) -> Result<(), String> {
        if !self.in_frame {
            return Err("Not in a frame".into());
        }
        self.commands.push(command);
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), String> {
        if !self.in_frame {
            return Err("Not in a frame".into());
        }
        self.in_frame = false;
        self.frame_count += 1;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        self.initialized = false;
        self.in_frame = false;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_default_matrices() {
        let cam = Camera::default();
        let view = cam.view_matrix();
        let proj = cam.projection_matrix();
        // Both should be non-zero, non-identity
        assert_ne!(view, Mat4::IDENTITY);
        assert_ne!(proj, Mat4::IDENTITY);
    }

    #[test]
    fn camera_view_projection() {
        let cam = Camera::default();
        let vp = cam.view_projection();
        let expected = cam.projection_matrix() * cam.view_matrix();
        assert_eq!(vp, expected);
    }

    #[test]
    fn null_renderer_lifecycle() {
        let mut r = NullRenderer::new();
        let cfg = RenderConfig::default();

        r.init(&cfg).unwrap();
        assert!(r.initialized);

        r.begin_frame().unwrap();
        r.submit(DrawCommand::Clear([0.0, 0.0, 0.0, 1.0])).unwrap();
        r.end_frame().unwrap();

        assert_eq!(r.frame_count, 1);
        assert_eq!(r.last_frame_command_count(), 1);
    }

    #[test]
    fn null_renderer_begin_without_init() {
        let mut r = NullRenderer::new();
        assert!(r.begin_frame().is_err());
    }

    #[test]
    fn null_renderer_submit_outside_frame() {
        let mut r = NullRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        assert!(r.submit(DrawCommand::Clear([0.0; 4])).is_err());
    }

    #[test]
    fn null_renderer_multiple_frames() {
        let mut r = NullRenderer::new();
        r.init(&RenderConfig::default()).unwrap();

        for _ in 0..3 {
            r.begin_frame().unwrap();
            r.submit(DrawCommand::Clear([0.1, 0.2, 0.3, 1.0])).unwrap();
            r.end_frame().unwrap();
        }
        assert_eq!(r.frame_count, 3);
    }

    #[test]
    fn null_renderer_shutdown() {
        let mut r = NullRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        r.shutdown().unwrap();
        assert!(!r.initialized);
    }

    #[test]
    fn render_config_default() {
        let cfg = RenderConfig::default();
        assert_eq!(cfg.width, 1280);
        assert_eq!(cfg.height, 720);
        assert!(cfg.vsync);
        assert!(!cfg.fullscreen);
    }
}
