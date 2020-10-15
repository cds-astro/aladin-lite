use {
    futures::{
        future::FutureExt,
        task::{ArcWake, waker_ref},
    },
    std::{
        future::Future,
        sync::Arc,
        task::{Context, Poll}
    },
};

struct Thread {
    _thread: std::thread::Thread,
}

use std::thread;
thread_local! {
    static CURRENT_THREAD_NOTIFY: Arc<Thread> = Arc::new(Thread {
        _thread: thread::current(),
    });
}

impl ArcWake for Thread {
    fn wake_by_ref(_arc_self: &Arc<Self>) {}
}

/// `Spawner` spawns new futures onto the task channel.
use std::collections::VecDeque;
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::pin::Pin;

use std::collections::HashMap;
// A queue that contains keyed element
struct KeyedVecDeque<K, V>
where K: Hash + Eq + Clone {
    keys: VecDeque<K>,
    values: HashMap<K, V>
}

use std::hash::Hash;
impl<K, V> KeyedVecDeque<K, V>
where K: Hash + Eq + Clone {
    fn new() -> Self {
        let keys = VecDeque::new();
        let values = HashMap::new();
        Self {
            keys,
            values
        }
    }

    fn push_front(&mut self, key: K, value: V) {
        self.keys.push_front(key.clone());
        self.values.insert(key, value);
    }

    fn pop_back(&mut self) -> Option<(K, V)> {
        if self.keys.is_empty() {
            None
        } else {
            let mut v = None;
            while !self.keys.is_empty() && v.is_none() {
                let k = self.keys.pop_back().unwrap();
                v = self.values.remove(&k).map(|v| (k, v));
            }

            v
        }
    }

    fn remove(&mut self, k: &K) -> Option<V> {
        self.values.remove(k)
    }
}

type Incoming<K, T> = RefCell<KeyedVecDeque<K, Pin<Box<dyn Future<Output=T> + 'static>>>>;

#[derive(Clone)]
pub struct Spawner<K, T>
where K: Hash + Eq + Clone {
    tasks: Weak<Incoming<K, T>>,
}

impl<K, T> Spawner<K, T>
where K: Hash + Eq + Clone {
    pub fn spawn(&mut self, key: K, future: impl Future<Output=T> + 'static) {
        let future = future.boxed_local();
        self.tasks.upgrade() // convert to Rc
            .unwrap()
            .borrow_mut() // Push the new task to the front of the queue
            .push_front(key, future);
    }
}

/// Task executor that receives tasks off of a channel and runs them.
pub struct Executor<K, T>
where K: Hash + Eq + Clone {
    tasks: Rc<Incoming<K, T>>,
    spawner: Spawner<K, T>,
}

impl<K, T> Default for Executor<K, T>
where K: Hash + Eq + Clone {
    fn default() -> Self {
        let tasks = Rc::new(RefCell::new(KeyedVecDeque::new()));
        let spawner = Spawner { 
            tasks: Rc::downgrade(&tasks),
        };
        Executor { tasks, spawner }
    }
}

impl<K, T> Executor<K, T>
where K: Hash + Eq + Clone + Sized {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawner(&mut self) -> &mut Spawner<K, T> {
        &mut self.spawner
    }

    pub fn run(&mut self, timeout: f32) -> Vec<T> {
        let mut results = vec![];

        CURRENT_THREAD_NOTIFY.with(|thread| {
            // Create a `LocalWaker` from the current thread itself
            let waker = waker_ref(thread);
            let mut cx = Context::from_waker(&waker);

            // Take all the task available from the channel
            // Exit the loop when either the channel is disconnected or
            // there are no tasks available to process.
            let mut tasks = self.tasks.borrow_mut();

            let window = web_sys::window().expect("should have a window in this context");
            let performance = window
                .performance()
                .expect("performance should be available");
            let start = performance.now() as f32;

            while let Some((k, mut task)) = tasks.pop_back() {
                // Take the future, and if it has not yet completed (is still Some),
                // poll it in an attempt to complete it.

                // We store `Pin<Box<dyn Future<Output = T> + 'static>>`.
                // We can get a `Pin<&mut dyn Future + 'static>`
                // from it by calling the `Pin::as_mut` method.
                let r = task.as_mut().poll(&mut cx);
                match r {
                    Poll::Pending => {
                        // Wake up the task pending immediately
                        //cx.waker().clone().wake();
                        // Reinsert not finished futures into the tasks queue
                        tasks.push_front(k, task);
                    },
                    Poll::Ready(result) => {
                        // If the future is completed, get the result
                        // and return it to the user
                        results.push(result);
                    },
                }
                let now = performance.now() as f32;
                // Break the running if we exceed the timeout
                if (now - start) >= timeout {
                    break;
                }
            }
        });

        results
    }

    // Remove a task from the executor so that
    // it won't be polled anymore
    pub fn remove(&mut self, k: &K) {
        self.tasks.borrow_mut().remove(k);
    }
}

