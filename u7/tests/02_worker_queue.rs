//! Run this file with `cargo test --test 02_worker_queue`.

//! TODO: implement a simple parallel job queue
//!
//! Implement a struct `WorkerQueue`, which will manage N worker threads.
//! It will allow its users to execute a job on a single worker, and then read the result of that
//! job.
//!
//! ## Creation of the queue
//! The queue should offer a `new` associated function, which will receive the number of workers
//! in the queue, along with the size of a queue for each individual workers.
//! For example, if you execute `WorkerQueue::new(4, 2)`, then four worker threads should be
//! spawned, and each worker thread should have its own queue of size (bound) of `2`.
//!
//! ## Jobs
//! It will be possible to execute a job using the `enqueue` method, which receives something
//! callable that can be executed within a worker.
//! `enqueue` should be callable on a shared reference to the queue.
//!
//! You will need to make sure that the passed function can be safely passed to a worker thread.
//! The queue should be generic over the return type of jobs, all jobs will return the same type.
//!
//! ## Job scheduling
//! Jobs should be scheduled in a trivial round-robin matter.
//! In other words, the first job goes to worker 0, the second to worker 1, the third to worker 2,
//! etc., until you run out of workers and you start from the beginning again.
//! Note that the goal is for the workers to run in parallel, so they should not block each other
//! from executing jobs.
//!
//! ## Reading results
//! The queue should offer a `next_result` method, which will block until the next result is
//! ready. Note that results can "skip ahead" one another, e.g. if you enqueue a job A, and then job
//! B, and job B finishes sooner than job A, then `next_result` should return the result of job B.
//! `next_result` should be callable on a shared reference to the queue.
//!
//! ## Closing of the queue
//! The queue should have a `close` method, which consumes it, drops all resources and waits
//! until all worker threads have terminated.
//!
//! See tests for more details.
//!
//! **DO NOT** use Rayon or any other crate, implement the queue manually using only libstd.
//!
//! TODO(question): is it possible to enqueue work to WorkerQueue from multiple threads?
//! Try it and see what happens. If it's not possible, how could you make it work?
//!
//! Hint: When writing parallel code, you might run into deadlocks (which will be presented as a
//! "blank screen" with no output and maybe a spinning wheel :) ).
//! If you want to see interactive output during the execution of a test, you can add stderr print
//! statements (e.g. using `eprintln!`) and run tests with `cargo test -- --nocapture`, so that you
//! see the output interactively. Alternatively, you can try to use a debugger (e.g. GDB, LLDB or
//! GDB/LLDB integrated within an IDE).

use std::{collections::VecDeque, sync::{atomic::{AtomicUsize, Ordering}, Arc, Condvar, Mutex}};
use std::thread;

pub struct WorkerQueue<T> {
    workers: Vec<Worker<T>>,
    result_receiver: std::sync::mpsc::Receiver<T>,
    enqueue_counter: AtomicUsize,
}

type BoxedFnOnce<T> = Option<Box<dyn FnOnce() -> T + Send>>;
type QueueMutexWithCond<T> = (Mutex<VecDeque<BoxedFnOnce<T>>>, Condvar);
struct Worker<T> {
    task_queue: Arc<QueueMutexWithCond<T>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl<T: Send + 'static> WorkerQueue<T> {
    pub fn new(worker_count: usize, queue_size: usize) -> Self {
        let (result_sender, result_receiver) = std::sync::mpsc::sync_channel(1000);
        let mut workers = Vec::with_capacity(worker_count);

        for _ in 0..worker_count {
            let task_queue = Arc::new((Mutex::new(VecDeque::<BoxedFnOnce<T>>::with_capacity(queue_size)), Condvar::new()));
            let task_queue_clone = Arc::clone(&task_queue);
            let result_sender = result_sender.clone();

            let handle = thread::spawn(move || loop {
                let task_option = {
                    let (lock, cvar) = &*task_queue_clone;
                    let mut queue = lock.lock().unwrap();

                    while queue.is_empty() {
                        queue = cvar.wait(queue).unwrap();
                    }

                    queue.pop_front()
                };

                match task_option {
                    Some(Some(task)) => {
                        let result = task();
                        result_sender.send(result).unwrap();

                        // Notify after completing a task to unblock any waiting `enqueue`
                        let (lock, cvar) = &*task_queue_clone;
                        let _queue = lock.lock().unwrap();
                        cvar.notify_all();
                    }
                    Some(None) | None => break,
                }
            });

            workers.push(Worker {
                task_queue,
                handle: Some(handle),
            });
        }

        WorkerQueue {
            workers,
            result_receiver,
            enqueue_counter: AtomicUsize::new(0),
        }
    }

    pub fn enqueue<F>(&self, job: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let worker_index = self.enqueue_counter.fetch_add(1, Ordering::SeqCst) % self.workers.len();
        let (queue, cvar) = &*self.workers[worker_index].task_queue;
        
        let mut queue_lock = queue.lock().unwrap();
    
        // Wait while the queue is at capacity
        while queue_lock.len() >= queue_lock.capacity() {
            queue_lock = cvar.wait(queue_lock).unwrap();
        }
    
        // Add the job to the queue and notify a worker thread
        queue_lock.push_back(Some(Box::new(job)));
        cvar.notify_one();
    }

