//! Object pooling and arena allocators.
//!
//! - [`Pool`] — reusable object pool with acquire/release
//! - [`FrameArena`] — per-frame bump allocator, reset each frame
//! - [`SimdVec`] — aligned f32 storage for SIMD-friendly data layouts

// ---------------------------------------------------------------------------
// Object pool
// ---------------------------------------------------------------------------

/// A reusable object pool — pre-allocates objects to avoid per-frame allocation.
///
/// Objects are created via the factory function on first acquire, then recycled.
/// Call [`release`](Pool::release) to return objects for reuse.
pub struct Pool<T> {
    /// Available (free) objects.
    free: Vec<T>,
    /// Factory for creating new objects.
    factory: Box<dyn Fn() -> T + Send + Sync>,
    /// Total objects created (free + in use).
    total_created: usize,
    /// High-water mark (max concurrent in-use).
    peak_in_use: usize,
    /// Current count in use.
    in_use: usize,
}

impl<T> Pool<T> {
    /// Create a new pool with a factory function.
    pub fn new(factory: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self {
            free: Vec::new(),
            factory: Box::new(factory),
            total_created: 0,
            peak_in_use: 0,
            in_use: 0,
        }
    }

    /// Create a pool pre-warmed with `count` objects.
    pub fn with_capacity(factory: impl Fn() -> T + Send + Sync + 'static, count: usize) -> Self {
        let factory_box: Box<dyn Fn() -> T + Send + Sync> = Box::new(factory);
        let mut free = Vec::with_capacity(count);
        for _ in 0..count {
            free.push(factory_box());
        }
        Self {
            free,
            factory: factory_box,
            total_created: count,
            peak_in_use: 0,
            in_use: 0,
        }
    }

    /// Acquire an object from the pool (reuses if available, creates if not).
    #[inline]
    pub fn acquire(&mut self) -> T {
        self.in_use += 1;
        if self.in_use > self.peak_in_use {
            self.peak_in_use = self.in_use;
        }
        if let Some(obj) = self.free.pop() {
            obj
        } else {
            self.total_created += 1;
            (self.factory)()
        }
    }

    /// Release an object back to the pool for reuse.
    #[inline]
    pub fn release(&mut self, obj: T) {
        self.in_use = self.in_use.saturating_sub(1);
        self.free.push(obj);
    }

    /// Number of objects available for immediate reuse.
    #[must_use]
    #[inline]
    pub fn available(&self) -> usize {
        self.free.len()
    }

    /// Number of objects currently in use.
    #[must_use]
    #[inline]
    pub fn in_use(&self) -> usize {
        self.in_use
    }

    /// Peak number of objects in use at any time.
    #[must_use]
    #[inline]
    pub fn peak_in_use(&self) -> usize {
        self.peak_in_use
    }

    /// Total objects ever created by this pool.
    #[must_use]
    #[inline]
    pub fn total_created(&self) -> usize {
        self.total_created
    }

    /// Pre-warm the pool to ensure at least `count` objects are available.
    pub fn warm(&mut self, count: usize) {
        while self.free.len() < count {
            self.free.push((self.factory)());
            self.total_created += 1;
        }
    }

    /// Shrink the free list to at most `max_free` objects.
    pub fn shrink(&mut self, max_free: usize) {
        self.free.truncate(max_free);
    }
}

// ---------------------------------------------------------------------------
// Frame arena
// ---------------------------------------------------------------------------

/// Per-frame bump allocator for temporary data that lives one frame.
///
/// Allocates from a contiguous buffer. Call [`reset`](FrameArena::reset) at
/// the start of each frame to reclaim all memory without freeing.
pub struct FrameArena {
    buffer: Vec<u8>,
    offset: usize,
}

impl FrameArena {
    /// Create an arena with the given capacity in bytes.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity],
            offset: 0,
        }
    }

    /// Allocate `count` elements of type T, returning a mutable slice.
    ///
    /// Returns `None` if the arena is full.
    pub fn alloc_slice<T: Copy + Default>(&mut self, count: usize) -> Option<&mut [T]> {
        let align = std::mem::align_of::<T>();
        let size = std::mem::size_of::<T>() * count;

        // Align the offset
        let aligned = (self.offset + align - 1) & !(align - 1);
        if aligned + size > self.buffer.len() {
            return None;
        }

        self.offset = aligned + size;

        // SAFETY: buffer is large enough, alignment is correct, T is Copy + Default
        let ptr = self.buffer[aligned..].as_mut_ptr().cast::<T>();
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, count) };

        // Initialize
        for elem in slice.iter_mut() {
            *elem = T::default();
        }

        Some(slice)
    }

    /// Reset the arena for the next frame (no deallocation, just resets offset).
    #[inline]
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Bytes currently used.
    #[must_use]
    #[inline]
    pub fn used(&self) -> usize {
        self.offset
    }

    /// Total capacity in bytes.
    #[must_use]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Remaining bytes available.
    #[must_use]
    #[inline]
    pub fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.offset)
    }
}

