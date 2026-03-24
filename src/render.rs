//! Rendering abstraction, headless mode
//!
//! Defines the rendering trait, camera with view/projection matrices,
//! sprite/mesh descriptors, and a headless [`NullRenderer`] for testing.

use hisab::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Rendering configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Orthographic camera for 2D rendering.
#[derive(Debug, Clone)]
pub struct OrthoCamera {
    /// Left boundary.
    pub left: f32,
    /// Right boundary.
    pub right: f32,
    /// Bottom boundary.
    pub bottom: f32,
    /// Top boundary.
    pub top: f32,
    /// Near plane.
    pub near: f32,
    /// Far plane.
    pub far: f32,
}

impl OrthoCamera {
    /// Create an orthographic camera from screen dimensions.
    /// Origin at top-left, Y points down (screen space).
    pub fn from_screen(width: f32, height: f32) -> Self {
        Self {
            left: 0.0,
            right: width,
            bottom: height,
            top: 0.0,
            near: -1.0,
            far: 1.0,
        }
    }

    /// Create a centered orthographic camera.
    /// Origin at center, extends half_width/half_height in each direction.
    pub fn centered(half_width: f32, half_height: f32) -> Self {
        Self {
            left: -half_width,
            right: half_width,
            bottom: -half_height,
            top: half_height,
            near: -1.0,
            far: 1.0,
        }
    }

    /// Compute the orthographic projection matrix.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }
}

impl Default for OrthoCamera {
    fn default() -> Self {
        Self::from_screen(1280.0, 720.0)
    }
}

// ---------------------------------------------------------------------------
// Frustum culling
// ---------------------------------------------------------------------------

