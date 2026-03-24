//! Job system — thread pool for parallel system execution.
//!
//! Provides a [`JobPool`] that dispatches closures to worker threads.
//! Used by the [`Scheduler`](crate::Scheduler) to run independent systems
//! concurrently within a stage.

use std::sync::{Arc, Mutex};

/// A job pool backed by a fixed number of worker threads.
///
/// Jobs are closures sent to idle workers. The pool uses a shared work queue
/// and worker threads that park when idle.
pub struct JobPool {
    workers: Vec<std::thread::JoinHandle<()>>,
    sender: Option<crossbeam_channel::Sender<Job>>,
    worker_count: usize,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl JobPool {
    /// Create a pool with the given number of worker threads.
    pub fn new(num_threads: usize) -> Self {
        let num_threads = num_threads.max(1);
        let (sender, receiver) = crossbeam_channel::unbounded::<Job>();
        let mut workers = Vec::with_capacity(num_threads);

        for i in 0..num_threads {
            let rx = receiver.clone();
            let handle = std::thread::Builder::new()
                .name(format!("kiran-worker-{i}"))
                .spawn(move || {
                    while let Ok(job) = rx.recv() {
                        job();
                    }
                })
                .expect("failed to spawn worker thread");
            workers.push(handle);
        }

        Self {
            workers,
            sender: Some(sender),
            worker_count: num_threads,
        }
    }

    /// Create a pool sized to the number of available CPU cores (minus 1 for main thread).
    pub fn auto() -> Self {
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);
        Self::new((cores - 1).max(1))
    }

    /// Submit a job to the pool.
    pub fn submit(&self, job: impl FnOnce() + Send + 'static) {
        if let Some(ref sender) = self.sender {
            let _ = sender.send(Box::new(job));
        }
    }

    /// Submit multiple jobs and wait for all to complete.
    pub fn scope<F>(&self, jobs: Vec<F>)
    where
        F: FnOnce() + Send + 'static,
    {
        if jobs.is_empty() {
            return;
        }

        let remaining = Arc::new(Mutex::new(jobs.len()));
        let condvar = Arc::new(std::sync::Condvar::new());

        for job in jobs {
            let remaining = Arc::clone(&remaining);
            let condvar = Arc::clone(&condvar);
            self.submit(move || {
                job();
                let mut count = remaining.lock().unwrap();
                *count -= 1;
                if *count == 0 {
                    condvar.notify_one();
                }
            });
        }

        // Wait for all jobs to complete
        let mut count = remaining.lock().unwrap();
        while *count > 0 {
            count = condvar.wait(count).unwrap();
        }
    }

    /// Number of worker threads.
    #[must_use]
    #[inline]
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }
}

impl Drop for JobPool {
    fn drop(&mut self) {
        // Drop sender to signal workers to exit
        self.sender.take();
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn pool_creates_workers() {
        let pool = JobPool::new(4);
        assert_eq!(pool.worker_count(), 4);
    }

    #[test]
    fn pool_auto() {
        let pool = JobPool::auto();
        assert!(pool.worker_count() >= 1);
    }

    #[test]
    fn pool_submit_runs() {
        let pool = JobPool::new(2);
        let counter = Arc::new(AtomicU32::new(0));

        let c = counter.clone();
        pool.submit(move || {
            c.fetch_add(1, Ordering::SeqCst);
        });

        // Give it a moment to run
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn pool_scope_waits() {
        let pool = JobPool::new(4);
        let counter = Arc::new(AtomicU32::new(0));

        let jobs: Vec<_> = (0..10)
            .map(|_| {
                let c = counter.clone();
                move || {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            })
            .collect();

        pool.scope(jobs);
        // All jobs must be done after scope returns
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn pool_scope_empty() {
        let pool = JobPool::new(2);
        pool.scope::<Box<dyn FnOnce() + Send>>(vec![]); // should not deadlock
    }

    #[test]
    fn pool_scope_single_job() {
        let pool = JobPool::new(2);
        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        pool.scope(vec![move || {
            c.fetch_add(42, Ordering::SeqCst);
        }]);
        assert_eq!(counter.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn pool_many_jobs() {
        let pool = JobPool::new(4);
        let counter = Arc::new(AtomicU32::new(0));

        let jobs: Vec<_> = (0..1000)
            .map(|_| {
                let c = counter.clone();
                move || {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            })
            .collect();

        pool.scope(jobs);
        assert_eq!(counter.load(Ordering::SeqCst), 1000);
    }

    #[test]
    fn pool_drop_cleans_up() {
        let pool = JobPool::new(2);
        drop(pool); // should not panic or hang
    }

    #[test]
    fn pool_as_world_resource() {
        let mut world = crate::World::new();
        world.insert_resource(JobPool::new(2));
        let pool = world.get_resource::<JobPool>().unwrap();
        assert_eq!(pool.worker_count(), 2);
    }
}
