//! Navigation and pathfinding via raasta
//!
//! Provides ECS components and systems for NPC navigation:
//! - [`NavAgent`] component for entities that navigate
//! - [`NavGrid`] and [`NavMesh`] re-exports from raasta
//! - Path following system

pub use raasta::{
    GridPos, NavGrid, NavMesh, NavPoly, NavPolyId, PathRequest, PathResult, PathStatus,
    SteerBehavior, SteerOutput, compute_steer, funnel_smooth,
};

use hisab::Vec2;

use crate::world::{Entity, World};

/// ECS component: a navigation agent that follows paths.
#[derive(Debug, Clone)]
pub struct NavAgent {
    /// Maximum movement speed.
    pub max_speed: f32,
    /// Current path waypoints (world space).
    pub path: Vec<Vec2>,
    /// Index of the next waypoint to reach.
    pub path_index: usize,
    /// Arrival threshold — how close to a waypoint before advancing.
    pub arrival_radius: f32,
    /// Whether the agent has reached its destination.
    pub arrived: bool,
}

impl NavAgent {
    pub fn new(max_speed: f32) -> Self {
        Self {
            max_speed,
            path: Vec::new(),
            path_index: 0,
            arrival_radius: 0.5,
            arrived: true,
        }
    }

    /// Set a new path for the agent to follow.
    pub fn set_path(&mut self, waypoints: Vec<Vec2>) {
        self.path = waypoints;
        self.path_index = 0;
        self.arrived = self.path.is_empty();
    }

    /// Clear the current path.
    pub fn stop(&mut self) {
        self.path.clear();
        self.path_index = 0;
        self.arrived = true;
    }

    /// Get the current target waypoint, if any.
    #[must_use]
    pub fn current_target(&self) -> Option<Vec2> {
        if self.arrived {
            return None;
        }
        self.path.get(self.path_index).copied()
    }

    /// Advance along the path given the agent's current position.
    /// Returns the steering output (desired velocity).
    pub fn step(&mut self, position: Vec2) -> SteerOutput {
        if self.arrived {
            return SteerOutput::default();
        }

        let radius_sq = self.arrival_radius * self.arrival_radius;

        // Skip past any waypoints within arrival radius (loop, not recursion)
        loop {
            let Some(&target) = self.path.get(self.path_index) else {
                self.arrived = true;
                return SteerOutput::default();
            };

            let diff = target - position;
            let dist_sq = diff.x * diff.x + diff.y * diff.y;

            if dist_sq >= radius_sq {
                // Not yet at this waypoint — steer toward it
                let is_last = self.path_index == self.path.len() - 1;
                let behavior = if is_last {
                    SteerBehavior::Arrive {
                        target,
                        slow_radius: self.arrival_radius * 4.0,
                    }
                } else {
                    SteerBehavior::Seek { target }
                };
                return compute_steer(&behavior, position, self.max_speed);
            }

            // Within arrival radius — advance to next waypoint
            self.path_index += 1;
            if self.path_index >= self.path.len() {
                self.arrived = true;
                return SteerOutput::default();
            }
        }
    }

    /// Remaining waypoints.
    #[must_use]
    #[inline]
    pub fn remaining_waypoints(&self) -> usize {
        if self.arrived {
            0
        } else {
            self.path.len().saturating_sub(self.path_index)
        }
    }
}

