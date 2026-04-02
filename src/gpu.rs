//! GPU rendering backend via soorat, prakash, and ranga
//!
//! Bridges the AGNOS rendering stack with kiran's Renderer trait and ECS:
//! - **soorat** — GPU rendering engine (sprites, meshes, PBR pipeline, shadows, etc.)
//! - **prakash** — Physically-based optics (BRDF math, spectral color, atmosphere, lenses)
//! - **ranga** — Image processing (pixel buffers, blend modes, filters, compositing)
//!
//! Provides a [`SooratRenderer`] that implements kiran's [`Renderer`] trait.

// ---------------------------------------------------------------------------
// soorat — GPU rendering
// ---------------------------------------------------------------------------

pub use soorat::GpuContext;
pub use soorat::animation::{AnimationClip, JointUniforms, Skeleton};
pub use soorat::capabilities::GpuCapabilities;
pub use soorat::color::Color;
pub use soorat::compute::ComputePipeline;
pub use soorat::debug_draw::{LineBatch, LinePipeline, LineVertex};
pub use soorat::fluid_render::FluidColorMode;
pub use soorat::gpu_particles::{GpuParticle, GpuParticleSystem, SimParams};
pub use soorat::hdr::{BloomPipeline, BloomUniforms, HdrFramebuffer};
pub use soorat::instancing::{InstanceBuffer, InstanceData};
pub use soorat::lights::{GpuLight, LightArrayUniforms};
pub use soorat::lod::{LodChain, TerrainLod};
pub use soorat::mesh_pipeline::{
    CameraUniforms, DepthBuffer, LightUniforms, Mesh, MeshDrawParams, MeshPipeline,
};
pub use soorat::pbr_material::MaterialUniforms;
pub use soorat::pipeline::{
    FrameStats, SpriteBatchDrawParams, SpriteBuffers, SpritePipeline, batch_to_vertices,
};
pub use soorat::postprocess::{PostProcessPipeline, PostProcessUniforms, ToneMapMode};
pub use soorat::profiler::{FrameProfiler as GpuFrameProfiler, GpuTimestamps, PassTiming};
pub use soorat::render_graph::{PassType, RenderGraph, RenderPassNode};
pub use soorat::render_target::RenderTarget;
pub use soorat::shadow::{ShadowMap, ShadowPipeline, ShadowUniforms};
pub use soorat::sprite::{Sprite, SpriteBatch, UvRect};
pub use soorat::ssao::{SsaoPipeline, SsaoUniforms};
pub use soorat::terrain::{TerrainConfig, TerrainData};
pub use soorat::text::{BitmapFont, TextBatch};
pub use soorat::texture::{Texture, TextureCache, create_default_sampler};
pub use soorat::ui::{UiBatch, UiLabel, UiPanel};
pub use soorat::vertex::{SkinnedVertex3D, Vertex2D, Vertex3D};
pub use soorat::window::{
    self as soorat_window, Window as SooratWindow, WindowConfig as SooratWindowConfig,
};

// ---------------------------------------------------------------------------
// prakash — physically-based optics
// ---------------------------------------------------------------------------

/// PBR shading math (Cook-Torrance BRDF, Fresnel, GGX distribution, geometry,
/// anisotropic, clearcoat, sheen, iridescence, subsurface).
pub use prakash::pbr;

/// Spectral color science (wavelength↔RGB, blackbody, CIE, color temperature).
pub use prakash::spectral;

/// Geometric optics (Snell's law, Fresnel equations, critical angle, dispersion).
pub use prakash::ray;

/// Wave optics (interference, diffraction, polarization, coherence).
pub use prakash::wave;

/// Lens/mirror geometry (thin/thick, aberrations, MTF, depth of field).
pub use prakash::lens;

/// Atmospheric scattering (Rayleigh, Mie, sky color, optical depth).
pub use prakash::atmosphere;

pub use prakash::PrakashError;

// ---------------------------------------------------------------------------
// ranga — image processing
// ---------------------------------------------------------------------------

/// Pixel buffers and formats.
pub use ranga::pixel::{BufferPool, PixelBuffer, PixelFormat, PixelView, PixelViewMut};

/// Blend modes (Porter-Duff: Normal, Multiply, Screen, Overlay, etc.).
pub use ranga::blend::{self, BlendMode};