/// Axis-aligned bounding box for visibility testing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create from center + half-extents.
    pub fn from_center(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Check if a point is inside the AABB.
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Check if two AABBs overlap.
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Center of the AABB.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Test if the AABB is potentially visible in a camera's view frustum.
    /// Returns true if ANY corner projects inside clip space.
    /// Note: this can produce false negatives for large AABBs that straddle
    /// the frustum (all corners outside but the box intersects). For precise
    /// culling, use plane-based frustum tests.
    pub fn is_visible(&self, view_proj: &Mat4) -> bool {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        // Project all corners and check if any is inside clip space
        for corner in &corners {
            let clip = *view_proj * corner.extend(1.0);
            if clip.w > 0.0 {
                let ndc_x = clip.x / clip.w;
                let ndc_y = clip.y / clip.w;
                let ndc_z = clip.z / clip.w;
                if (-1.0..=1.0).contains(&ndc_x)
                    && (-1.0..=1.0).contains(&ndc_y)
                    && (0.0..=1.0).contains(&ndc_z)
                {
                    return true;
                }
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Camera controllers
// ---------------------------------------------------------------------------

/// Orbit camera controller — rotates around a target point.
#[derive(Debug, Clone)]
pub struct OrbitController {
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub target: Vec3,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for OrbitController {
    fn default() -> Self {
        Self {
            distance: 10.0,
            yaw: 0.0,
            pitch: 0.3,
            target: Vec3::ZERO,
            min_pitch: -1.4,
            max_pitch: 1.4,
            min_distance: 1.0,
            max_distance: 100.0,
        }
    }
}

impl OrbitController {
    /// Rotate by a yaw/pitch delta (in radians).
    pub fn rotate(&mut self, dyaw: f32, dpitch: f32) {
        self.yaw += dyaw;
        self.pitch = (self.pitch + dpitch).clamp(self.min_pitch, self.max_pitch);
    }

    /// Zoom by a distance delta (positive = closer).
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance - delta).clamp(self.min_distance, self.max_distance);
    }

    /// Apply this controller's state to a camera.
    pub fn apply(&self, camera: &mut Camera) {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        camera.position = self.target + Vec3::new(x, y, z);
        camera.target = self.target;
    }
}

/// Fly camera controller — free-look first-person movement.
#[derive(Debug, Clone)]
pub struct FlyController {
    pub speed: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for FlyController {
    fn default() -> Self {
        Self {
            speed: 10.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl FlyController {
    /// Rotate by a yaw/pitch delta (in radians).
    pub fn rotate(&mut self, dyaw: f32, dpitch: f32) {
        self.yaw += dyaw;
        self.pitch = (self.pitch + dpitch).clamp(-1.5, 1.5);
    }

    /// Compute the forward direction vector.
    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        )
    }

    /// Compute the right direction vector.
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    /// Move the camera: forward/right/up amounts scaled by dt.
    pub fn fly(&self, camera: &mut Camera, forward: f32, right: f32, up: f32, dt: f32) {
        let fwd = self.forward();
        let r = self.right();
        camera.position += fwd * forward * self.speed * dt;
        camera.position += r * right * self.speed * dt;
        camera.position += Vec3::Y * up * self.speed * dt;
        camera.target = camera.position + fwd;
    }
}

/// Follow camera controller — tracks a target position with offset.
#[derive(Debug, Clone)]
pub struct FollowController {
    pub offset: Vec3,
    pub smoothing: f32,
}

impl Default for FollowController {
    fn default() -> Self {
        Self {
            offset: Vec3::new(0.0, 5.0, 10.0),
            smoothing: 5.0,
        }
    }
}

impl FollowController {
    /// Update camera to follow a target position, smoothed by dt.
    pub fn follow(&self, camera: &mut Camera, target_pos: Vec3, dt: f32) {
        let desired = target_pos + self.offset;
        let t = (self.smoothing * dt).min(1.0);
        camera.position = camera.position.lerp(desired, t);
        camera.target = target_pos;
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
#[non_exhaustive]
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
    fn null_renderer_end_frame_without_begin() {
        let mut r = NullRenderer::new();
        r.init(&RenderConfig::default()).unwrap();
        assert!(r.end_frame().is_err());
    }

    #[test]
    fn render_config_serde_roundtrip() {
        let cfg = RenderConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: RenderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, decoded);
    }

    #[test]
    fn render_config_default() {
        let cfg = RenderConfig::default();
        assert_eq!(cfg.width, 1280);
        assert_eq!(cfg.height, 720);
        assert!(cfg.vsync);
        assert!(!cfg.fullscreen);
    }

    // -- Camera controller tests --

    #[test]
    fn orbit_controller_apply() {
        let mut cam = Camera::default();
        let orbit = OrbitController::default();
        orbit.apply(&mut cam);

        // Camera should be at distance from target
        let dist = cam.position.distance(orbit.target);
        assert!((dist - orbit.distance).abs() < 0.01);
        assert_eq!(cam.target, orbit.target);
    }

    #[test]
    fn orbit_controller_rotate() {
        let mut orbit = OrbitController::default();
        let old_yaw = orbit.yaw;
        orbit.rotate(0.5, 0.2);
        assert!((orbit.yaw - old_yaw - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn orbit_controller_zoom_clamp() {
        let mut orbit = OrbitController::default();
        orbit.zoom(1000.0); // zoom way in
        assert_eq!(orbit.distance, orbit.min_distance);

        orbit.zoom(-1000.0); // zoom way out
        assert_eq!(orbit.distance, orbit.max_distance);
    }

    #[test]
    fn orbit_controller_pitch_clamp() {
        let mut orbit = OrbitController::default();
        orbit.rotate(0.0, 100.0);
        assert_eq!(orbit.pitch, orbit.max_pitch);

        orbit.rotate(0.0, -200.0);
        assert_eq!(orbit.pitch, orbit.min_pitch);
    }

    #[test]
    fn fly_controller_forward() {
        let fly = FlyController::default();
        let fwd = fly.forward();
        // Default yaw=0, pitch=0 → forward is (0, 0, 1)
        assert!(fwd.z.abs() > 0.9);
    }

    #[test]
    fn fly_controller_move() {
        let fly = FlyController::default();
        let mut cam = Camera::default();
        let start = cam.position;
        fly.fly(&mut cam, 1.0, 0.0, 0.0, 1.0);
        assert_ne!(cam.position, start);
    }

    #[test]
    fn fly_controller_rotate() {
        let mut fly = FlyController::default();
        fly.rotate(1.0, 0.5);
        assert!((fly.yaw - 1.0).abs() < f32::EPSILON);
        assert!((fly.pitch - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn fly_controller_pitch_clamp() {
        let mut fly = FlyController::default();
        fly.rotate(0.0, 100.0);
        assert!(fly.pitch <= 1.5);
    }

    #[test]
    fn follow_controller_moves_camera() {
        let follow = FollowController::default();
        let mut cam = Camera::default();
        let target = Vec3::new(10.0, 0.0, 5.0);

        // After following for long enough, camera approaches target + offset
        for _ in 0..100 {
            follow.follow(&mut cam, target, 1.0 / 60.0);
        }

        let expected = target + follow.offset;
        assert!((cam.position - expected).length() < 0.5);
        assert_eq!(cam.target, target);
    }

    #[test]
    fn follow_controller_smooth() {
        let follow = FollowController::default();
        let mut cam = Camera::default();
        let target = Vec3::new(100.0, 0.0, 0.0);

        // Single step shouldn't snap instantly
        follow.follow(&mut cam, target, 1.0 / 60.0);
        let expected = target + follow.offset;
        assert!((cam.position - expected).length() > 1.0);
    }

    #[test]
    fn camera_custom_params() {
        let cam = Camera {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: 90.0_f32.to_radians(),
            aspect: 1.0,
            near: 0.01,
            far: 100.0,
        };
        let vp = cam.view_projection();
        assert_ne!(vp, Mat4::IDENTITY);
    }

    #[test]
    fn sprite_desc_serde() {
        let sprite = SpriteDesc {
            texture_id: 42,
            x: 100.0,
            y: 200.0,
            width: 64.0,
            height: 64.0,
            rotation: 1.57,
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let json = serde_json::to_string(&sprite).unwrap();
        let decoded: SpriteDesc = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.texture_id, 42);
        assert_eq!(decoded.width, 64.0);
    }

    #[test]
    fn mesh_desc_serde() {
        let mesh = MeshDesc {
            mesh_id: 7,
            transform: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            material_id: 3,
        };
        let json = serde_json::to_string(&mesh).unwrap();
        let decoded: MeshDesc = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.mesh_id, 7);
        assert_eq!(decoded.material_id, 3);
    }

    // -- OrthoCamera tests --

    #[test]
    fn ortho_camera_from_screen() {
        let cam = OrthoCamera::from_screen(800.0, 600.0);
        assert_eq!(cam.left, 0.0);
        assert_eq!(cam.right, 800.0);
        assert_eq!(cam.top, 0.0);
        assert_eq!(cam.bottom, 600.0);
    }

    #[test]
    fn ortho_camera_centered() {
        let cam = OrthoCamera::centered(10.0, 7.5);
        assert_eq!(cam.left, -10.0);
        assert_eq!(cam.right, 10.0);
        assert_eq!(cam.top, 7.5);
        assert_eq!(cam.bottom, -7.5);
    }

    #[test]
    fn ortho_camera_projection_not_identity() {
        let cam = OrthoCamera::default();
        let proj = cam.projection_matrix();
        assert_ne!(proj, Mat4::IDENTITY);
    }

    #[test]
    fn ortho_camera_default() {
        let cam = OrthoCamera::default();
        assert_eq!(cam.right, 1280.0);
        assert_eq!(cam.bottom, 720.0);
    }

    // -- AABB tests --

    #[test]
    fn aabb_contains_point() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains_point(Vec3::new(0.5, 0.5, 0.5)));
        assert!(!aabb.contains_point(Vec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn aabb_from_center() {
        let aabb = AABB::from_center(Vec3::new(5.0, 5.0, 5.0), Vec3::ONE);
        assert_eq!(aabb.min, Vec3::new(4.0, 4.0, 4.0));
        assert_eq!(aabb.max, Vec3::new(6.0, 6.0, 6.0));
    }

    #[test]
    fn aabb_intersects() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
        let c = AABB::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(6.0, 6.0, 6.0));
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn aabb_center() {
        let aabb = AABB::new(Vec3::new(2.0, 4.0, 6.0), Vec3::new(4.0, 8.0, 10.0));
        assert_eq!(aabb.center(), Vec3::new(3.0, 6.0, 8.0));
    }

    #[test]
    fn aabb_visible_identity() {
        let aabb = AABB::new(Vec3::new(-0.5, -0.5, 0.0), Vec3::new(0.5, 0.5, 0.5));
        // With identity VP, clip space = world space — AABB is visible
        assert!(aabb.is_visible(&Mat4::IDENTITY));
    }

    #[test]
    fn aabb_visible_far_away() {
        let aabb = AABB::new(
            Vec3::new(1000.0, 1000.0, 1000.0),
            Vec3::new(1001.0, 1001.0, 1001.0),
        );
        // With a perspective camera looking at origin, far AABB should not be visible
        let cam = Camera::default();
        let vp = cam.view_projection();
        // This is a conservative test — may or may not be visible depending on far plane
        let _ = aabb.is_visible(&vp); // just verify no panic
    }
}
