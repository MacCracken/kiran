//! Frame profiling — per-system cost tracking and slow frame detection.
//!
//! Provides [`FrameProfiler`] resource for measuring system execution times
//! and detecting performance bottlenecks.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Per-system timing data for a single frame.
#[derive(Debug, Clone)]
pub struct SystemTiming {
    pub name: String,
    pub duration: Duration,
}

/// Frame profiling data — stores timing information for the current and recent frames.
pub struct FrameProfiler {
    /// Current frame start time.
    frame_start: Option<Instant>,
    /// System timings for the current frame.
    current_frame: Vec<SystemTiming>,
    /// Total frame durations for the last N frames.
    frame_history: Vec<Duration>,
    /// Per-system average durations.
    system_averages: HashMap<String, Duration>,
    /// Maximum history length.
    history_size: usize,
    /// Slow frame threshold.
    slow_threshold: Duration,
    /// Count of slow frames detected.
    pub slow_frame_count: u64,
    /// Total frames profiled.
    pub total_frames: u64,
}

impl FrameProfiler {
    /// Create a new profiler with the given history size and slow frame threshold.
    pub fn new(history_size: usize, slow_threshold_ms: f64) -> Self {
        Self {
            frame_start: None,
            current_frame: Vec::new(),
            frame_history: Vec::with_capacity(history_size),
            system_averages: HashMap::new(),
            history_size,
            slow_threshold: Duration::from_secs_f64(slow_threshold_ms / 1000.0),
            slow_frame_count: 0,
            total_frames: 0,
        }
    }

    /// Start timing a new frame.
    pub fn begin_frame(&mut self) {
        self.frame_start = Some(Instant::now());
        self.current_frame.clear();
    }

    /// Record a system's execution time.
    pub fn record_system(&mut self, name: impl Into<String>, duration: Duration) {
        self.current_frame.push(SystemTiming {
            name: name.into(),
            duration,
        });
    }

    /// Time a system execution using a closure.
    pub fn time_system<F, R>(&mut self, name: impl Into<String>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        self.record_system(name, duration);
        result
    }

    /// End the current frame and record its duration.
    pub fn end_frame(&mut self) {
        let frame_duration = self
            .frame_start
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO);

        self.total_frames += 1;

        if frame_duration > self.slow_threshold {
            self.slow_frame_count += 1;
            tracing::warn!(
                duration_ms = frame_duration.as_secs_f64() * 1000.0,
                threshold_ms = self.slow_threshold.as_secs_f64() * 1000.0,
                "slow frame detected"
            );
        }

        // Update history
        if self.frame_history.len() >= self.history_size {
            self.frame_history.remove(0);
        }
        self.frame_history.push(frame_duration);

        // Update per-system averages (exponential moving average)
        for timing in &self.current_frame {
            let avg = self
                .system_averages
                .entry(timing.name.clone())
                .or_insert(timing.duration);
            // EMA with alpha=0.1
            let alpha = 0.1;
            *avg = Duration::from_secs_f64(
                avg.as_secs_f64() * (1.0 - alpha) + timing.duration.as_secs_f64() * alpha,
            );
        }

        self.frame_start = None;
    }

    /// Get the average frame duration over the history window.
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_history.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.frame_history.iter().sum();
        total / self.frame_history.len() as u32
    }

    /// Get the average FPS over the history window.
    pub fn average_fps(&self) -> f64 {
        let avg = self.average_frame_time();
        if avg.is_zero() {
            0.0
        } else {
            1.0 / avg.as_secs_f64()
        }
    }

    /// Get the worst frame time in the history window.
    pub fn worst_frame_time(&self) -> Duration {
        self.frame_history
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::ZERO)
    }

    /// Get per-system average durations.
    pub fn system_averages(&self) -> &HashMap<String, Duration> {
        &self.system_averages
    }

    /// Get the current frame's system timings.
    pub fn current_frame_timings(&self) -> &[SystemTiming] {
        &self.current_frame
    }

    /// Get the slow frame threshold.
    pub fn slow_threshold(&self) -> Duration {
        self.slow_threshold
    }

    /// Percentage of frames that were slow.
    pub fn slow_frame_percentage(&self) -> f64 {
        if self.total_frames == 0 {
            0.0
        } else {
            self.slow_frame_count as f64 / self.total_frames as f64 * 100.0
        }
    }

    /// Generate a multi-line debug overlay string for on-screen display.
    pub fn overlay_text(&self, entity_count: usize) -> String {
        let fps = self.average_fps();
        let frame_ms = self.average_frame_time().as_secs_f64() * 1000.0;
        let worst_ms = self.worst_frame_time().as_secs_f64() * 1000.0;

        let mut text = format!(
            "FPS: {:.0} | Frame: {:.2}ms | Worst: {:.2}ms | Entities: {}",
            fps, frame_ms, worst_ms, entity_count
        );

        if !self.current_frame.is_empty() {
            text.push_str("\nSystems:");
            for timing in &self.current_frame {
                text.push_str(&format!(
                    "\n  {} {:.3}ms",
                    timing.name,
                    timing.duration.as_secs_f64() * 1000.0
                ));
            }
        }

        if self.slow_frame_count > 0 {
            text.push_str(&format!(
                "\nSlow: {} ({:.1}%)",
                self.slow_frame_count,
                self.slow_frame_percentage()
            ));
        }

        text
    }
}