/// Image filters (blur, sharpen, brightness, contrast, saturation, etc.).
pub use ranga::filter;

/// Layer compositing (masks, transitions, gradients, premultiplied alpha).
pub use ranga::composite;

/// Color space conversions (sRGB, linear, HSL, Oklab, CIE, P3, CMYK).
pub use ranga::color as ranga_color;

/// Histograms (luminance, RGB, equalization, auto-levels).
pub use ranga::histogram;

/// Spatial transforms (crop, resize, affine, perspective, flip).
pub use ranga::transform;

/// Pixel format conversion (BT.601/709/2020, ARGB↔NV12).
pub use ranga::convert;

/// ICC color profile parsing (v2/v4, tone curves, embedded sRGB).
pub use ranga::icc;

pub use ranga::RangaError;

use crate::render::{Camera, DrawCommand, MeshDesc, RenderConfig, Renderer};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Texture processing pipeline
// ---------------------------------------------------------------------------

/// A filter operation in a texture processing pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TextureFilter {
    /// Brightness offset (-1.0 to 1.0).
    Brightness(f32),
    /// Contrast multiplier (1.0 = unchanged).
    Contrast(f32),
    /// Saturation multiplier (0.0 = grayscale, 1.0 = unchanged).
    Saturation(f32),
    /// Hue rotation in degrees.
    HueShift(f32),
    /// Convert to grayscale (BT.709 luminance).
    Grayscale,
    /// Invert colors.
    Invert,
    /// Gaussian blur with the given radius.
    GaussianBlur(u32),
    /// Unsharp mask sharpening.
    Sharpen {
        /// Sharpening strength.
        amount: f32,
        /// Blur radius for the unsharp mask.
        radius: u32,
    },
    /// Vignette darkening at edges (0.0–1.0).
    Vignette(f32),
    /// Vibrance (selective saturation boost).
    Vibrance(f32),
}

/// Texture processor — applies a chain of filters to a pixel buffer.
///
/// Attach to an entity alongside a texture asset to process it at load time,
/// or use standalone for runtime texture manipulation.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "rendering")] {
/// use kiran::gpu::{TextureProcessor, TextureFilter};
///
/// let proc = TextureProcessor::new()
///     .push(TextureFilter::Brightness(0.1))
///     .push(TextureFilter::Contrast(1.2))
///     .push(TextureFilter::Sharpen { amount: 0.5, radius: 1 });
/// assert_eq!(proc.len(), 3);
/// # }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextureProcessor {
    /// Ordered list of filters to apply.
    pub filters: Vec<TextureFilter>,
}

