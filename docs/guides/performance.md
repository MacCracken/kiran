# Performance Tuning

## Frame Profiler

The `FrameProfiler` resource tracks per-system execution cost with exponential moving averages and slow frame detection.

```rust
use kiran::profiler::FrameProfiler;

// history_size=120 frames, slow threshold=18ms (targeting 60 FPS)
let mut profiler = FrameProfiler::new(120, 18.0);

// In your game loop:
profiler.begin_frame();
// ... run systems, recording each:
profiler.record_system("physics_step", physics_duration);
profiler.record_system("render_submit", render_duration);
profiler.end_frame();

// Query results
let avg = profiler.average_frame_time();
let slow_pct = profiler.slow_frame_count as f64 / profiler.total_frames as f64;
```

`SystemTiming` captures individual system names and durations. Use `system_averages()` to find your hottest systems.

## Benchmark History Tracking

All performance claims must be backed by numbers. The `bench-history.sh` script runs criterion benchmarks, appends results to `bench-history.csv`, and generates `benchmarks.md` with three-point tracking (baseline / previous / latest).

```bash
./scripts/bench-history.sh                # run all, append to CSV
./scripts/bench-history.sh results.csv    # custom output file
```

The CSV format: `timestamp,commit,branch,benchmark,estimate_ns`. Never delete baseline rows.

## Hot-Path Guidelines

These rules apply to any code in the game loop:

- **`#[inline]`** on functions called every frame (entity access, component queries)
- **`Vec` arena over `HashMap`** when indices are known — O(1) direct access beats hashing
- **`write!` over `format!`** — avoids temporary `String` allocations
- **`Cow` over `clone`** — borrow when possible, allocate only when mutation is needed
- **Pre-allocate** — `Vec::with_capacity` for known sizes, avoid repeated reallocation

## Common Bottlenecks

| Area | Symptom | Mitigation |
|------|---------|------------|
| Scene loading | Spike on `load_scene` / `spawn_scene` | Load scenes async, use prefab caching |
| Physics step | Consistent high cost in `Physics` stage | Reduce collision pairs, use broadphase, lower substep count |
| Audio graph | Per-frame allocation in mixer | Pre-allocate buffers, pool `SoundSource` instances |
| Renderer submit | Draw call overhead | Batch draw commands, cull off-screen entities |
| Component queries | Linear scan per type | Keep component vecs dense, despawn promptly |

## Profiling Workflow

1. Run `./scripts/bench-history.sh` to establish baseline
2. Make changes
3. Run again, compare latest vs previous in `benchmarks.md`
4. If regression: use `FrameProfiler` to isolate which system regressed
5. Fix, re-run, confirm improvement in CSV history
