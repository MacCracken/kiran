# 003 — WASM via kavach/wasmtime for Scripting Sandbox

## Status: Accepted

## Context

Game scripting must let gameplay authors write logic without recompiling the
engine, while preventing scripts from corrupting engine state, accessing the
filesystem arbitrarily, or consuming unbounded resources. The engine ships
to end users who may load untrusted mods.

Options considered:

| Approach | Isolation | Portability | Performance |
|----------|-----------|-------------|-------------|
| Lua (rlua/mlua) | Process-level only | Good | Fast startup |
| WASM (wasmtime) | Memory + capability sandbox | Excellent | Near-native after JIT |
| Native .so/.dll | None | Platform-specific | Native |

## Decision

Use **WASM** executed through the AGNOS **kavach** crate (which wraps
wasmtime) with a minimal WASI policy. Communication between the host engine
and guest scripts uses a message-passing protocol (`ScriptMessage`).

Key implementation details (`script.rs`):

- **`ScriptEngine`** holds a `kavach::backend::wasm::WasmBackend` and a
  single-threaded tokio runtime for async kavach calls.
- **`ScriptConfig`** enforces per-script limits: 16 ms timeout (one frame
  at 60 fps) and 16 MB memory cap.
- **Bounded message buffers**: inbox and outbox are capped at
  `MAX_SCRIPT_MESSAGES` (1024) to prevent runaway scripts from exhausting
  host memory.
- Scripts receive and produce `ScriptMessage` values (JSON payload); they
  never get direct pointers to host memory or ECS storage.

## Consequences

**Positive**

- Strong memory isolation: a script cannot read or write host memory.
- Capability-based security: WASI policy controls filesystem, network, and
  clock access per-script.
- Language-agnostic: any language that compiles to WASM (Rust, C, AssemblyScript,
  Go) can target the scripting layer.
- Deterministic resource limits via timeout and memory cap.

**Negative**

- No direct host API access: scripts cannot call engine functions directly;
  all interaction goes through serialized messages, adding latency.
- Async tokio runtime is required even for synchronous script calls, adding
  a dependency and scheduling complexity.
- kavach does not yet expose a memory-limit setter on `SandboxConfigBuilder`;
  the 16 MB cap is documented intent, not yet enforced upstream.
- WASM module compilation has a cold-start cost; the first call to a new
  script is slower than subsequent calls.
