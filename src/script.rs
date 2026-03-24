//! Scripting integration via kavach
//!
//! Provides sandboxed WASM script execution for game logic:
//! - [`ScriptEngine`] resource managing kavach sandbox lifecycle
//! - [`Script`] component attaching a script to an entity
//! - [`ScriptMessage`] for passing data between scripts and the engine

use serde::{Deserialize, Serialize};

use crate::world::{Entity, World};

// ---------------------------------------------------------------------------
// Script component
// ---------------------------------------------------------------------------

/// A script attached to an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    /// Path to the WASM script file.
    pub source: String,
    /// Whether this script is enabled.
    pub enabled: bool,
    /// Script-local state as JSON string (round-tripped to WASM).
    #[serde(default)]
    pub state: String,
}

impl Script {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            enabled: true,
            state: String::new(),
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }
}

// ---------------------------------------------------------------------------
// Script messages
// ---------------------------------------------------------------------------

/// A message passed between the engine and a script.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptMessage {
    /// Source entity that sent the message.
    pub sender: Option<u64>,
    /// Target entity (None = broadcast).
    pub target: Option<u64>,
    /// Message type identifier.
    pub kind: String,
    /// Payload as JSON string.
    pub payload: String,
}

impl ScriptMessage {
    pub fn new(kind: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            sender: None,
            target: None,
            kind: kind.into(),
            payload: payload.into(),
        }
    }

    pub fn from_entity(mut self, entity: Entity) -> Self {
        self.sender = Some(entity.id());
        self
    }

    pub fn to_entity(mut self, entity: Entity) -> Self {
        self.target = Some(entity.id());
        self
    }
}

// ---------------------------------------------------------------------------
// Script engine resource
// ---------------------------------------------------------------------------

/// Script execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptConfig {
    /// Maximum execution time per script call (ms).
    pub timeout_ms: u64,
    /// Maximum memory per script (bytes).
    pub max_memory: usize,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 16,               // ~1 frame at 60fps
            max_memory: 16 * 1024 * 1024, // 16 MB
        }
    }
}

/// The script engine resource — manages kavach sandboxes for WASM execution.
pub struct ScriptEngine {
    config: ScriptConfig,
    /// Pending outbound messages from scripts.
    outbox: Vec<ScriptMessage>,
    /// Pending inbound messages to scripts.
    inbox: Vec<ScriptMessage>,
    /// Number of scripts executed this frame.
    exec_count: u64,
    /// Kavach WASM backend (if scripting feature enabled).
    #[cfg(feature = "scripting")]
    wasm: Option<kavach::backend::wasm::WasmBackend>,
    /// Reusable tokio runtime for async kavach calls (created once).
    #[cfg(feature = "scripting")]
    rt: Option<tokio::runtime::Runtime>,
}

impl ScriptEngine {
    pub fn new(config: ScriptConfig) -> Self {
        #[cfg(feature = "scripting")]
        let wasm = {
            let sandbox_config = kavach::SandboxConfig::builder()
                .backend(kavach::Backend::Wasm)
                .timeout_ms(config.timeout_ms)
                .build();
            kavach::backend::wasm::WasmBackend::new(&sandbox_config).ok()
        };

        #[cfg(feature = "scripting")]
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .ok();

        Self {
            config,
            outbox: Vec::new(),
            inbox: Vec::new(),
            exec_count: 0,
            #[cfg(feature = "scripting")]
            wasm,
            #[cfg(feature = "scripting")]
            rt,
        }
    }

    /// Check if the WASM backend is available.
    pub fn wasm_available(&self) -> bool {
        #[cfg(feature = "scripting")]
        {
            self.wasm.is_some()
        }
        #[cfg(not(feature = "scripting"))]
        {
            false
        }
    }