impl TextureProcessor {
    /// Create an empty texture processor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a filter to the pipeline.
    pub fn push(mut self, filter: TextureFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Number of filters in the pipeline.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Whether the pipeline is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Apply the filter pipeline to a pixel buffer (CPU path).
    pub fn apply(&self, buf: &mut PixelBuffer) -> Result<(), RangaError> {
        for f in &self.filters {
            match f {
                TextureFilter::Brightness(v) => filter::brightness(buf, *v)?,
                TextureFilter::Contrast(v) => filter::contrast(buf, *v)?,
                TextureFilter::Saturation(v) => filter::saturation(buf, *v)?,
                TextureFilter::HueShift(deg) => filter::hue_shift(buf, *deg)?,
                TextureFilter::Grayscale => filter::grayscale(buf)?,
                TextureFilter::Invert => filter::invert(buf)?,
                TextureFilter::GaussianBlur(r) => {
                    *buf = filter::gaussian_blur(buf, *r)?;
                }
                TextureFilter::Sharpen { amount, radius } => {
                    *buf = filter::unsharp_mask(buf, *radius, *amount)?;
                }
                TextureFilter::Vignette(s) => filter::vignette(buf, *s)?,
                TextureFilter::Vibrance(v) => filter::vibrance(buf, *v)?,
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Color grading
// ---------------------------------------------------------------------------

/// Color grading settings for post-processing.
///
/// Controls exposure, temperature, hue, and saturation for cinematic visuals.
/// Can be attached as an ECS resource or per-camera component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorGrading {
    /// Exposure adjustment in stops (-5.0 to 5.0).
    pub exposure: f32,
    /// Hue rotation in degrees.
    pub hue_shift: f32,
    /// Saturation multiplier (0.0 = grayscale, 1.0 = unchanged).
    pub saturation: f32,
    /// Vibrance (selective saturation, less aggressive than saturation).
    pub vibrance: f32,
    /// Brightness offset (-1.0 to 1.0).
    pub brightness: f32,
    /// Contrast multiplier.
    pub contrast: f32,
    /// Vignette strength (0.0 = none).
    pub vignette: f32,
}

impl Default for ColorGrading {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            hue_shift: 0.0,
            saturation: 1.0,
            vibrance: 0.0,
            brightness: 0.0,
            contrast: 1.0,
            vignette: 0.0,
        }
    }
}

impl ColorGrading {
    /// Create neutral color grading (no adjustments).
    pub fn neutral() -> Self {
        Self::default()
    }

    /// Apply this grading to a pixel buffer.
    pub fn apply(&self, buf: &mut PixelBuffer) -> Result<(), RangaError> {
        if (self.brightness).abs() > f32::EPSILON {
            filter::brightness(buf, self.brightness)?;
        }
        if (self.contrast - 1.0).abs() > f32::EPSILON {
            filter::contrast(buf, self.contrast)?;
        }
        if (self.saturation - 1.0).abs() > f32::EPSILON {
            filter::saturation(buf, self.saturation)?;
        }
        if (self.vibrance).abs() > f32::EPSILON {
            filter::vibrance(buf, self.vibrance)?;
        }
        if (self.hue_shift).abs() > f32::EPSILON {
            filter::hue_shift(buf, self.hue_shift)?;
        }
        if self.vignette > f32::EPSILON {
            filter::vignette(buf, self.vignette)?;
        }
        Ok(())
    }

    /// Whether all settings are at neutral (no-op).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        (self.exposure).abs() < f32::EPSILON
            && (self.hue_shift).abs() < f32::EPSILON
            && (self.saturation - 1.0).abs() < f32::EPSILON
            && (self.vibrance).abs() < f32::EPSILON
            && (self.brightness).abs() < f32::EPSILON
            && (self.contrast - 1.0).abs() < f32::EPSILON
            && self.vignette < f32::EPSILON
    }
}

// ---------------------------------------------------------------------------
// Screen capture analysis
// ---------------------------------------------------------------------------

/// Histogram analysis of a captured frame.
///
/// Use for performance debugging, QA validation, or adaptive rendering.
#[derive(Debug, Clone)]
pub struct FrameHistogram {
    /// Luminance histogram (256 bins, normalized 0.0–1.0).
    pub luminance: Vec<f64>,
    /// Per-channel RGB histograms (256 bins each).
    pub red: Vec<f64>,
    /// Green channel histogram.
    pub green: Vec<f64>,
    /// Blue channel histogram.
    pub blue: Vec<f64>,
}

impl FrameHistogram {
    /// Compute histograms from a pixel buffer.
    pub fn from_buffer(buf: &PixelBuffer) -> Result<Self, RangaError> {
        let luminance = histogram::luminance_histogram(buf, 256)?;
        let [red, green, blue] = histogram::rgb_histograms(buf)?;
        Ok(Self {
            luminance,
            red,
            green,
            blue,
        })
    }

    /// Average luminance (0.0 = black, 1.0 = white).
    #[must_use]
    pub fn average_luminance(&self) -> f64 {
        self.luminance
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64 / 255.0) * v)
            .sum()
    }

    /// Whether the frame is predominantly dark (avg luminance < 0.2).
    #[must_use]
    pub fn is_underexposed(&self) -> bool {
        self.average_luminance() < 0.2
    }

    /// Whether the frame is predominantly bright (avg luminance > 0.8).
    #[must_use]
    pub fn is_overexposed(&self) -> bool {
        self.average_luminance() > 0.8
    }
}

// ---------------------------------------------------------------------------
// SooratRenderer — bridges soorat with kiran's Renderer trait
// ---------------------------------------------------------------------------