// ---------------------------------------------------------------------------
// SIMD-friendly SOA storage
// ---------------------------------------------------------------------------

/// Aligned f32 vector for SIMD-friendly data layouts.
///
/// Stores contiguous f32 values suitable for SIMD processing (positions,
/// velocities, etc.). Used as Structure of Arrays (SOA) storage.
///
/// ```rust
/// use kiran::pool::SimdVec;
///
/// let mut positions_x = SimdVec::new();
/// let mut positions_y = SimdVec::new();
///
/// // Add 4 entities
/// for i in 0..4 {
///     positions_x.push(i as f32 * 10.0);
///     positions_y.push(i as f32 * 5.0);
/// }
///
/// // SIMD-friendly batch update (all X values contiguous)
/// positions_x.apply(|x| x + 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct SimdVec {
    data: Vec<f32>,
}

impl SimdVec {
    /// Create an empty SIMD vector.
    #[must_use]
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Create filled with a value.
    #[must_use]
    pub fn filled(value: f32, count: usize) -> Self {
        Self {
            data: vec![value; count],
        }
    }

    /// Push a value.
    #[inline]
    pub fn push(&mut self, value: f32) {
        self.data.push(value);
    }

    /// Get a value by index.
    #[must_use]
    #[inline]
    pub fn get(&self, index: usize) -> Option<f32> {
        self.data.get(index).copied()
    }

    /// Set a value by index.
    #[inline]
    pub fn set(&mut self, index: usize, value: f32) {
        if let Some(slot) = self.data.get_mut(index) {
            *slot = value;
        }
    }

    /// Apply a function to every element (SIMD-friendly — contiguous memory).
    #[inline]
    pub fn apply(&mut self, f: impl Fn(f32) -> f32) {
        for v in &mut self.data {
            *v = f(*v);
        }
    }

    /// Apply element-wise addition with another SimdVec.
    #[inline]
    pub fn add_assign(&mut self, other: &SimdVec) {
        let len = self.data.len().min(other.data.len());
        for i in 0..len {
            self.data[i] += other.data[i];
        }
    }

    /// Apply element-wise multiplication with a scalar.
    #[inline]
    pub fn scale(&mut self, factor: f32) {
        for v in &mut self.data {
            *v *= factor;
        }
    }

    /// Dot product with another SimdVec.
    #[must_use]
    pub fn dot(&self, other: &SimdVec) -> f32 {
        let len = self.data.len().min(other.data.len());
        let mut sum = 0.0f32;
        for i in 0..len {
            sum += self.data[i] * other.data[i];
        }
        sum
    }

    /// Sum all elements.
    #[must_use]
    #[inline]
    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    /// Number of elements.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Raw slice access (for SIMD intrinsics or bulk operations).
    #[must_use]
    #[inline]
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    /// Mutable raw slice access.
    #[must_use]
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Clear all elements.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Resize to a new length, filling with a default value.
    pub fn resize(&mut self, new_len: usize, value: f32) {
        self.data.resize(new_len, value);
    }
}

impl Default for SimdVec {
    fn default() -> Self {
        Self::new()
    }
}

/// SOA (Structure of Arrays) storage for 2D positions.
///
/// Instead of `Vec<[f32; 2]>` (AOS), stores X and Y as separate
/// contiguous arrays for SIMD-friendly iteration.
#[derive(Debug, Clone, Default)]
pub struct Soa2d {
    /// X components.
    pub x: SimdVec,
    /// Y components.
    pub y: SimdVec,
}

impl Soa2d {
    /// Create an empty 2D SOA store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a 2D point.
    pub fn push(&mut self, px: f32, py: f32) {
        self.x.push(px);
        self.y.push(py);
    }

    /// Get a 2D point by index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<(f32, f32)> {
        Some((self.x.get(index)?, self.y.get(index)?))
    }

    /// Number of points.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Whether the store is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// Translate all points by (dx, dy).
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x.apply(|v| v + dx);
        self.y.apply(|v| v + dy);
    }

    /// Scale all points by a factor.
    pub fn scale(&mut self, factor: f32) {
        self.x.scale(factor);
        self.y.scale(factor);
    }

    /// Remove all points.
    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
    }
}

/// SOA (Structure of Arrays) storage for 3D positions.
#[derive(Debug, Clone, Default)]
pub struct Soa3d {
    /// X components.
    pub x: SimdVec,
    /// Y components.
    pub y: SimdVec,
    /// Z components.
    pub z: SimdVec,
}

impl Soa3d {
    /// Create an empty 3D SOA store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a 3D point.
    pub fn push(&mut self, px: f32, py: f32, pz: f32) {
        self.x.push(px);
        self.y.push(py);
        self.z.push(pz);
    }