    /// Execute a WASM file and return the result.
    /// Returns None if scripting feature is disabled or WASM backend unavailable.
    #[cfg(feature = "scripting")]
    pub fn exec_wasm(&self, wasm_path: &str) -> Option<kavach::ExecResult> {
        use kavach::backend::SandboxBackend;

        let backend = self.wasm.as_ref()?;
        let rt = self.rt.as_ref()?;
        let policy = kavach::SandboxPolicy::minimal();

        rt.block_on(backend.exec(wasm_path, &policy)).ok()
    }

    /// Send a message to a script.
    pub fn send(&mut self, msg: ScriptMessage) {
        self.inbox.push(msg);
    }

    /// Drain outbound messages from scripts.
    pub fn drain_outbox(&mut self) -> Vec<ScriptMessage> {
        std::mem::take(&mut self.outbox)
    }

    /// Drain inbound messages (consumed by script execution).
    pub fn drain_inbox(&mut self) -> Vec<ScriptMessage> {
        std::mem::take(&mut self.inbox)
    }

    /// Number of scripts executed this frame.
    pub fn exec_count(&self) -> u64 {
        self.exec_count
    }

    /// Reset per-frame counters.
    pub fn begin_frame(&mut self) {
        self.exec_count = 0;
    }

    /// Get the script configuration.
    pub fn config(&self) -> &ScriptConfig {
        &self.config
    }

    /// Record a script execution (called by the script runner system).
    pub fn record_exec(&mut self) {
        self.exec_count += 1;
    }

    /// Push a message to the outbox (from script execution).
    pub fn push_outbox(&mut self, msg: ScriptMessage) {
        self.outbox.push(msg);
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new(ScriptConfig::default())
    }
}