impl Default for FrameProfiler {
    fn default() -> Self {
        Self::new(120, 16.67) // 120 frame history, 60fps threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiler_default() {
        let p = FrameProfiler::default();
        assert_eq!(p.total_frames, 0);
        assert_eq!(p.slow_frame_count, 0);
        assert_eq!(p.average_fps(), 0.0);
    }

    #[test]
    fn profiler_frame_lifecycle() {
        let mut p = FrameProfiler::default();
        p.begin_frame();
        p.record_system("physics", Duration::from_micros(500));
        p.record_system("render", Duration::from_micros(1000));
        p.end_frame();

        assert_eq!(p.total_frames, 1);
        assert_eq!(p.current_frame_timings().len(), 2); // timings from last frame
    }

    #[test]
    fn profiler_time_system() {
        let mut p = FrameProfiler::default();
        p.begin_frame();

        let result = p.time_system("compute", || 42);
        assert_eq!(result, 42);
        assert_eq!(p.current_frame_timings().len(), 1);
        assert_eq!(p.current_frame_timings()[0].name, "compute");

        p.end_frame();
    }

    #[test]
    fn profiler_average_fps() {
        let mut p = FrameProfiler::new(10, 100.0);

        for _ in 0..5 {
            p.begin_frame();
            std::thread::sleep(Duration::from_millis(1));
            p.end_frame();
        }

        assert!(p.average_fps() > 0.0);
        assert_eq!(p.total_frames, 5);
    }

    #[test]
    fn profiler_slow_frame_detection() {
        let mut p = FrameProfiler::new(10, 0.001); // 0.001ms threshold — everything is slow

        p.begin_frame();
        std::thread::sleep(Duration::from_millis(1));
        p.end_frame();

        assert_eq!(p.slow_frame_count, 1);
        assert!(p.slow_frame_percentage() > 0.0);
    }

    #[test]
    fn profiler_no_slow_frames() {
        let mut p = FrameProfiler::new(10, 1000.0); // 1 second threshold

        p.begin_frame();
        p.end_frame();

        assert_eq!(p.slow_frame_count, 0);
        assert_eq!(p.slow_frame_percentage(), 0.0);
    }

    #[test]
    fn profiler_worst_frame() {
        let mut p = FrameProfiler::new(10, 100.0);

        p.begin_frame();
        p.end_frame();

        p.begin_frame();
        std::thread::sleep(Duration::from_millis(5));
        p.end_frame();

        p.begin_frame();
        p.end_frame();

        assert!(p.worst_frame_time() >= Duration::from_millis(4));
    }

    #[test]
    fn profiler_system_averages() {
        let mut p = FrameProfiler::default();

        for _ in 0..10 {
            p.begin_frame();
            p.record_system("physics", Duration::from_micros(100));
            p.record_system("render", Duration::from_micros(200));
            p.end_frame();
        }

        let avgs = p.system_averages();
        assert!(avgs.contains_key("physics"));
        assert!(avgs.contains_key("render"));
        assert!(avgs["render"] > avgs["physics"]);
    }

    #[test]
    fn profiler_history_wraps() {
        let mut p = FrameProfiler::new(5, 100.0);

        for _ in 0..20 {
            p.begin_frame();
            p.end_frame();
        }

        assert_eq!(p.total_frames, 20);
        // History should be capped at 5
        assert!(p.frame_history.len() <= 5);
    }

    #[test]
    fn profiler_as_world_resource() {
        let mut world = crate::World::new();
        world.insert_resource(FrameProfiler::default());

        let profiler = world.get_resource::<FrameProfiler>().unwrap();
        assert_eq!(profiler.total_frames, 0);
    }

    #[test]
    fn overlay_text_basic() {
        let mut p = FrameProfiler::default();
        p.begin_frame();
        p.record_system("physics", Duration::from_micros(500));
        p.record_system("render", Duration::from_millis(2));
        p.end_frame();

        let text = p.overlay_text(42);
        assert!(text.contains("FPS:"));
        assert!(text.contains("Entities: 42"));
        assert!(text.contains("physics"));
        assert!(text.contains("render"));
    }

    #[test]
    fn overlay_text_empty() {
        let p = FrameProfiler::default();
        let text = p.overlay_text(0);
        assert!(text.contains("FPS: 0"));
        assert!(text.contains("Entities: 0"));
    }
}