    /// Get a 3D point by index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<(f32, f32, f32)> {
        Some((self.x.get(index)?, self.y.get(index)?, self.z.get(index)?))
    }

    /// Number of points.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Whether the store is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// Translate all points.
    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        self.x.apply(|v| v + dx);
        self.y.apply(|v| v + dy);
        self.z.apply(|v| v + dz);
    }

    /// Add velocities (element-wise addition).
    pub fn add_velocities(&mut self, vx: &SimdVec, vy: &SimdVec, vz: &SimdVec) {
        self.x.add_assign(vx);
        self.y.add_assign(vy);
        self.z.add_assign(vz);
    }

    /// Remove all points.
    pub fn clear(&mut self) {
        self.x.clear();
        self.y.clear();
        self.z.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Pool tests --

    #[test]
    fn pool_acquire_release() {
        let mut pool: Pool<Vec<u8>> = Pool::new(Vec::new);
        let v = pool.acquire();
        assert_eq!(pool.in_use(), 1);
        assert_eq!(pool.available(), 0);
        pool.release(v);
        assert_eq!(pool.in_use(), 0);
        assert_eq!(pool.available(), 1);
    }

    #[test]
    fn pool_reuses_objects() {
        let mut pool: Pool<Vec<u8>> = Pool::new(Vec::new);
        let mut v = pool.acquire();
        v.push(42);
        pool.release(v);

        let v2 = pool.acquire();
        // Reused — still has the data
        assert_eq!(v2[0], 42);
        assert_eq!(pool.total_created(), 1); // only created once
    }

    #[test]
    fn pool_with_capacity() {
        let pool: Pool<Vec<f32>> = Pool::with_capacity(Vec::new, 10);
        assert_eq!(pool.available(), 10);
        assert_eq!(pool.total_created(), 10);
        assert_eq!(pool.in_use(), 0);
    }

    #[test]
    fn pool_warm() {
        let mut pool: Pool<String> = Pool::new(String::new);
        pool.warm(5);
        assert_eq!(pool.available(), 5);
    }

    #[test]
    fn pool_shrink() {
        let mut pool: Pool<u32> = Pool::with_capacity(|| 0, 20);
        assert_eq!(pool.available(), 20);
        pool.shrink(5);
        assert_eq!(pool.available(), 5);
    }

    #[test]
    fn pool_peak_tracking() {
        let mut pool: Pool<u32> = Pool::new(|| 0);
        let a = pool.acquire();
        let b = pool.acquire();
        let c = pool.acquire();
        assert_eq!(pool.peak_in_use(), 3);
        pool.release(a);
        pool.release(b);
        pool.release(c);
        assert_eq!(pool.peak_in_use(), 3); // peak unchanged
        assert_eq!(pool.in_use(), 0);
    }

    // -- FrameArena tests --

    #[test]
    fn arena_alloc_and_reset() {
        let mut arena = FrameArena::new(1024);
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.capacity(), 1024);

        let slice = arena.alloc_slice::<f32>(10).unwrap();
        assert_eq!(slice.len(), 10);
        assert!(arena.used() > 0);

        arena.reset();
        assert_eq!(arena.used(), 0);
    }

    #[test]
    fn arena_alloc_multiple() {
        let mut arena = FrameArena::new(4096);
        let a_ptr = arena.alloc_slice::<f32>(100).unwrap().as_mut_ptr();
        let b_ptr = arena.alloc_slice::<u32>(50).unwrap().as_mut_ptr();
        // SAFETY: allocations don't overlap, arena is alive
        unsafe {
            *a_ptr = 1.0;
            *b_ptr = 42;
            assert_eq!(*a_ptr, 1.0);
            assert_eq!(*b_ptr, 42);
        }
    }

    #[test]
    fn arena_overflow() {
        let mut arena = FrameArena::new(32);
        let result = arena.alloc_slice::<f32>(100); // 400 bytes > 32
        assert!(result.is_none());
    }

    #[test]
    fn arena_remaining() {
        let mut arena = FrameArena::new(1024);
        assert_eq!(arena.remaining(), 1024);
        arena.alloc_slice::<f32>(10).unwrap(); // 40 bytes + alignment
        assert!(arena.remaining() < 1024);
    }

    #[test]
    fn arena_reset_reuses_memory() {
        let mut arena = FrameArena::new(256);
        for _ in 0..10 {
            arena.alloc_slice::<f32>(10).unwrap();
            arena.reset();
        }
        // Should never run out
        assert_eq!(arena.used(), 0);
    }

    // -- SimdVec tests --

    #[test]
    fn simd_vec_basic() {
        let mut v = SimdVec::new();
        v.push(1.0);
        v.push(2.0);
        v.push(3.0);
        assert_eq!(v.len(), 3);
        assert_eq!(v.get(0), Some(1.0));
        assert_eq!(v.get(2), Some(3.0));
        assert_eq!(v.get(5), None);
    }

    #[test]
    fn simd_vec_apply() {
        let mut v = SimdVec::filled(2.0, 4);
        v.apply(|x| x * 3.0);
        assert_eq!(v.as_slice(), &[6.0, 6.0, 6.0, 6.0]);
    }

    #[test]
    fn simd_vec_add_assign() {
        let mut a = SimdVec::filled(1.0, 3);
        let b = SimdVec::filled(2.0, 3);
        a.add_assign(&b);
        assert_eq!(a.as_slice(), &[3.0, 3.0, 3.0]);
    }

    #[test]
    fn simd_vec_scale() {
        let mut v = SimdVec::filled(5.0, 3);
        v.scale(0.5);
        assert_eq!(v.as_slice(), &[2.5, 2.5, 2.5]);
    }

    #[test]
    fn simd_vec_dot() {
        let a = SimdVec::filled(2.0, 3);
        let b = SimdVec::filled(3.0, 3);
        assert!((a.dot(&b) - 18.0).abs() < f32::EPSILON);
    }

    #[test]
    fn simd_vec_sum() {
        let v = SimdVec::filled(1.5, 4);
        assert!((v.sum() - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn simd_vec_set() {
        let mut v = SimdVec::filled(0.0, 3);
        v.set(1, 42.0);
        assert_eq!(v.get(1), Some(42.0));
    }

    #[test]
    fn simd_vec_resize() {
        let mut v = SimdVec::new();
        v.resize(5, 1.0);
        assert_eq!(v.len(), 5);
        assert_eq!(v.as_slice(), &[1.0, 1.0, 1.0, 1.0, 1.0]);
    }

    // -- SOA tests --

    #[test]
    fn soa2d_basic() {
        let mut soa = Soa2d::new();
        soa.push(1.0, 2.0);
        soa.push(3.0, 4.0);
        assert_eq!(soa.len(), 2);
        assert_eq!(soa.get(0), Some((1.0, 2.0)));
        assert_eq!(soa.get(1), Some((3.0, 4.0)));
    }

    #[test]
    fn soa2d_translate() {
        let mut soa = Soa2d::new();
        soa.push(0.0, 0.0);
        soa.push(1.0, 1.0);
        soa.translate(10.0, 20.0);
        assert_eq!(soa.get(0), Some((10.0, 20.0)));
        assert_eq!(soa.get(1), Some((11.0, 21.0)));
    }

    #[test]
    fn soa3d_basic() {
        let mut soa = Soa3d::new();
        soa.push(1.0, 2.0, 3.0);
        assert_eq!(soa.len(), 1);
        assert_eq!(soa.get(0), Some((1.0, 2.0, 3.0)));
    }

    #[test]
    fn soa3d_add_velocities() {
        let mut positions = Soa3d::new();
        positions.push(0.0, 0.0, 0.0);
        positions.push(10.0, 10.0, 10.0);

        let vx = SimdVec::filled(1.0, 2);
        let vy = SimdVec::filled(2.0, 2);
        let vz = SimdVec::filled(3.0, 2);
        positions.add_velocities(&vx, &vy, &vz);

        assert_eq!(positions.get(0), Some((1.0, 2.0, 3.0)));
        assert_eq!(positions.get(1), Some((11.0, 12.0, 13.0)));
    }

    #[test]
    fn soa3d_translate() {
        let mut soa = Soa3d::new();
        soa.push(0.0, 0.0, 0.0);
        soa.translate(5.0, 10.0, 15.0);
        assert_eq!(soa.get(0), Some((5.0, 10.0, 15.0)));
    }

    #[test]
    fn soa2d_scale() {
        let mut soa = Soa2d::new();
        soa.push(2.0, 4.0);
        soa.scale(0.5);
        assert_eq!(soa.get(0), Some((1.0, 2.0)));
    }

    #[test]
    fn pool_as_world_resource() {
        let mut world = crate::World::new();
        let pool: Pool<Vec<f32>> = Pool::with_capacity(Vec::new, 10);
        world.insert_resource(pool);

        let pool = world.get_resource_mut::<Pool<Vec<f32>>>().unwrap();
        let v = pool.acquire();
        assert_eq!(pool.in_use(), 1);
        pool.release(v);
    }

    #[test]
    fn arena_as_world_resource() {
        let mut world = crate::World::new();
        world.insert_resource(FrameArena::new(4096));

        let arena = world.get_resource_mut::<FrameArena>().unwrap();
        let slice = arena.alloc_slice::<f32>(100).unwrap();
        slice[0] = 42.0;
        assert_eq!(slice[0], 42.0);
    }
}
