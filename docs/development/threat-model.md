# Threat Model

**Scope**: scripting sandbox, multiplayer protocol, asset loading

**Last updated**: 2026-04-01

---

## 1. Scripting Sandbox (WASM via kavach)

### Attack surface

- Untrusted WASM modules loaded from user-provided paths.
- Script-to-host message channel (inbox/outbox).
- Resource consumption: CPU time and memory per script invocation.

### Mitigations applied

| Threat | Mitigation | Location |
|--------|-----------|----------|
| Arbitrary host memory access | WASM linear memory isolation (wasmtime) | kavach backend |
| Filesystem/network escape | Minimal WASI policy; no ambient capabilities | `ScriptEngine::new` |
| CPU exhaustion (infinite loop) | 16 ms per-call timeout via `ScriptConfig` | `script.rs` |
| Memory exhaustion | 16 MB per-script memory cap (documented intent) | `script.rs` |
| Message flood from script | Bounded outbox at `MAX_SCRIPT_MESSAGES` (1024) | `script.rs` |
| Message flood to script | Bounded inbox at `MAX_SCRIPT_MESSAGES` (1024) | `script.rs` |
| Malformed JSON payload | Serde deserialization with typed `ScriptMessage` | `script.rs` |

### Residual risks

- **Memory cap not enforced upstream.** kavach's `SandboxConfigBuilder` does
  not yet expose a memory-limit setter. The 16 MB cap is config-only; a
  malicious module could allocate more until kavach adds enforcement.
- **Side channels.** Timing side channels (cache, branch predictor) are
  inherent to shared-CPU WASM execution. Not mitigated; acceptable for
  game workloads, not for cryptographic isolation.
- **Module validation.** Kiran does not independently validate WASM module
  structure before passing to kavach/wasmtime. A malformed module could
  trigger a wasmtime bug. Mitigation: keep wasmtime updated.

---

## 2. Multiplayer Protocol (majra)

### Attack surface

- Inbound network messages from untrusted clients.
- Node identity (`NodeId`) spoofing.
- State snapshot and delta injection.
- Input replication payload.

### Mitigations applied

| Threat | Mitigation | Location |
|--------|-----------|----------|
| Oversized snapshots | `MAX_ENTITY_LIST_SIZE` (10,000) enforced in `validate_message` | `net.rs` |
| Oversized deltas | Same entity list cap on `StateDelta.changes` | `net.rs` |
| Oversized input payload | `MAX_INPUT_PAYLOAD_LEN` (1 MB) check | `net.rs` |
| Long NodeId strings | `MAX_NODE_ID_LEN` (256) on all NodeId fields | `net.rs` |
| Inbox flooding | `MAX_INBOX_SIZE` (4096) drops excess inbound messages | `net.rs` |
| Outbox flooding | `MAX_OUTBOX_SIZE` (4096) drops excess outbound messages | `net.rs` |
| Reliable channel memory | `MAX_DEDUP_ENTRIES` (16,384) with automatic trimming | `net.rs` |
| Identity spoofing | Doc-level security warnings on `PlayerJoin`/`PlayerLeave`: app layer MUST authenticate | `net.rs` |

### Residual risks

- **No transport-layer authentication.** The network layer does not
  authenticate peers. `PlayerJoin` and `PlayerLeave` carry `node_id` but the
  application layer is responsible for verifying identity (e.g., via token
  exchange). A malicious client can spoof any `node_id` if the app layer
  does not authenticate.
- **No encryption.** Message payloads are serialized JSON over the transport
  provided by majra. If the transport is unencrypted, game state and input
  are visible to network observers. Mitigation: use TLS at the transport
  layer (consumer responsibility).
- **Deserialization amplification.** A small wire message could deserialize
  into large in-memory structures. The entity list and payload size caps
  bound this, but deeply nested JSON in `InputMessage.payload` is not
  depth-limited.
- **Tick manipulation.** Clients send their own tick values in
  `InputMessage`. A malicious client could send future ticks to desync
  other clients. Server-side tick validation is the consumer's
  responsibility.

---

## 3. Asset Loading

### Attack surface

- File paths provided by scene files, scripts, or user input.
- Asset file contents (images, audio, WASM modules, TOML scenes).
- Asset file size.

### Mitigations applied

| Threat | Mitigation | Location |
|--------|-----------|----------|
| Path traversal (`../../../etc/passwd`) | `validate_asset_path` rejects `..` components and verifies canonical path stays under cwd | `asset.rs` |
| Symlink escape | Canonicalization resolves symlinks; result checked against cwd | `asset.rs` |
| Oversized assets | `MAX_ASSET_FILE_SIZE` (256 MB) cap | `asset.rs` |
| Unexpected file types | `AssetType` classification; unknown types are loadable but typed | `asset.rs` |

### Residual risks

- **TOCTOU on path validation.** The path is validated then opened in
  separate operations. A race condition could swap a safe path for a symlink
  between validation and open. Low risk on typical game asset directories;
  not mitigated.
- **Content-based attacks.** Malformed images or audio files could exploit
  bugs in decoder libraries (image codecs, audio decoders). Kiran delegates
  decoding to soorat (rendering) and dhvani/shravan (audio); those crates
  own decoder security.
- **Scene file injection.** A crafted TOML scene could reference paths
  outside the asset directory. Mitigated by `validate_asset_path` being
  called on all path references, but only if the consumer consistently uses
  the validation API.
- **No content signing.** Assets are not cryptographically signed. A
  compromised asset directory could serve modified files. Relevant for
  distribution pipelines; not in scope for the engine runtime.