#[cfg(test)]
mod tests {
    use super::Executor;
    use std::task::{Context, Poll};
    use std::pin::Pin;
    use futures::Future;
    // Define some futures to run concurrently on a single thread
    struct LearnTask(usize);
    impl LearnTask {
        fn new() -> Self {
            LearnTask(0)
        }
    }

    impl Future for LearnTask {
        type Output = u8;
        
        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
            self.0 += 1;
            println!("I'm learning {}", self.0);
            
            if self.0 == 10000 {
                Poll::Ready(0)
            } else {
                //cx.waker().clone().wake();
                Poll::Pending
            }
        }
    }


    struct DanceTask(usize);
    impl DanceTask {
        fn new() -> Self {
            DanceTask(0)
        }
    }
    
    impl Future for DanceTask {
        type Output = u8;
        
        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
            self.0 += 1;
            println!("I'm dancing {}", self.0);
            
            if self.0 == 10000 {
                Poll::Ready(0)
            } else {
                //cx.waker().clone().wake();
                Poll::Pending
            }
        }
    }

    #[test]
    fn it_works() {
        let executor = Executor::new();
        let spawner = executor.spawner();
        // Spawn a task to print before and after waiting on a timer.
        spawner.spawn(async {
            println!("LEARN begin!");
            // Wait for our timer future to complete after two seconds.
            LearnTask::new().await;
            println!("LEARN done!");

            10
        });
        spawner.spawn(async {
            println!("DANCE begin!");
            // Wait for our timer future to complete after two seconds.
            DanceTask::new().await;
            println!("DANCE done!");

            10
        });

        // Run the executor for a duration of 5 milliseconds
        executor.run(5_f32);
    }
    use futures::stream::Stream;
    
    pub struct ParseTable {
        table: Vec<u32>,
        ready: bool,
        idx: u32,
    }
    
    impl ParseTable {
        pub fn new(table: Vec<u32>) -> Self {
            let idx = 0;
            let ready = false;
            Self {
                table,
                idx,
                ready,
            }
        }
    }
    
    impl Stream for ParseTable {
        type Item = u32;
    
        /// Attempt to resolve the next item in the stream.
        /// Returns `Poll::Pending` if not ready, `Poll::Ready(Some(x))` if a value
        /// is ready, and `Poll::Ready(None)` if the stream has completed.
        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
            -> Poll<Option<Self::Item>> {
            // Deserialize row by row.
            let len = self.table.len();
            let idx = self.idx as usize;
            while self.idx < len as u32 {
                if !self.ready {
                    self.table[idx] += 10;
                    self.ready = true;
                    return Poll::Pending;
                } else {
                    println!("{}", idx);
                    let row = self.table[idx];

                    self.idx += 1;
                    self.ready = false;
                    return Poll::Ready(Some(row));
                }
            }
    
            Poll::Ready(None)
        }
    }
    use futures::stream::StreamExt; // for `next`
    #[test]
    fn it_works2() {
        let executor = Executor::new();
        let spawner = executor.spawner();
        // Spawn a task to print before and after waiting on a timer.
        spawner.spawn(async {
            println!("BEGIN parsing!");
            let mut stream = ParseTable::new((0..100000).collect());
            let mut results: Vec<u32> = vec![];
            while let Some(item) = stream.next().await {
                results.push(item);
            }
            println!("END parsing!");

            10
        });

        // Run the executor for a duration of 5 milliseconds
        executor.run(50_f32);
    }

}
