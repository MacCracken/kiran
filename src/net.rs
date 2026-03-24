//! Multiplayer networking via majra
//!
//! Provides networked game state synchronization:
//! - [`NetState`] resource managing node identity and messaging
//! - [`NetOwner`] component marking entity ownership
//! - State snapshots and delta compression
//! - Input replication

use serde::{Deserialize, Serialize};

use crate::world::{Entity, World};

// ---------------------------------------------------------------------------
// Node identity
// ---------------------------------------------------------------------------

/// Unique identifier for a network node (player/server).
pub type NodeId = String;

/// Role of this node in the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NetRole {
    /// Authoritative server — owns game state, validates inputs.
    Server,
    /// Client — sends input, receives state updates.
    #[default]
    Client,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks which network node owns an entity.
/// Only the owner can modify the entity's state; others receive updates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetOwner(pub NodeId);

/// Marks an entity as replicated across the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Replicated;

// ---------------------------------------------------------------------------
// Network messages
// ---------------------------------------------------------------------------

/// A network message sent between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessage {
    /// State snapshot — full entity positions.
    StateSnapshot(StateSnapshot),
    /// State delta — only changed entities.
    StateDelta(StateDelta),
    /// Input from a client.
    InputReplication(InputMessage),
    /// Player joined.
    PlayerJoin { node_id: NodeId },
    /// Player left.
    PlayerLeave { node_id: NodeId },
}

/// Full state snapshot of all replicated entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tick: u64,
    pub entities: Vec<EntityState>,
}

/// State of a single entity for network sync.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityState {
    pub entity_id: u64,
    pub position: [f32; 3],
    pub owner: Option<NodeId>,
}

/// Delta update — only changed entities since last snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateDelta {
    pub base_tick: u64,
    pub tick: u64,
    pub changes: Vec<EntityState>,
}

/// Replicated input from a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
    pub node_id: NodeId,
    pub tick: u64,
    pub payload: String,
}

// ---------------------------------------------------------------------------
// NetState resource
// ---------------------------------------------------------------------------

/// Network state resource — manages multiplayer connections.
pub struct NetState {
    /// This node's ID.
    pub node_id: NodeId,
    /// Role (server or client).
    pub role: NetRole,
    /// majra relay for message routing.
    relay: majra::relay::Relay,
    /// Connected peer node IDs.
    peers: Vec<NodeId>,
    /// Outbound message queue.
    outbox: Vec<NetMessage>,
    /// Inbound message queue.
    inbox: Vec<NetMessage>,
    /// Current network tick.
    pub tick: u64,
}

impl NetState {
    /// Create a new network state.
    pub fn new(node_id: impl Into<String>, role: NetRole) -> Self {
        let node_id = node_id.into();
        let relay = majra::relay::Relay::new(&node_id);
        Self {
            node_id,
            role,
            relay,
            peers: Vec::new(),
            outbox: Vec::new(),
            inbox: Vec::new(),
            tick: 0,
        }
    }

    /// Create a server node.
    pub fn server(node_id: impl Into<String>) -> Self {
        Self::new(node_id, NetRole::Server)
    }

    /// Create a client node.
    pub fn client(node_id: impl Into<String>) -> Self {
        Self::new(node_id, NetRole::Client)
    }

    /// Add a peer node.
    pub fn add_peer(&mut self, peer_id: impl Into<String>) {
        let id = peer_id.into();
        if !self.peers.contains(&id) {
            self.peers.push(id);
        }
    }

    /// Remove a peer node.
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.retain(|p| p != peer_id);
    }

    /// Number of connected peers.
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Get connected peer IDs.
    pub fn peers(&self) -> &[NodeId] {
        &self.peers
    }

    /// Queue a message for sending.
    pub fn send(&mut self, msg: NetMessage) {
        self.outbox.push(msg);
    }

    /// Drain outbound messages.
    pub fn drain_outbox(&mut self) -> Vec<NetMessage> {
        std::mem::take(&mut self.outbox)
    }

    /// Push an inbound message (received from network).
    pub fn receive(&mut self, msg: NetMessage) {
        self.inbox.push(msg);
    }

    /// Drain inbound messages.
    pub fn drain_inbox(&mut self) -> Vec<NetMessage> {
        std::mem::take(&mut self.inbox)
    }

    /// Advance the network tick.
    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    /// Access the underlying majra relay.
    pub fn relay(&self) -> &majra::relay::Relay {
        &self.relay
    }

    /// Broadcast a message to all peers via majra relay.
    pub fn broadcast_via_relay(&self, topic: &str, payload: serde_json::Value) -> u64 {
        self.relay.broadcast(topic, payload)
    }

    /// Is this node the server?
    pub fn is_server(&self) -> bool {
        self.role == NetRole::Server
    }

    /// Is this node a client?
    pub fn is_client(&self) -> bool {
        self.role == NetRole::Client
    }
}