/// Request a grid path for an entity. Sets the path on its NavAgent component.
pub fn request_grid_path(
    world: &mut World,
    entity: Entity,
    grid: &NavGrid,
    start: GridPos,
    goal: GridPos,
) -> bool {
    let path = grid.find_path(start, goal);
    if let Some(positions) = path {
        let waypoints: Vec<Vec2> = positions.iter().map(|p| grid.grid_to_world(*p)).collect();

        if let Some(agent) = world.get_component_mut::<NavAgent>(entity) {
            agent.set_path(waypoints);
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_agent_new() {
        let agent = NavAgent::new(5.0);
        assert!(agent.arrived);
        assert!(agent.path.is_empty());
        assert_eq!(agent.max_speed, 5.0);
    }

    #[test]
    fn nav_agent_set_path() {
        let mut agent = NavAgent::new(5.0);
        agent.set_path(vec![
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(3.0, 0.0),
        ]);
        assert!(!agent.arrived);
        assert_eq!(agent.remaining_waypoints(), 3);
        assert_eq!(agent.current_target(), Some(Vec2::new(1.0, 0.0)));
    }

    #[test]
    fn nav_agent_stop() {
        let mut agent = NavAgent::new(5.0);
        agent.set_path(vec![Vec2::new(1.0, 0.0)]);
        assert!(!agent.arrived);
        agent.stop();
        assert!(agent.arrived);
    }

    #[test]
    fn nav_agent_step_toward_target() {
        let mut agent = NavAgent::new(5.0);
        agent.set_path(vec![Vec2::new(10.0, 0.0)]);
        let out = agent.step(Vec2::ZERO);
        assert!(out.velocity.x > 0.0);
        assert!(out.velocity.y.abs() < 0.01);
    }

    #[test]
    fn nav_agent_reaches_waypoint() {
        let mut agent = NavAgent::new(5.0);
        agent.arrival_radius = 1.0;
        agent.set_path(vec![Vec2::new(1.0, 0.0), Vec2::new(5.0, 0.0)]);

        // Step near first waypoint
        let out = agent.step(Vec2::new(0.8, 0.0));
        // Should advance to second waypoint
        assert_eq!(agent.path_index, 1);
        assert!(!agent.arrived);
        assert!(out.velocity.x > 0.0);
    }

    #[test]
    fn nav_agent_arrives_at_destination() {
        let mut agent = NavAgent::new(5.0);
        agent.arrival_radius = 1.0;
        agent.set_path(vec![Vec2::new(0.5, 0.0)]);

        let out = agent.step(Vec2::new(0.2, 0.0));
        assert!(agent.arrived);
        assert!(out.speed() < f32::EPSILON);
    }

    #[test]
    fn nav_agent_empty_path() {
        let mut agent = NavAgent::new(5.0);
        agent.set_path(vec![]);
        assert!(agent.arrived);
        let out = agent.step(Vec2::ZERO);
        assert!(out.speed() < f32::EPSILON);
    }

    #[test]
    fn nav_agent_as_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, NavAgent::new(3.0)).unwrap();
        assert!(world.has_component::<NavAgent>(e));
    }

    #[test]
    fn request_grid_path_works() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, NavAgent::new(5.0)).unwrap();

        let grid = NavGrid::new(10, 10, 1.0);
        let ok = request_grid_path(&mut world, e, &grid, GridPos::new(0, 0), GridPos::new(9, 9));
        assert!(ok);

        let agent = world.get_component::<NavAgent>(e).unwrap();
        assert!(!agent.arrived);
        assert!(!agent.path.is_empty());
    }

    #[test]
    fn request_grid_path_blocked() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, NavAgent::new(5.0)).unwrap();

        let mut grid = NavGrid::new(5, 1, 1.0);
        grid.set_walkable(2, 0, false);

        let ok = request_grid_path(&mut world, e, &grid, GridPos::new(0, 0), GridPos::new(4, 0));
        assert!(!ok);

        let agent = world.get_component::<NavAgent>(e).unwrap();
        assert!(agent.arrived); // unchanged
    }

    #[test]
    fn nav_grid_reexport() {
        let grid = NavGrid::new(5, 5, 1.0);
        assert_eq!(grid.width(), 5);
    }

    #[test]
    fn nav_mesh_reexport() {
        let mesh = NavMesh::new();
        assert_eq!(mesh.poly_count(), 0);
    }

    #[test]
    fn steer_reexport() {
        let out = compute_steer(
            &SteerBehavior::Seek {
                target: Vec2::new(1.0, 0.0),
            },
            Vec2::ZERO,
            1.0,
        );
        assert!(out.speed() > 0.0);
    }

    #[test]
    fn smooth_reexport() {
        let smoothed = funnel_smooth(&[Vec2::ZERO, Vec2::new(1.0, 1.0)]);
        assert_eq!(smoothed.len(), 2);
    }
}
