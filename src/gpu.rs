//! GPU rendering backend via soorat
//!
//! Bridges soorat's GPU rendering engine with kiran's Renderer trait and ECS.
//! Re-exports key soorat types and provides a [`SooratRenderer`] that implements
//! kiran's [`Renderer`](crate::render::Renderer) trait.

pub use soorat::GpuContext;
pub use soorat::color::Color;
pub use soorat::sprite::{Sprite, SpriteBatch};
pub use soorat::vertex::{Vertex2D, Vertex3D};
pub use soorat::window::WindowConfig as SooratWindowConfig;

use crate::render::{Camera, DrawCommand, RenderConfig, Renderer};

// ---------------------------------------------------------------------------
// SooratRenderer — bridges soorat with kiran's Renderer trait
// ---------------------------------------------------------------------------

/// GPU-accelerated renderer backed by soorat (wgpu).
///
/// Implements kiran's `Renderer` trait, translating `DrawCommand`s to soorat
/// sprite batches and GPU operations.
pub struct SooratRenderer {
    config: RenderConfig,
    initialized: bool,
    in_frame: bool,
    frame_count: u64,
    /// Sprites queued for the current frame.
    sprite_batch: SpriteBatch,
    /// Clear color for the frame.
    clear_color: Color,
    /// Current camera (updated via SetCamera commands).
    camera: Option<Camera>,
}

impl SooratRenderer {
    pub fn new() -> Self {
        Self {
            config: RenderConfig::default(),
            initialized: false,
            in_frame: false,
            frame_count: 0,
            sprite_batch: SpriteBatch::new(),
            clear_color: Color::CORNFLOWER_BLUE,
            camera: None,
        }
    }

    /// Get the clear color for the current frame.
    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    /// Get the number of sprites queued this frame.
    pub fn sprite_count(&self) -> usize {
        self.sprite_batch.len()
    }

    /// Get the total frames rendered.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get the current camera, if set.
    pub fn camera(&self) -> Option<&Camera> {
        self.camera.as_ref()
    }

    /// Access the sprite batch.
    pub fn sprite_batch(&self) -> &SpriteBatch {
        &self.sprite_batch
    }
}

impl Default for SooratRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for SooratRenderer {
    fn init(&mut self, config: &RenderConfig) -> Result<(), String> {
        self.config = config.clone();
        self.initialized = true;
        tracing::info!(
            width = config.width,
            height = config.height,
            vsync = config.vsync,
            "soorat renderer initialized"
        );
        Ok(())
    }

    fn begin_frame(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("SooratRenderer not initialized".into());
        }
        self.sprite_batch.clear();
        self.camera = None;
        self.clear_color = Color::CORNFLOWER_BLUE;
        self.in_frame = true;
        Ok(())
    }

    fn submit(&mut self, command: DrawCommand) -> Result<(), String> {
        if !self.in_frame {
            return Err("Not in a frame".into());
        }
        match command {
            DrawCommand::Clear(color) => {
                self.clear_color = Color::new(color[0], color[1], color[2], color[3]);
            }
            DrawCommand::Sprite(desc) => {
                self.sprite_batch.push(Sprite {
                    x: desc.x,
                    y: desc.y,
                    width: desc.width,
                    height: desc.height,
                    rotation: desc.rotation,
                    color: Color::new(desc.color[0], desc.color[1], desc.color[2], desc.color[3]),
                    texture_id: desc.texture_id,
                    z_order: 0,
                });
            }
            DrawCommand::Mesh(_desc) => {
                // 3D mesh rendering — will be implemented in soorat V0.3
            }
            DrawCommand::SetCamera(cam) => {
                self.camera = Some(cam);
            }
        }
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), String> {
        if !self.in_frame {
            return Err("Not in a frame".into());
        }
        self.sprite_batch.sort_by_z();
        self.in_frame = false;
        self.frame_count += 1;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), String> {
        self.initialized = false;
        self.in_frame = false;
        tracing::info!("soorat renderer shut down");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{DrawCommand, RenderConfig, Renderer, SpriteDesc};

    #[test]
    fn soorat_renderer_lifecycle() {
        let mut r = SooratRenderer::new();
        assert!(!r.initialized);

        r.init(&RenderConfig::default()).unwrap();
        assert!(r.initialized);

        r.begin_frame().unwrap();
        r.submit(DrawCommand::Clear([0.1, 0.2, 0.3, 1.0])).unwrap();
        r.end_frame().unwrap();

        assert_eq!(r.frame_count(), 1);

        r.shutdown().unwrap();
        assert!(!r.initialized);
    }

    #[test]
    fn soorat_renderer_clear_color() {
        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        r.begin_frame().unwrap();

        r.submit(DrawCommand::Clear([0.5, 0.6, 0.7, 1.0])).unwrap();
        assert!((r.clear_color().r - 0.5).abs() < f32::EPSILON);
        assert!((r.clear_color().g - 0.6).abs() < f32::EPSILON);

        r.end_frame().unwrap();
    }

    #[test]
    fn soorat_renderer_sprites() {
        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        r.begin_frame().unwrap();

        r.submit(DrawCommand::Sprite(SpriteDesc {
            texture_id: 1,
            x: 10.0,
            y: 20.0,
            width: 64.0,
            height: 64.0,
            rotation: 0.0,
            color: [1.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();

        r.submit(DrawCommand::Sprite(SpriteDesc {
            texture_id: 2,
            x: 100.0,
            y: 200.0,
            width: 32.0,
            height: 32.0,
            rotation: 1.57,
            color: [0.0, 1.0, 0.0, 1.0],
        }))
        .unwrap();

        assert_eq!(r.sprite_count(), 2);
        r.end_frame().unwrap();
    }

    #[test]
    fn soorat_renderer_camera() {
        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        r.begin_frame().unwrap();

        assert!(r.camera().is_none());

        let cam = Camera::default();
        r.submit(DrawCommand::SetCamera(cam)).unwrap();
        assert!(r.camera().is_some());

        r.end_frame().unwrap();
    }

    #[test]
    fn soorat_renderer_begin_without_init() {
        let mut r = SooratRenderer::new();
        assert!(r.begin_frame().is_err());
    }

    #[test]
    fn soorat_renderer_submit_outside_frame() {
        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        assert!(r.submit(DrawCommand::Clear([0.0; 4])).is_err());
    }

    #[test]
    fn soorat_renderer_end_without_begin() {
        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        assert!(r.end_frame().is_err());
    }

    #[test]
    fn soorat_renderer_multiple_frames() {
        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();

        for _ in 0..5 {
            r.begin_frame().unwrap();
            r.submit(DrawCommand::Clear([0.0, 0.0, 0.0, 1.0])).unwrap();
            r.end_frame().unwrap();
        }
        assert_eq!(r.frame_count(), 5);
    }

    #[test]
    fn soorat_color_bridge() {
        // Verify soorat Color types are accessible
        let c = Color::from_hex(0xFF0000FF);
        assert_eq!(c.r, 1.0);

        let lerped = Color::RED.lerp(Color::BLUE, 0.5);
        assert!((lerped.r - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn soorat_sprite_bridge() {
        let sprite = Sprite::new(10.0, 20.0, 64.0, 64.0)
            .with_color(Color::GREEN)
            .with_z_order(3);
        assert_eq!(sprite.z_order, 3);
        assert_eq!(sprite.center(), (42.0, 52.0));
    }
}
