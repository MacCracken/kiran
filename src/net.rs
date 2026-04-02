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
#[non_exhaustive]
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

/// Message reliability mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Reliability {
    /// Fire-and-forget — no retransmit, no ordering. Use for state updates.
    Unreliable,
    /// Guaranteed delivery with ordering. Use for RPCs, events.
    Reliable,
}

/// A network message sent between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum NetMessage {
    /// State snapshot — full entity positions.
    StateSnapshot(StateSnapshot),
    /// State delta — only changed entities.
    StateDelta(StateDelta),
    /// Input from a client.
    InputReplication(InputMessage),
    /// Player joined.
    PlayerJoin {
        /// ID of the joining node.
        node_id: NodeId,
    },
    /// Player left.
    PlayerLeave {
        /// ID of the leaving node.
        node_id: NodeId,
    },
}

/// Full state snapshot of all replicated entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Server tick when this snapshot was taken.
    pub tick: u64,
    /// Entity states included in the snapshot.
    pub entities: Vec<EntityState>,
}

/// State of a single entity for network sync.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityState {
    /// Unique entity identifier.
    pub entity_id: u64,
    /// World-space position.
    pub position: [f32; 3],
    /// Owning network node, if any.
    pub owner: Option<NodeId>,
}

/// Delta update — only changed entities since last snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateDelta {
    /// Tick of the base snapshot this delta is relative to.
    pub base_tick: u64,
    /// Tick of the new state.
    pub tick: u64,
    /// Only the entities that changed since base_tick.
    pub changes: Vec<EntityState>,
}

/// Replicated input from a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
    /// Node that sent this input.
    pub node_id: NodeId,
    /// Tick the input applies to.
    pub tick: u64,
    /// Serialized input data.
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
    #[must_use]
    #[inline]
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Get connected peer IDs.
    #[must_use]
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
    #[must_use]
    #[inline]
    pub fn is_server(&self) -> bool {
        self.role == NetRole::Server
    }

    /// Is this node a client?
    #[must_use]
    #[inline]
    pub fn is_client(&self) -> bool {
        self.role == NetRole::Client
    }
}

// ---------------------------------------------------------------------------
// Reliable channel
// ---------------------------------------------------------------------------

/// A tagged message with reliability and sequence number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedMessage {
    /// The network message payload.
    pub message: NetMessage,
    /// Delivery guarantee for this message.
    pub reliability: Reliability,
    /// Monotonic sequence number.
    pub sequence: u64,
}

/// Reliable channel — tracks outbound messages and handles acks/retransmit.
pub struct ReliableChannel {
    next_sequence: u64,
    /// Unacknowledged reliable messages (sequence → message).
    pending_ack: std::collections::HashMap<u64, TaggedMessage>,
    /// Received reliable sequence numbers (for dedup).
    received: std::collections::HashSet<u64>,
}

impl Default for ReliableChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl ReliableChannel {
    /// Create a new reliable channel.
    pub fn new() -> Self {
        Self {
            next_sequence: 1,
            pending_ack: std::collections::HashMap::new(),
            received: std::collections::HashSet::new(),
        }
    }

    /// Send a message with the specified reliability.
    pub fn send(&mut self, message: NetMessage, reliability: Reliability) -> TaggedMessage {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        let tagged = TaggedMessage {
            message,
            reliability,
            sequence: seq,
        };
        if reliability == Reliability::Reliable {
            self.pending_ack.insert(seq, tagged.clone());
        }
        tagged
    }

    /// Acknowledge receipt of a reliable message.
    pub fn ack(&mut self, sequence: u64) {
        self.pending_ack.remove(&sequence);
    }

    /// Get messages that need retransmitting (unacked reliable messages).
    pub fn pending_retransmit(&self) -> Vec<&TaggedMessage> {
        self.pending_ack.values().collect()
    }

    /// Check if an incoming reliable message is a duplicate.
    /// Returns true if this is a new message (not seen before).
    pub fn receive(&mut self, sequence: u64) -> bool {
        self.received.insert(sequence)
    }

    /// Number of unacknowledged reliable messages.
    #[must_use]
    #[inline]
    pub fn pending_count(&self) -> usize {
        self.pending_ack.len()
    }