/// Collect all entities with enabled scripts and run them.
///
/// With `scripting` feature: executes WASM modules via kavach for entities
/// that have a `.wasm` source file. Falls back to message-based state update
/// for entities without WASM files.
///
/// Without `scripting` feature: delivers messages to script state directly.
pub fn run_scripts(world: &mut World) {
    let Some(engine) = world.get_resource_mut::<ScriptEngine>() else {
        return;
    };
    engine.begin_frame();
    let messages = engine.drain_inbox();

    // Process inbound messages — update script state
    for msg in &messages {
        let Some(target_id) = msg.target else {
            continue;
        };
        let target = Entity::from_id(target_id);

        // Read script info without holding a mutable borrow
        let script_info = world
            .get_component::<Script>(target)
            .map(|s| (s.enabled, s.source.ends_with(".wasm"), s.source.clone()));
        let Some((enabled, is_wasm, source)) = script_info else {
            continue;
        };
        if !enabled {
            continue;
        }

        // Try WASM execution for .wasm sources
        #[cfg(feature = "scripting")]
        if is_wasm {
            let wasm_result = world
                .get_resource::<ScriptEngine>()
                .and_then(|e| e.exec_wasm(&source));
            if let Some(result) = wasm_result {
                if let Some(script) = world.get_component_mut::<Script>(target)
                    && !result.stdout.is_empty()
                {
                    script.state = result.stdout;
                }
                if let Some(engine) = world.get_resource_mut::<ScriptEngine>() {
                    engine.record_exec();
                }
                continue;
            }
        }

        // Fallback: store message payload as script state
        if let Some(script) = world.get_component_mut::<Script>(target) {
            script.state = msg.payload.clone();
        }
        if let Some(engine) = world.get_resource_mut::<ScriptEngine>() {
            engine.record_exec();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_builder() {
        let s = Script::new("scripts/player.wasm").with_state(r#"{"hp": 100}"#);
        assert_eq!(s.source, "scripts/player.wasm");
        assert!(s.enabled);
        assert_eq!(s.state, r#"{"hp": 100}"#);
    }

    #[test]
    fn script_disabled() {
        let s = Script::new("test.wasm").disabled();
        assert!(!s.enabled);
    }

    #[test]
    fn script_message() {
        let msg = ScriptMessage::new("damage", r#"{"amount": 50}"#);
        assert_eq!(msg.kind, "damage");
        assert!(msg.sender.is_none());
        assert!(msg.target.is_none());
    }

    #[test]
    fn script_message_routing() {
        let sender = Entity::new(1, 0);
        let target = Entity::new(2, 0);
        let msg = ScriptMessage::new("heal", r#"{"amount": 25}"#)
            .from_entity(sender)
            .to_entity(target);
        assert_eq!(msg.sender, Some(sender.id()));
        assert_eq!(msg.target, Some(target.id()));
    }

    #[test]
    fn script_message_serde() {
        let msg = ScriptMessage::new("test", "{}");
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: ScriptMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn script_config_default() {
        let cfg = ScriptConfig::default();
        assert_eq!(cfg.timeout_ms, 16);
        assert_eq!(cfg.max_memory, 16 * 1024 * 1024);
    }

    #[test]
    fn script_engine_messaging() {
        let mut engine = ScriptEngine::default();

        engine.send(ScriptMessage::new("init", "{}"));
        engine.send(ScriptMessage::new("update", r#"{"dt": 0.016}"#));

        let inbox = engine.drain_inbox();
        assert_eq!(inbox.len(), 2);
        assert_eq!(inbox[0].kind, "init");

        // Inbox is now empty
        assert!(engine.drain_inbox().is_empty());
    }

    #[test]
    fn script_engine_outbox() {
        let mut engine = ScriptEngine::default();
        engine.push_outbox(ScriptMessage::new("spawn", r#"{"entity": "bullet"}"#));

        let outbox = engine.drain_outbox();
        assert_eq!(outbox.len(), 1);
        assert!(engine.drain_outbox().is_empty());
    }

    #[test]
    fn script_engine_frame_counter() {
        let mut engine = ScriptEngine::default();
        engine.begin_frame();
        assert_eq!(engine.exec_count(), 0);

        engine.record_exec();
        engine.record_exec();
        assert_eq!(engine.exec_count(), 2);

        engine.begin_frame();
        assert_eq!(engine.exec_count(), 0);
    }

    #[test]
    fn script_as_component() {
        let mut world = World::new();
        let e = world.spawn();
        world
            .insert_component(e, Script::new("ai/npc.wasm"))
            .unwrap();

        assert!(world.has_component::<Script>(e));
        let s = world.get_component::<Script>(e).unwrap();
        assert_eq!(s.source, "ai/npc.wasm");
    }

    #[test]
    fn run_scripts_delivers_messages() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let entity = world.spawn();
        world
            .insert_component(entity, Script::new("test.wasm").with_state("initial"))
            .unwrap();

        // Send message to entity
        engine.send(ScriptMessage::new("update", "new_state").to_entity(entity));

        world.insert_resource(engine);
        run_scripts(&mut world);

        let script = world.get_component::<Script>(entity).unwrap();
        assert_eq!(script.state, "new_state");
    }

    #[test]
    fn run_scripts_increments_exec_count() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let entity = world.spawn();
        world
            .insert_component(entity, Script::new("test.wasm"))
            .unwrap();

        engine.send(ScriptMessage::new("update", "data").to_entity(entity));

        world.insert_resource(engine);
        run_scripts(&mut world);

        let engine = world.get_resource::<ScriptEngine>().unwrap();
        assert_eq!(engine.exec_count(), 1);
    }

    #[test]
    fn script_serde_roundtrip() {
        let script = Script::new("game.wasm").with_state(r#"{"level": 5}"#);
        let json = serde_json::to_string(&script).unwrap();
        let decoded: Script = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.source, "game.wasm");
        assert_eq!(decoded.state, r#"{"level": 5}"#);
        assert!(decoded.enabled);
    }

    #[test]
    fn broadcast_message_not_delivered() {
        // Broadcast (target=None) is not delivered to specific entities
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let entity = world.spawn();
        world
            .insert_component(entity, Script::new("test.wasm").with_state("original"))
            .unwrap();

        // No target — broadcast
        engine.send(ScriptMessage::new("broadcast", "data"));

        world.insert_resource(engine);
        run_scripts(&mut world);

        let script = world.get_component::<Script>(entity).unwrap();
        assert_eq!(script.state, "original"); // unchanged
    }

    #[test]
    fn recycled_entity_message_delivery() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        // Spawn, despawn, respawn to get generation > 0
        let e1 = world.spawn();
        world.despawn(e1).unwrap();
        let e2 = world.spawn(); // same index, generation 1
        assert_eq!(e2.generation(), 1);

        world
            .insert_component(e2, Script::new("test.wasm").with_state("initial"))
            .unwrap();

        // Message targets the recycled entity using its full id
        engine.send(ScriptMessage::new("update", "recycled_data").to_entity(e2));

        world.insert_resource(engine);
        run_scripts(&mut world);

        let script = world.get_component::<Script>(e2).unwrap();
        assert_eq!(script.state, "recycled_data");
    }

    #[test]
    fn run_scripts_skips_disabled() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let entity = world.spawn();
        world
            .insert_component(
                entity,
                Script::new("test.wasm").disabled().with_state("original"),
            )
            .unwrap();

        engine.send(ScriptMessage::new("update", "should_not_apply").to_entity(entity));

        world.insert_resource(engine);
        run_scripts(&mut world);

        let script = world.get_component::<Script>(entity).unwrap();
        assert_eq!(script.state, "original");
    }

    #[test]
    fn multiple_messages_same_frame() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let entity = world.spawn();
        world
            .insert_component(entity, Script::new("test.wasm").with_state("initial"))
            .unwrap();

        // Send multiple messages — last one wins
        engine.send(ScriptMessage::new("update", "first").to_entity(entity));
        engine.send(ScriptMessage::new("update", "second").to_entity(entity));
        engine.send(ScriptMessage::new("update", "third").to_entity(entity));

        world.insert_resource(engine);
        run_scripts(&mut world);

        let script = world.get_component::<Script>(entity).unwrap();
        assert_eq!(script.state, "third");

        let engine = world.get_resource::<ScriptEngine>().unwrap();
        assert_eq!(engine.exec_count(), 3);
    }

    #[test]
    fn message_to_nonexistent_entity() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        // Target an entity that doesn't exist
        let fake = Entity::new(999, 0);
        engine.send(ScriptMessage::new("update", "data").to_entity(fake));

        world.insert_resource(engine);
        run_scripts(&mut world); // should not panic
    }

    #[test]
    fn multiple_entities_receive_messages() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let e1 = world.spawn();
        let e2 = world.spawn();
        world
            .insert_component(e1, Script::new("a.wasm").with_state("a_init"))
            .unwrap();
        world
            .insert_component(e2, Script::new("b.wasm").with_state("b_init"))
            .unwrap();

        engine.send(ScriptMessage::new("update", "a_new").to_entity(e1));
        engine.send(ScriptMessage::new("update", "b_new").to_entity(e2));

        world.insert_resource(engine);
        run_scripts(&mut world);

        assert_eq!(world.get_component::<Script>(e1).unwrap().state, "a_new");
        assert_eq!(world.get_component::<Script>(e2).unwrap().state, "b_new");
    }

    #[test]
    fn script_config_serde() {
        let cfg = ScriptConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: ScriptConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.timeout_ms, 16);
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn wasm_backend_available() {
        let engine = ScriptEngine::default();
        assert!(engine.wasm_available());
    }

    #[cfg(not(feature = "scripting"))]
    #[test]
    fn wasm_backend_unavailable_without_feature() {
        let engine = ScriptEngine::default();
        assert!(!engine.wasm_available());
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn exec_wasm_nonexistent_file() {
        let engine = ScriptEngine::default();
        let result = engine.exec_wasm("/nonexistent/script.wasm");
        // Should return None (execution fails gracefully)
        assert!(result.is_none());
    }

    #[test]
    fn run_scripts_non_wasm_source_still_works() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let entity = world.spawn();
        // Source is not a .wasm file — fallback to message state
        world
            .insert_component(entity, Script::new("logic.lua").with_state("initial"))
            .unwrap();

        engine.send(ScriptMessage::new("update", "new_state").to_entity(entity));

        world.insert_resource(engine);
        run_scripts(&mut world);

        let script = world.get_component::<Script>(entity).unwrap();
        assert_eq!(script.state, "new_state");
    }

    #[test]
    fn custom_script_config() {
        let config = ScriptConfig {
            timeout_ms: 100,
            max_memory: 32 * 1024 * 1024,
        };
        let engine = ScriptEngine::new(config);
        assert_eq!(engine.config().timeout_ms, 100);
        assert_eq!(engine.config().max_memory, 32 * 1024 * 1024);
    }

    #[test]
    fn mixed_wasm_and_non_wasm_entities() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let e_wasm = world.spawn();
        world
            .insert_component(e_wasm, Script::new("game.wasm").with_state("wasm_init"))
            .unwrap();

        let e_lua = world.spawn();
        world
            .insert_component(e_lua, Script::new("game.lua").with_state("lua_init"))
            .unwrap();

        // Send messages to both
        engine.send(ScriptMessage::new("tick", "wasm_data").to_entity(e_wasm));
        engine.send(ScriptMessage::new("tick", "lua_data").to_entity(e_lua));

        world.insert_resource(engine);
        run_scripts(&mut world);

        // Non-wasm entity gets message payload
        let lua_script = world.get_component::<Script>(e_lua).unwrap();
        assert_eq!(lua_script.state, "lua_data");

        // WASM entity: exec_wasm fails (no real file) → falls back to message state
        let wasm_script = world.get_component::<Script>(e_wasm).unwrap();
        assert_eq!(wasm_script.state, "wasm_data");
    }

    #[test]
    fn run_scripts_10_entities() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let mut entities = Vec::new();
        for i in 0..10 {
            let e = world.spawn();
            world
                .insert_component(e, Script::new(format!("s{i}.lua")))
                .unwrap();
            engine.send(ScriptMessage::new("tick", format!("data_{i}")).to_entity(e));
            entities.push(e);
        }

        world.insert_resource(engine);
        run_scripts(&mut world);

        for (i, &e) in entities.iter().enumerate() {
            let script = world.get_component::<Script>(e).unwrap();
            assert_eq!(script.state, format!("data_{i}"));
        }

        let engine = world.get_resource::<ScriptEngine>().unwrap();
        assert_eq!(engine.exec_count(), 10);
    }

    #[test]
    fn run_scripts_disabled_wasm_entity_skipped() {
        let mut world = World::new();
        let mut engine = ScriptEngine::default();

        let entity = world.spawn();
        world
            .insert_component(
                entity,
                Script::new("game.wasm").disabled().with_state("unchanged"),
            )
            .unwrap();

        engine.send(ScriptMessage::new("tick", "should_not_apply").to_entity(entity));

        world.insert_resource(engine);
        run_scripts(&mut world);

        let script = world.get_component::<Script>(entity).unwrap();
        assert_eq!(script.state, "unchanged");
    }

    #[test]
    fn script_engine_outbox_flow() {
        let mut engine = ScriptEngine::default();

        // Simulate script producing output messages
        engine.push_outbox(ScriptMessage::new("spawn_bullet", r#"{"x": 10}"#));
        engine.push_outbox(ScriptMessage::new("play_sound", "explosion.wav"));

        let outbox = engine.drain_outbox();
        assert_eq!(outbox.len(), 2);
        assert_eq!(outbox[0].kind, "spawn_bullet");
        assert_eq!(outbox[1].kind, "play_sound");

        // Outbox drained
        assert!(engine.drain_outbox().is_empty());
    }
}
