# 001 — Vec Arena Storage for ECS Components

## Status: Accepted

## Context

Game engines iterate over thousands of entities every frame. The ECS component
storage must provide O(1) access by entity and cache-friendly sequential
iteration for systems that touch every entity (physics step, transform
propagation, render submission).

HashMap-based storage pays per-lookup hashing costs and scatters data across
the heap, destroying cache locality on tight per-frame loops. With entity
counts in the tens of thousands, the overhead becomes measurable.

## Decision

Component storage is a dense `Vec<Option<Box<dyn Any + Send + Sync>>>`
indexed directly by the entity's `u32` index (the lower 32 bits of the
`Entity` handle). Entity allocation uses a generational index scheme:

- **`EntityAllocator`** maintains a `Vec<u32>` of generations and a free list
  of recycled indices.
- **`Entity`** packs generation (upper 32) and index (lower 32) into a single
  `u64`, enabling cheap copy and comparison.
- **`World.components`** maps `TypeId` to a per-type `ComponentVec`. Lookups
  are a single Vec index operation after the type lookup.

For hot-path bulk iteration, the opt-in `ArchetypeStorage` in
`archetype.rs` groups entities with identical component signatures into
contiguous SOA columns, giving linear memory access patterns.

## Consequences

**Positive**

- O(1) component get/set per entity (single Vec index).
- Sequential iteration is cache-line friendly, critical for physics and
  rendering systems that touch every entity each frame.
- Generational indices detect use-after-despawn without garbage collection.
- Archetype layer adds SOA iteration for the hottest paths.

**Negative**

- Sparse populations waste memory: if only entity 0 and entity 10 000 exist,
  the Vec still allocates 10 001 slots per component type.
- Removing a component type from all entities requires an O(n) scan of its
  Vec, though this is rare at runtime.
- The `Option` wrapper adds a branch per access (predictable in practice).

**Trade-off accepted** because game workloads are dense (most indices
occupied) and the cache/iteration wins dominate the sparse-waste cost.