// ---------------------------------------------------------------------------
// State sync helpers
// ---------------------------------------------------------------------------

/// Build a full state snapshot from the world.
pub fn build_snapshot(world: &World, tick: u64, entities: &[Entity]) -> StateSnapshot {
    use crate::scene::Position;

    let mut states = Vec::with_capacity(entities.len());
    for &entity in entities {
        if !world.is_alive(entity) {
            continue;
        }
        let position = world
            .get_component::<Position>(entity)
            .map(|p| [p.0.x, p.0.y, p.0.z])
            .unwrap_or([0.0, 0.0, 0.0]);
        let owner = world.get_component::<NetOwner>(entity).map(|o| o.0.clone());

        states.push(EntityState {
            entity_id: entity.id(),
            position,
            owner,
        });
    }

    StateSnapshot {
        tick,
        entities: states,
    }
}

/// Build a delta from two snapshots (only changed entities).
/// Uses linear scan for small snapshots (<256 entities) and HashMap for larger ones.
pub fn build_delta(old: &StateSnapshot, new: &StateSnapshot) -> StateDelta {
    let mut changes = Vec::new();

    if old.entities.len() < 256 {
        // Linear scan — better cache locality for small entity counts
        for new_state in &new.entities {
            let changed = old
                .entities
                .iter()
                .find(|o| o.entity_id == new_state.entity_id)
                .is_none_or(|old_state| old_state.position != new_state.position);
            if changed {
                changes.push(new_state.clone());
            }
        }
    } else {
        // HashMap for large entity counts — O(n+m) vs O(n*m)
        use std::collections::HashMap;
        let old_map: HashMap<u64, &EntityState> =
            old.entities.iter().map(|e| (e.entity_id, e)).collect();
        for new_state in &new.entities {
            let changed = old_map
                .get(&new_state.entity_id)
                .is_none_or(|old_state| old_state.position != new_state.position);
            if changed {
                changes.push(new_state.clone());
            }
        }
    }

    StateDelta {
        base_tick: old.tick,
        tick: new.tick,
        changes,
    }
}

/// Apply a state snapshot to the world (update entity positions).
pub fn apply_snapshot(world: &mut World, snapshot: &StateSnapshot) {
    use crate::scene::Position;

    for state in &snapshot.entities {
        let entity = Entity::from_id(state.entity_id);
        if let Some(pos) = world.get_component_mut::<Position>(entity) {
            pos.0 = hisab::Vec3::new(state.position[0], state.position[1], state.position[2]);
        }
    }
}