    /// Trim received dedup set — discard sequences below a threshold.
    /// Call periodically to prevent unbounded memory growth.
    pub fn trim_received(&mut self, min_sequence: u64) {
        self.received.retain(|&seq| seq >= min_sequence);
    }
}

// ---------------------------------------------------------------------------
// Interest management
// ---------------------------------------------------------------------------

/// Area-of-interest filter — determines which entities to replicate to each client.
pub struct InterestArea {
    /// Center position of the interest area.
    pub center: [f32; 3],
    /// Radius of interest (entities beyond this are not replicated).
    pub radius: f32,
}

impl InterestArea {
    /// Create an interest area centered at `center` with the given `radius`.
    pub fn new(center: [f32; 3], radius: f32) -> Self {
        Self { center, radius }
    }

    /// Check if a position is within the interest area.
    #[must_use]
    #[inline]
    pub fn contains(&self, position: [f32; 3]) -> bool {
        let dx = position[0] - self.center[0];
        let dy = position[1] - self.center[1];
        let dz = position[2] - self.center[2];
        (dx * dx + dy * dy + dz * dz) <= self.radius * self.radius
    }
}

/// Build a snapshot filtered by interest area — only entities within range.
pub fn build_snapshot_filtered(
    world: &World,
    tick: u64,
    entities: &[Entity],
    interest: &InterestArea,
) -> StateSnapshot {
    use crate::scene::Position;

    let mut states = Vec::new();
    for &entity in entities {
        if !world.is_alive(entity) {
            continue;
        }
        let position = world
            .get_component::<Position>(entity)
            .map(|p| [p.0.x, p.0.y, p.0.z])
            .unwrap_or([0.0, 0.0, 0.0]);

        if !interest.contains(position) {
            continue;
        }

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

// ---------------------------------------------------------------------------
// Interpolation
// ---------------------------------------------------------------------------

/// Interpolation state for smoothing network updates.
#[derive(Debug, Clone)]
pub struct NetInterpolation {
    /// Position at the start of interpolation.
    pub previous: [f32; 3],
    /// Target position from latest network update.
    pub target: [f32; 3],
    /// Interpolation progress (0.0–1.0).
    pub alpha: f32,
}

impl Default for NetInterpolation {
    fn default() -> Self {
        Self {
            previous: [0.0; 3],
            target: [0.0; 3],
            alpha: 1.0,
        }
    }
}

impl NetInterpolation {
    /// Set a new target position (call when network state arrives).
    pub fn set_target(&mut self, target: [f32; 3]) {
        self.previous = self.current();
        self.target = target;
        self.alpha = 0.0;
    }

    /// Advance interpolation by dt (in fraction of interpolation period).
    pub fn advance(&mut self, dt: f32) {
        self.alpha = (self.alpha + dt).min(1.0);
    }

    /// Get the current interpolated position.
    #[must_use]
    #[inline]
    pub fn current(&self) -> [f32; 3] {
        [
            self.previous[0] + (self.target[0] - self.previous[0]) * self.alpha,
            self.previous[1] + (self.target[1] - self.previous[1]) * self.alpha,
            self.previous[2] + (self.target[2] - self.previous[2]) * self.alpha,
        ]
    }

    /// Is interpolation complete (alpha >= 1.0)?
    #[must_use]
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.alpha >= 1.0
    }
}

/// Apply snapshot with interpolation instead of snapping.
pub fn apply_snapshot_interpolated(world: &mut World, snapshot: &StateSnapshot) {
    for state in &snapshot.entities {
        let entity = Entity::from_id(state.entity_id);
        if let Some(interp) = world.get_component_mut::<NetInterpolation>(entity) {
            interp.set_target(state.position);
        }
    }
}

/// Advance all interpolations and update positions.
pub fn step_interpolation(world: &mut World, dt: f32) {
    use crate::scene::Position;

    let entities: Vec<Entity> = world
        .query::<NetInterpolation>()
        .iter()
        .map(|(e, _)| *e)
        .collect();

    for entity in entities {
        if let Some(interp) = world.get_component_mut::<NetInterpolation>(entity) {
            interp.advance(dt);
            let pos = interp.current();
            if let Some(position) = world.get_component_mut::<Position>(entity) {
                position.0 = hisab::Vec3::new(pos[0], pos[1], pos[2]);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Client-side prediction
// ---------------------------------------------------------------------------

/// Stores predicted state for rollback.
#[derive(Debug, Clone)]
pub struct PredictionBuffer {
    /// Ring buffer of predicted positions indexed by tick (VecDeque for O(1) eviction).
    history: std::collections::VecDeque<([f32; 3], u64)>,
    /// Maximum history size.
    max_size: usize,
}

impl PredictionBuffer {
    /// Create a prediction buffer with the given capacity.
    pub fn new(max_size: usize) -> Self {
        Self {
            history: std::collections::VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Record a predicted position at a tick.
    pub fn record(&mut self, position: [f32; 3], tick: u64) {
        if self.history.len() >= self.max_size {
            self.history.pop_front();
        }
        self.history.push_back((position, tick));
    }

    /// Get the predicted position at a tick (for server reconciliation).
    #[must_use]
    pub fn at_tick(&self, tick: u64) -> Option<[f32; 3]> {
        self.history
            .iter()
            .find(|(_, t)| *t == tick)
            .map(|(pos, _)| *pos)
    }

    /// Check if server state diverges from our prediction at a tick.
    /// Returns the error magnitude if they differ.
    pub fn check_prediction(&self, server_pos: [f32; 3], tick: u64, threshold: f32) -> Option<f32> {
        let predicted = self.at_tick(tick)?;
        let dx = server_pos[0] - predicted[0];
        let dy = server_pos[1] - predicted[1];
        let dz = server_pos[2] - predicted[2];
        let error = (dx * dx + dy * dy + dz * dz).sqrt();
        if error > threshold { Some(error) } else { None }
    }

    /// Discard predictions older than a tick.
    pub fn discard_before(&mut self, tick: u64) {
        self.history.retain(|(_, t)| *t >= tick);
    }

    /// Number of stored predictions.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Returns true if no predictions are stored.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Component-generic replication
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Clock synchronization
// ---------------------------------------------------------------------------

/// Simple clock sync — tracks offset between local and server time.
#[derive(Debug, Clone)]
pub struct ClockSync {
    /// Estimated offset: server_time = local_time + offset.
    pub offset_ms: f64,
    /// Round-trip time estimate.
    pub rtt_ms: f64,
    /// Number of samples taken.
    pub samples: u32,
}

impl Default for ClockSync {
    fn default() -> Self {
        Self {
            offset_ms: 0.0,
            rtt_ms: 0.0,
            samples: 0,
        }
    }
}

impl ClockSync {
    /// Create a new clock sync with zero offset.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a ping/pong measurement.
    /// `local_send`: local timestamp when ping was sent.
    /// `server_time`: server timestamp from the pong.
    /// `local_recv`: local timestamp when pong was received.
    pub fn record_sample(&mut self, local_send: f64, server_time: f64, local_recv: f64) {
        let rtt = local_recv - local_send;
        let estimated_server_now = server_time + rtt / 2.0;
        let offset = estimated_server_now - local_recv;

        // Exponential moving average
        let alpha = if self.samples == 0 { 1.0 } else { 0.1 };
        self.offset_ms = self.offset_ms * (1.0 - alpha) + offset * alpha;
        self.rtt_ms = self.rtt_ms * (1.0 - alpha) + rtt * alpha;
        self.samples += 1;
    }

    /// Convert local time to estimated server time.
    #[must_use]
    #[inline]
    pub fn to_server_time(&self, local_ms: f64) -> f64 {
        local_ms + self.offset_ms
    }

    /// Convert server time to estimated local time.
    #[must_use]
    #[inline]
    pub fn to_local_time(&self, server_ms: f64) -> f64 {
        server_ms - self.offset_ms
    }
}

// ---------------------------------------------------------------------------
// Component-generic replication
// ---------------------------------------------------------------------------

/// Serialize a component to JSON for network replication.
pub fn serialize_component<T: serde::Serialize + 'static + Send + Sync>(
    world: &World,
    entity: Entity,
) -> Option<String> {
    let component = world.get_component::<T>(entity)?;
    serde_json::to_string(component).ok()
}

/// Deserialize and apply a component from JSON.
pub fn apply_replicated_component<T: serde::de::DeserializeOwned + 'static + Send + Sync>(
    world: &mut World,
    entity: Entity,
    json: &str,
) -> bool {
    if let Ok(component) = serde_json::from_str::<T>(json) {
        world.insert_component(entity, component).is_ok()
    } else {
        false
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

    #[test]
    fn full_snapshot_delta_apply_cycle() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world
            .insert_component(e1, Position(hisab::Vec3::new(0.0, 0.0, 0.0)))
            .unwrap();
        world
            .insert_component(e2, Position(hisab::Vec3::new(10.0, 0.0, 0.0)))
            .unwrap();

        // Tick 1: snapshot
        let snap1 = build_snapshot(&world, 1, &[e1, e2]);

        // Simulate movement
        world.get_component_mut::<Position>(e1).unwrap().0.x = 5.0;

        // Tick 2: snapshot + delta
        let snap2 = build_snapshot(&world, 2, &[e1, e2]);
        let delta = build_delta(&snap1, &snap2);

        assert_eq!(delta.changes.len(), 1);
        assert_eq!(delta.changes[0].entity_id, e1.id());

        // Apply delta to a fresh world with same entities
        let mut world2 = World::new();
        let e1b = world2.spawn();
        let e2b = world2.spawn();
        world2
            .insert_component(e1b, Position(hisab::Vec3::ZERO))
            .unwrap();
        world2
            .insert_component(e2b, Position(hisab::Vec3::new(10.0, 0.0, 0.0)))
            .unwrap();

        apply_delta(&mut world2, &delta);

        assert_eq!(world2.get_component::<Position>(e1b).unwrap().0.x, 5.0);
        assert_eq!(world2.get_component::<Position>(e2b).unwrap().0.x, 10.0);
    }

    #[test]
    fn build_delta_large_hashmap_path() {
        // >256 entities triggers HashMap path
        let old = StateSnapshot {
            tick: 1,
            entities: (0..300)
                .map(|i| EntityState {
                    entity_id: i,
                    position: [i as f32, 0.0, 0.0],
                    owner: None,
                })
                .collect(),
        };
        let mut new_entities: Vec<EntityState> = (0..300)
            .map(|i| EntityState {
                entity_id: i,
                position: [i as f32, 0.0, 0.0],
                owner: None,
            })
            .collect();
        // Change 10 entities
        for i in 0..10 {
            new_entities[i * 30].position[0] += 100.0;
        }
        let new = StateSnapshot {
            tick: 2,
            entities: new_entities,
        };

        let delta = build_delta(&old, &new);
        assert_eq!(delta.changes.len(), 10);
    }

    // -- Interpolation tests --

    #[test]
    fn interpolation_basic() {
        let mut interp = NetInterpolation::default();
        interp.set_target([10.0, 0.0, 0.0]);

        assert_eq!(interp.current(), [0.0, 0.0, 0.0]); // alpha=0
        interp.advance(0.5);
        assert_eq!(interp.current(), [5.0, 0.0, 0.0]); // halfway
        interp.advance(0.5);
        assert_eq!(interp.current(), [10.0, 0.0, 0.0]); // complete
        assert!(interp.is_complete());
    }

    #[test]
    fn interpolation_clamps() {
        let mut interp = NetInterpolation::default();
        interp.set_target([10.0, 0.0, 0.0]);
        interp.advance(2.0); // overshoot
        assert_eq!(interp.alpha, 1.0);
        assert_eq!(interp.current(), [10.0, 0.0, 0.0]);
    }

    #[test]
    fn interpolation_retarget() {
        let mut interp = NetInterpolation::default();
        interp.set_target([10.0, 0.0, 0.0]);
        interp.advance(0.5); // at [5, 0, 0]

        interp.set_target([20.0, 0.0, 0.0]);
        assert_eq!(interp.alpha, 0.0);
        assert_eq!(interp.current(), [5.0, 0.0, 0.0]); // previous is [5,0,0]
    }

    // -- Prediction tests --

    #[test]
    fn prediction_buffer_record_and_lookup() {
        let mut buf = PredictionBuffer::new(100);
        buf.record([1.0, 0.0, 0.0], 1);
        buf.record([2.0, 0.0, 0.0], 2);
        buf.record([3.0, 0.0, 0.0], 3);

        assert_eq!(buf.at_tick(2), Some([2.0, 0.0, 0.0]));
        assert_eq!(buf.at_tick(99), None);
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn prediction_buffer_overflow() {
        let mut buf = PredictionBuffer::new(3);
        buf.record([1.0, 0.0, 0.0], 1);
        buf.record([2.0, 0.0, 0.0], 2);
        buf.record([3.0, 0.0, 0.0], 3);
        buf.record([4.0, 0.0, 0.0], 4); // oldest evicted

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.at_tick(1), None); // evicted
        assert_eq!(buf.at_tick(4), Some([4.0, 0.0, 0.0]));
    }

    #[test]
    fn prediction_check_within_threshold() {
        let mut buf = PredictionBuffer::new(10);
        buf.record([10.0, 0.0, 0.0], 5);

        // Server agrees within threshold
        assert!(buf.check_prediction([10.01, 0.0, 0.0], 5, 0.1).is_none());
        // Server disagrees beyond threshold
        let error = buf.check_prediction([15.0, 0.0, 0.0], 5, 0.1);
        assert!(error.is_some());
        assert!(error.unwrap() > 4.0);
    }

    #[test]
    fn prediction_discard_old() {
        let mut buf = PredictionBuffer::new(10);
        buf.record([1.0, 0.0, 0.0], 1);
        buf.record([2.0, 0.0, 0.0], 2);
        buf.record([3.0, 0.0, 0.0], 3);
        buf.discard_before(2);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.at_tick(1), None);
    }

    // -- Replication tests --

    #[test]
    fn serialize_replicate_component() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(e, Position(hisab::Vec3::new(1.0, 2.0, 3.0)))
            .unwrap();

        // Position doesn't implement Serialize — use NetOwner which does
        world
            .insert_component(e, NetOwner("player-1".into()))
            .unwrap();

        let json = serialize_component::<NetOwner>(&world, e).unwrap();
        assert!(json.contains("player-1"));

        // Apply to another world
        let mut world2 = World::new();
        let e2 = world2.spawn();
        assert!(apply_replicated_component::<NetOwner>(
            &mut world2,
            e2,
            &json
        ));
        assert_eq!(world2.get_component::<NetOwner>(e2).unwrap().0, "player-1");
    }

    // -- Step interpolation end-to-end --

    #[test]
    fn step_interpolation_updates_position() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(e, Position(hisab::Vec3::ZERO))
            .unwrap();
        world
            .insert_component(e, NetInterpolation::default())
            .unwrap();

        // Set target
        {
            let interp = world.get_component_mut::<NetInterpolation>(e).unwrap();
            interp.set_target([10.0, 0.0, 0.0]);
        }

        // Step halfway
        step_interpolation(&mut world, 0.5);

        let pos = world.get_component::<Position>(e).unwrap();
        assert!((pos.0.x - 5.0).abs() < 0.01);

        // Step to completion
        step_interpolation(&mut world, 0.5);

        let pos = world.get_component::<Position>(e).unwrap();
        assert!((pos.0.x - 10.0).abs() < 0.01);
    }

    #[test]
    fn prediction_buffer_as_component() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(e, PredictionBuffer::new(64))
            .unwrap();

        let buf = world.get_component_mut::<PredictionBuffer>(e).unwrap();
        buf.record([1.0, 0.0, 0.0], 1);
        assert_eq!(buf.len(), 1);
    }

    // -- Reliable channel tests --

    #[test]
    fn reliable_channel_send_unreliable() {
        let mut ch = ReliableChannel::new();
        let msg = ch.send(
            NetMessage::PlayerJoin {
                node_id: "p1".into(),
            },
            Reliability::Unreliable,
        );
        assert_eq!(msg.reliability, Reliability::Unreliable);
        assert_eq!(ch.pending_count(), 0); // not tracked
    }

    #[test]
    fn reliable_channel_send_reliable() {
        let mut ch = ReliableChannel::new();
        let msg = ch.send(
            NetMessage::PlayerJoin {
                node_id: "p1".into(),
            },
            Reliability::Reliable,
        );
        assert_eq!(msg.reliability, Reliability::Reliable);
        assert_eq!(ch.pending_count(), 1); // tracked for ack
    }

    #[test]
    fn reliable_channel_ack() {
        let mut ch = ReliableChannel::new();
        let msg = ch.send(
            NetMessage::PlayerLeave {
                node_id: "p1".into(),
            },
            Reliability::Reliable,
        );
        assert_eq!(ch.pending_count(), 1);

        ch.ack(msg.sequence);
        assert_eq!(ch.pending_count(), 0);
    }

    #[test]
    fn reliable_channel_dedup() {
        let mut ch = ReliableChannel::new();
        assert!(ch.receive(1)); // new
        assert!(!ch.receive(1)); // duplicate
        assert!(ch.receive(2)); // new
    }

    #[test]
    fn reliable_channel_retransmit() {
        let mut ch = ReliableChannel::new();
        ch.send(
            NetMessage::PlayerJoin {
                node_id: "a".into(),
            },
            Reliability::Reliable,
        );
        ch.send(
            NetMessage::PlayerJoin {
                node_id: "b".into(),
            },
            Reliability::Reliable,
        );

        let pending = ch.pending_retransmit();
        assert_eq!(pending.len(), 2);
    }

    // -- Interest management tests --

    #[test]
    fn interest_area_contains() {
        let area = InterestArea::new([0.0, 0.0, 0.0], 10.0);
        assert!(area.contains([5.0, 0.0, 0.0]));
        assert!(area.contains([0.0, 0.0, 0.0]));
        assert!(!area.contains([15.0, 0.0, 0.0]));
    }

    #[test]
    fn interest_area_boundary() {
        let area = InterestArea::new([0.0, 0.0, 0.0], 10.0);
        // Exactly on boundary
        assert!(area.contains([10.0, 0.0, 0.0]));
        // Just outside
        assert!(!area.contains([10.1, 0.0, 0.0]));
    }

    #[test]
    fn build_snapshot_filtered_test() {
        let mut world = World::new();
        let near = world.spawn();
        let far = world.spawn();
        world
            .insert_component(near, Position(hisab::Vec3::new(5.0, 0.0, 0.0)))
            .unwrap();
        world
            .insert_component(far, Position(hisab::Vec3::new(100.0, 0.0, 0.0)))
            .unwrap();

        let interest = InterestArea::new([0.0, 0.0, 0.0], 20.0);
        let snapshot = build_snapshot_filtered(&world, 1, &[near, far], &interest);

        assert_eq!(snapshot.entities.len(), 1); // only near entity
        assert_eq!(snapshot.entities[0].entity_id, near.id());
    }

    #[test]
    fn build_snapshot_filtered_all_in_range() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world
            .insert_component(e1, Position(hisab::Vec3::new(1.0, 0.0, 0.0)))
            .unwrap();
        world
            .insert_component(e2, Position(hisab::Vec3::new(2.0, 0.0, 0.0)))
            .unwrap();

        let interest = InterestArea::new([0.0, 0.0, 0.0], 100.0);
        let snapshot = build_snapshot_filtered(&world, 1, &[e1, e2], &interest);
        assert_eq!(snapshot.entities.len(), 2);
    }

    #[test]
    fn reliability_serde() {
        let r = Reliability::Reliable;
        let json = serde_json::to_string(&r).unwrap();
        let decoded: Reliability = serde_json::from_str(&json).unwrap();
        assert_eq!(r, decoded);
    }

    #[test]
    fn tagged_message_serde() {
        let msg = TaggedMessage {
            message: NetMessage::PlayerJoin {
                node_id: "p1".into(),
            },
            reliability: Reliability::Reliable,
            sequence: 42,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: TaggedMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.sequence, 42);
    }
}