    pub fn next_result(&self) -> T {
        self.result_receiver.recv().unwrap()
    }

    pub fn close(self) {
        for worker in &self.workers {
            let (queue, cvar) = &*worker.task_queue;
            let mut queue_lock = queue.lock().unwrap();
            queue_lock.push_back(None); 
            cvar.notify_all();
        }

        for worker in self.workers {
            if let Some(handle) = worker.handle {
                handle.join().unwrap();
            }
        }
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::WorkerQueue;
    use std::sync::{Arc, Mutex};
    use std::thread::ThreadId;
    use std::time::{Duration, Instant};

    #[test]
    fn empty_queue() {
        let queue = WorkerQueue::<u32>::new(4, 4);
        queue.close();
    }

    #[test]
    fn enqueue_read() {
        let queue = WorkerQueue::<u32>::new(1, 1);
        queue.enqueue(|| 1);
        assert_eq!(queue.next_result(), 1);

        queue.close();
    }

    #[test]
    fn different_type() {
        // In particular, this type is not Clone, which should not be required
        #[derive(Debug, Eq, PartialEq)]
        struct Foo(String);

        let queue = WorkerQueue::<Foo>::new(1, 1);
        queue.enqueue(|| Foo("foo".to_string()));
        assert_eq!(queue.next_result(), Foo("foo".to_string()));

        queue.close();
    }

    #[test]
    fn close_while_nonempty() {
        let queue = WorkerQueue::<u32>::new(1, 1);
        queue.enqueue(|| 1);
        std::thread::sleep(Duration::from_millis(100));

        queue.close();
    }

    #[test]
    fn close_while_working() {
        let queue = WorkerQueue::<u32>::new(1, 1);
        queue.enqueue(|| {
            std::thread::sleep(Duration::from_secs(1));
            1
        });

        // Oops. The queue should exit gracefully when this happens
        queue.close();
    }

    #[test]
    fn round_robin() {
        let queue = WorkerQueue::<u32>::new(4, 1);

        let thread_ids: Arc<Mutex<Option<ThreadId>>> = Arc::new(Mutex::new(None));
        for i in 0..4 {
            let thread_ids = thread_ids.clone();
            queue.enqueue(move || {
                let thread_id = std::thread::current().id();
                let mut guard = thread_ids.lock().unwrap();
                if let Some(previous_id) = guard.as_ref() {
                    assert_ne!(previous_id, &thread_id);
                }
                *guard = Some(thread_id);
                i
            });
            assert_eq!(queue.next_result(), i);
        }

        queue.close();
    }

    #[test]
    fn is_parallel() {
        let queue = WorkerQueue::<u32>::new(2, 4);

        assert_duration(
            || {
                for id in 0..2 {
                    queue.enqueue(move || {
                        std::thread::sleep(Duration::from_secs(1));
                        id
                    });
                }

                let r1 = queue.next_result();
                let r2 = queue.next_result();
                assert!(r1 == 0 || r1 == 1);
                assert!(r2 == 0 || r2 == 1);
            },
            |d| d < 1.9,
        );

        queue.close();
    }

    #[test]
    fn earliest_first() {
        let queue = WorkerQueue::<u32>::new(4, 4);

        queue.enqueue(move || {
            std::thread::sleep(Duration::from_secs(1));
            1
        });
        queue.enqueue(move || {
            std::thread::sleep(Duration::from_millis(10));
            5
        });

        assert_eq!(queue.next_result(), 5);
        assert_eq!(queue.next_result(), 1);

        queue.close();
    }

    #[test]
    fn works_with_shared_ref() {
        // Make sure that we can't get &mut WorkerQueue
        let queue = Arc::new(WorkerQueue::<u32>::new(4, 4));

        // enqueuing work and reading results should be possible with only &WorkerQueue
        queue.enqueue(move || 1);
        assert_eq!(queue.next_result(), 1);

        Arc::into_inner(queue).unwrap().close();
    }

    #[test]
    fn many_enqueues() {
        let worker_count = 4;
        let queue_size = 8;
        let queue = WorkerQueue::<u32>::new(worker_count, queue_size);

        let mut inflight = 0;
        for id in 0..10000 {
            queue.enqueue(move || id);
            inflight += 1;

            // Avoid deadlock
            if inflight == queue_size {
                queue.next_result();
                inflight -= 1;
            }
        }
        for _ in 0..inflight {
            queue.next_result();
        }

        queue.close();
    }

    #[test]
    fn queue_size() {
        let queue = WorkerQueue::<u32>::new(2, 2);

        // This should fill the queue of each worker
        // 2 in queue + 1 being processed per worker
        for _ in 0..8 {
            queue.enqueue(|| {
                std::thread::sleep(Duration::from_secs(1));
                1
            });
        }
        assert_duration(
            || {
                queue.enqueue(|| 1);
            },
            |d| d > 0.1,
        );

        queue.close();
    }

    #[track_caller]
    fn assert_duration<F: FnOnce(), Check: FnOnce(f64) -> bool>(f: F, check: Check) {
        let start = Instant::now();
        f();
        let duration = start.elapsed().as_secs_f64();
        if !check(duration) {
            panic!("Duration {duration} did not pass check");
        }
    }
}