/// GPU-accelerated renderer backed by soorat (wgpu).
///
/// Implements kiran's `Renderer` trait, translating `DrawCommand`s to soorat
/// sprite batches. This is a **data-layer bridge** — it collects and sorts
/// sprites but does not perform GPU rendering directly. Actual GPU rendering
/// happens through soorat's `Window::run()` + `SpritePipeline::draw()` which
/// require a live window and surface.
///
/// For headless testing, this behaves like NullRenderer with soorat types.
/// For real rendering, access `sprite_batch()` and `clear_color()` after
/// `end_frame()` and pass them to soorat's pipeline.
pub struct SooratRenderer {
    config: RenderConfig,
    initialized: bool,
    in_frame: bool,
    frame_count: u64,
    /// Sprites queued for the current frame.
    sprite_batch: SpriteBatch,
    /// Mesh draw commands queued for the current frame.
    mesh_queue: Vec<MeshDesc>,
    /// Clear color for the frame.
    clear_color: Color,
    /// Current camera (updated via SetCamera commands).
    camera: Option<Camera>,
}

impl SooratRenderer {
    /// Create a new soorat-backed renderer (headless-safe).
    pub fn new() -> Self {
        Self {
            config: RenderConfig::default(),
            initialized: false,
            in_frame: false,
            frame_count: 0,
            sprite_batch: SpriteBatch::new(),
            mesh_queue: Vec::new(),
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

    /// Access the mesh draw queue.
    pub fn mesh_queue(&self) -> &[MeshDesc] {
        &self.mesh_queue
    }

    /// Number of meshes queued this frame.
    pub fn mesh_count(&self) -> usize {
        self.mesh_queue.len()
    }

    /// Get the render config.
    pub fn config(&self) -> &RenderConfig {
        &self.config
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
        self.mesh_queue.clear();
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
                    uv: UvRect::FULL,
                });
            }
            DrawCommand::Mesh(desc) => {
                self.mesh_queue.push(desc);
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

    #[test]
    fn soorat_renderer_mesh_no_panic() {
        use crate::render::MeshDesc;

        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        r.begin_frame().unwrap();

        r.submit(DrawCommand::Mesh(MeshDesc {
            mesh_id: 1,
            transform: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            material_id: 0,
        }))
        .unwrap();

        // Mesh doesn't add to sprite count
        assert_eq!(r.sprite_count(), 0);
        r.end_frame().unwrap();
    }

    #[test]
    fn soorat_renderer_sprite_batch_clears_between_frames() {
        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();

        // Frame 1: add 3 sprites
        r.begin_frame().unwrap();
        for _ in 0..3 {
            r.submit(DrawCommand::Sprite(SpriteDesc {
                texture_id: 0,
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
                rotation: 0.0,
                color: [1.0; 4],
            }))
            .unwrap();
        }
        assert_eq!(r.sprite_count(), 3);
        r.end_frame().unwrap();

        // Frame 2: batch should be empty
        r.begin_frame().unwrap();
        assert_eq!(r.sprite_count(), 0);
        r.end_frame().unwrap();
    }

    #[test]
    fn soorat_renderer_default_clear_color() {
        let r = SooratRenderer::new();
        assert_eq!(r.clear_color(), Color::CORNFLOWER_BLUE);
    }

    #[test]
    fn soorat_renderer_config() {
        let mut r = SooratRenderer::new();
        let cfg = RenderConfig {
            width: 1920,
            height: 1080,
            vsync: false,
            fullscreen: true,
            title: "Test".into(),
        };
        r.init(&cfg).unwrap();

        assert_eq!(r.config().width, 1920);
        assert_eq!(r.config().height, 1080);
        assert!(!r.config().vsync);
        assert!(r.config().fullscreen);
    }

    #[test]
    fn soorat_renderer_shutdown_reinit() {
        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        r.begin_frame().unwrap();
        r.end_frame().unwrap();
        assert_eq!(r.frame_count(), 1);

        r.shutdown().unwrap();
        assert!(!r.initialized);

        // Re-init
        r.init(&RenderConfig::default()).unwrap();
        r.begin_frame().unwrap();
        r.end_frame().unwrap();
        assert_eq!(r.frame_count(), 2);
    }

    #[test]
    fn soorat_renderer_camera_persists_across_frames() {
        let mut r = SooratRenderer::new();
        r.init(&RenderConfig::default()).unwrap();

        // Frame 1: set camera
        r.begin_frame().unwrap();
        r.submit(DrawCommand::SetCamera(Camera::default())).unwrap();
        r.end_frame().unwrap();

        // Frame 2: camera should still be set
        r.begin_frame().unwrap();
        assert!(r.camera().is_some());
        r.end_frame().unwrap();
    }
}