/// Apply a state delta to the world.
pub fn apply_delta(world: &mut World, delta: &StateDelta) {
    use crate::scene::Position;

    for state in &delta.changes {
        let entity = Entity::from_id(state.entity_id);
        if let Some(pos) = world.get_component_mut::<Position>(entity) {
            pos.0 = hisab::Vec3::new(state.position[0], state.position[1], state.position[2]);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Position;

    #[test]
    fn net_state_server() {
        let state = NetState::server("server-1");
        assert!(state.is_server());
        assert!(!state.is_client());
        assert_eq!(state.node_id, "server-1");
        assert_eq!(state.peer_count(), 0);
    }

    #[test]
    fn net_state_client() {
        let state = NetState::client("player-42");
        assert!(state.is_client());
        assert_eq!(state.node_id, "player-42");
    }

    #[test]
    fn net_state_peers() {
        let mut state = NetState::server("server");
        state.add_peer("player-1");
        state.add_peer("player-2");
        assert_eq!(state.peer_count(), 2);

        // Duplicate ignored
        state.add_peer("player-1");
        assert_eq!(state.peer_count(), 2);

        state.remove_peer("player-1");
        assert_eq!(state.peer_count(), 1);
        assert_eq!(state.peers()[0], "player-2");
    }

    #[test]
    fn net_state_messaging() {
        let mut state = NetState::server("server");

        state.send(NetMessage::PlayerJoin {
            node_id: "p1".into(),
        });
        state.send(NetMessage::PlayerJoin {
            node_id: "p2".into(),
        });

        let outbox = state.drain_outbox();
        assert_eq!(outbox.len(), 2);
        assert!(state.drain_outbox().is_empty());
    }

    #[test]
    fn net_state_inbox() {
        let mut state = NetState::client("player");

        state.receive(NetMessage::PlayerJoin {
            node_id: "other".into(),
        });

        let inbox = state.drain_inbox();
        assert_eq!(inbox.len(), 1);
    }

    #[test]
    fn net_state_tick() {
        let mut state = NetState::server("server");
        assert_eq!(state.tick, 0);
        state.advance_tick();
        state.advance_tick();
        assert_eq!(state.tick, 2);
    }

    #[test]
    fn build_snapshot_basic() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world
            .insert_component(e1, Position(hisab::Vec3::new(1.0, 2.0, 3.0)))
            .unwrap();
        world
            .insert_component(e2, Position(hisab::Vec3::new(4.0, 5.0, 6.0)))
            .unwrap();
        world
            .insert_component(e1, NetOwner("player-1".into()))
            .unwrap();

        let snapshot = build_snapshot(&world, 10, &[e1, e2]);
        assert_eq!(snapshot.tick, 10);
        assert_eq!(snapshot.entities.len(), 2);
        assert_eq!(snapshot.entities[0].position, [1.0, 2.0, 3.0]);
        assert_eq!(snapshot.entities[0].owner, Some("player-1".into()));
        assert!(snapshot.entities[1].owner.is_none());
    }

    #[test]
    fn build_delta_detects_changes() {
        let old = StateSnapshot {
            tick: 1,
            entities: vec![
                EntityState {
                    entity_id: 0,
                    position: [0.0, 0.0, 0.0],
                    owner: None,
                },
                EntityState {
                    entity_id: 1,
                    position: [10.0, 0.0, 0.0],
                    owner: None,
                },
            ],
        };
        let new = StateSnapshot {
            tick: 2,
            entities: vec![
                EntityState {
                    entity_id: 0,
                    position: [0.0, 0.0, 0.0], // unchanged
                    owner: None,
                },
                EntityState {
                    entity_id: 1,
                    position: [15.0, 0.0, 0.0], // moved
                    owner: None,
                },
            ],
        };

        let delta = build_delta(&old, &new);
        assert_eq!(delta.base_tick, 1);
        assert_eq!(delta.tick, 2);
        assert_eq!(delta.changes.len(), 1); // only entity 1 changed
        assert_eq!(delta.changes[0].entity_id, 1);
    }

    #[test]
    fn build_delta_new_entity() {
        let old = StateSnapshot {
            tick: 1,
            entities: vec![],
        };
        let new = StateSnapshot {
            tick: 2,
            entities: vec![EntityState {
                entity_id: 42,
                position: [1.0, 2.0, 3.0],
                owner: None,
            }],
        };

        let delta = build_delta(&old, &new);
        assert_eq!(delta.changes.len(), 1);
    }

    #[test]
    fn apply_snapshot_updates_positions() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(e, Position(hisab::Vec3::ZERO))
            .unwrap();

        let snapshot = StateSnapshot {
            tick: 5,
            entities: vec![EntityState {
                entity_id: e.id(),
                position: [10.0, 20.0, 30.0],
                owner: None,
            }],
        };

        apply_snapshot(&mut world, &snapshot);

        let pos = world.get_component::<Position>(e).unwrap();
        assert_eq!(pos.0.x, 10.0);
        assert_eq!(pos.0.y, 20.0);
        assert_eq!(pos.0.z, 30.0);
    }

    #[test]
    fn apply_delta_updates_only_changed() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world
            .insert_component(e1, Position(hisab::Vec3::new(1.0, 0.0, 0.0)))
            .unwrap();
        world
            .insert_component(e2, Position(hisab::Vec3::new(2.0, 0.0, 0.0)))
            .unwrap();

        let delta = StateDelta {
            base_tick: 1,
            tick: 2,
            changes: vec![EntityState {
                entity_id: e2.id(),
                position: [99.0, 0.0, 0.0],
                owner: None,
            }],
        };

        apply_delta(&mut world, &delta);

        // e1 unchanged
        assert_eq!(world.get_component::<Position>(e1).unwrap().0.x, 1.0);
        // e2 updated
        assert_eq!(world.get_component::<Position>(e2).unwrap().0.x, 99.0);
    }

    #[test]
    fn snapshot_serde_roundtrip() {
        let snapshot = StateSnapshot {
            tick: 42,
            entities: vec![EntityState {
                entity_id: 7,
                position: [1.0, 2.0, 3.0],
                owner: Some("player".into()),
            }],
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        let decoded: StateSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.tick, 42);
        assert_eq!(decoded.entities[0].position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn net_message_serde() {
        let msg = NetMessage::InputReplication(InputMessage {
            node_id: "p1".into(),
            tick: 100,
            payload: r#"{"keys": ["W", "Space"]}"#.into(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: NetMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            NetMessage::InputReplication(input) => {
                assert_eq!(input.node_id, "p1");
                assert_eq!(input.tick, 100);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn net_owner_component() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(e, NetOwner("server".into()))
            .unwrap();
        assert_eq!(world.get_component::<NetOwner>(e).unwrap().0, "server");
    }

    #[test]
    fn replicated_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Replicated).unwrap();
        assert!(world.has_component::<Replicated>(e));
    }

    #[test]
    fn relay_accessible() {
        let state = NetState::server("test");
        let relay = state.relay();
        assert_eq!(relay.node_id(), "test");
    }

    #[test]
    fn net_role_default() {
        assert_eq!(NetRole::default(), NetRole::Client);
    }

    #[test]
    fn empty_delta_no_changes() {
        let snap = StateSnapshot {
            tick: 1,
            entities: vec![EntityState {
                entity_id: 0,
                position: [1.0, 2.0, 3.0],
                owner: None,
            }],
        };
        let delta = build_delta(&snap, &snap);
        assert!(delta.changes.is_empty());
    }

    #[test]
    fn delta_with_removed_entity() {
        let old = StateSnapshot {
            tick: 1,
            entities: vec![
                EntityState {
                    entity_id: 0,
                    position: [0.0, 0.0, 0.0],
                    owner: None,
                },
                EntityState {
                    entity_id: 1,
                    position: [1.0, 0.0, 0.0],
                    owner: None,
                },
            ],
        };
        let new = StateSnapshot {
            tick: 2,
            entities: vec![
                EntityState {
                    entity_id: 0,
                    position: [0.0, 0.0, 0.0],
                    owner: None,
                },
                // entity 1 removed
            ],
        };
        let delta = build_delta(&old, &new);
        // No changes — delta only tracks entities present in NEW
        assert!(delta.changes.is_empty());
    }

    #[test]
    fn apply_snapshot_missing_entity_no_panic() {
        let mut world = World::new();
        // Snapshot references entity not in world
        let snapshot = StateSnapshot {
            tick: 1,
            entities: vec![EntityState {
                entity_id: 99999,
                position: [1.0, 2.0, 3.0],
                owner: None,
            }],
        };
        apply_snapshot(&mut world, &snapshot); // should not panic
    }

    #[test]
    fn large_snapshot_1000() {
        let mut world = World::new();
        let mut entities = Vec::new();
        for i in 0..1000 {
            let e = world.spawn();
            world
                .insert_component(e, Position(hisab::Vec3::new(i as f32, 0.0, 0.0)))
                .unwrap();
            entities.push(e);
        }

        let snapshot = build_snapshot(&world, 42, &entities);
        assert_eq!(snapshot.entities.len(), 1000);
        assert_eq!(snapshot.tick, 42);
    }

    #[test]
    fn net_state_as_world_resource() {
        let mut world = World::new();
        world.insert_resource(NetState::server("game-server"));

        let state = world.get_resource::<NetState>().unwrap();
        assert_eq!(state.node_id, "game-server");
        assert!(state.is_server());

        let state = world.get_resource_mut::<NetState>().unwrap();
        state.add_peer("player-1");
        assert_eq!(state.peer_count(), 1);
    }

    #[test]
    fn entity_state_partial_eq() {
        let a = EntityState {
            entity_id: 1,
            position: [1.0, 2.0, 3.0],
            owner: None,
        };
        let b = EntityState {
            entity_id: 1,
            position: [1.0, 2.0, 3.0],
            owner: None,
        };
        let c = EntityState {
            entity_id: 1,
            position: [4.0, 5.0, 6.0],
            owner: None,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn input_message_serde() {
        let msg = InputMessage {
            node_id: "p1".into(),
            tick: 42,
            payload: r#"{"w":true}"#.into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: InputMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.node_id, "p1");
        assert_eq!(decoded.tick, 42);
    }

    #[test]
    fn delta_serde_roundtrip() {
        let delta = StateDelta {
            base_tick: 1,
            tick: 2,
            changes: vec![EntityState {
                entity_id: 5,
                position: [10.0, 20.0, 30.0],
                owner: Some("server".into()),
            }],
        };
        let json = serde_json::to_string(&delta).unwrap();
        let decoded: StateDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(delta, decoded);
    }

    #[test]
    fn broadcast_via_relay() {
        let state = NetState::server("test");
        // broadcast returns sequence number
        let seq = state.broadcast_via_relay("game.state", serde_json::json!({"tick": 1}));
        assert!(seq > 0);
    }
}
