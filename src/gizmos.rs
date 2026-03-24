//! Debug gizmos — draw debug shapes from game code.
//!
//! Provides a [`Gizmos`] resource for drawing debug lines, boxes, spheres,
//! and rays. Shapes are collected per frame and rendered by the debug pipeline.

/// A debug draw command.
#[derive(Debug, Clone)]
pub enum GizmoCommand {
    Line {
        a: [f32; 3],
        b: [f32; 3],
        color: [f32; 4],
    },
    Box {
        min: [f32; 3],
        max: [f32; 3],
        color: [f32; 4],
    },
    Sphere {
        center: [f32; 3],
        radius: f32,
        color: [f32; 4],
    },
    Ray {
        origin: [f32; 3],
        direction: [f32; 3],
        length: f32,
        color: [f32; 4],
    },
    Point {
        position: [f32; 3],
        size: f32,
        color: [f32; 4],
    },
}

/// Per-frame debug gizmo accumulator.
/// Insert as a resource, draw shapes from systems, clear each frame.
#[derive(Debug, Default)]
pub struct Gizmos {
    commands: Vec<GizmoCommand>,
}

impl Gizmos {
    pub fn new() -> Self {
        Self::default()
    }

    /// Draw a line between two points.
    pub fn line(&mut self, a: [f32; 3], b: [f32; 3], color: [f32; 4]) {
        self.commands.push(GizmoCommand::Line { a, b, color });
    }

    /// Draw a wireframe box.
    pub fn draw_box(&mut self, min: [f32; 3], max: [f32; 3], color: [f32; 4]) {
        self.commands.push(GizmoCommand::Box { min, max, color });
    }

    /// Draw a wireframe sphere.
    pub fn sphere(&mut self, center: [f32; 3], radius: f32, color: [f32; 4]) {
        self.commands.push(GizmoCommand::Sphere {
            center,
            radius,
            color,
        });
    }

    /// Draw a ray from origin in direction for length.
    pub fn ray(&mut self, origin: [f32; 3], direction: [f32; 3], length: f32, color: [f32; 4]) {
        self.commands.push(GizmoCommand::Ray {
            origin,
            direction,
            length,
            color,
        });
    }

    /// Draw a point (rendered as small cross).
    pub fn point(&mut self, position: [f32; 3], size: f32, color: [f32; 4]) {
        self.commands.push(GizmoCommand::Point {
            position,
            size,
            color,
        });
    }

    /// Get all pending gizmo commands.
    pub fn commands(&self) -> &[GizmoCommand] {
        &self.commands
    }

    /// Number of pending gizmo commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Clear all gizmo commands (call at start of each frame).
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gizmos_default() {
        let g = Gizmos::new();
        assert!(g.is_empty());
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn gizmos_draw_shapes() {
        let mut g = Gizmos::new();
        g.line([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [1.0, 0.0, 0.0, 1.0]);
        g.draw_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 0.0, 1.0]);
        g.sphere([5.0, 0.0, 0.0], 2.0, [0.0, 0.0, 1.0, 1.0]);
        g.ray([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 10.0, [1.0, 1.0, 0.0, 1.0]);
        g.point([3.0, 4.0, 5.0], 0.1, [1.0, 1.0, 1.0, 1.0]);

        assert_eq!(g.len(), 5);
    }

    #[test]
    fn gizmos_clear() {
        let mut g = Gizmos::new();
        g.line([0.0; 3], [1.0; 3], [1.0; 4]);
        g.clear();
        assert!(g.is_empty());
    }

    #[test]
    fn gizmos_as_resource() {
        let mut world = crate::World::new();
        world.insert_resource(Gizmos::new());

        {
            let g = world.get_resource_mut::<Gizmos>().unwrap();
            g.line([0.0; 3], [1.0; 3], [1.0; 4]);
        }

        let g = world.get_resource::<Gizmos>().unwrap();
        assert_eq!(g.len(), 1);
    }

    #[test]
    fn gizmo_commands_accessible() {
        let mut g = Gizmos::new();
        g.draw_box([0.0; 3], [1.0; 3], [1.0; 4]);
        let cmds = g.commands();
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            GizmoCommand::Box { min, max, .. } => {
                assert_eq!(*min, [0.0; 3]);
                assert_eq!(*max, [1.0; 3]);
            }
            _ => panic!("expected box"),
        }
    }
}
