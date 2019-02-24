use crossbeam::channel::bounded;
use crossbeam::deque::{Steal, Stealer, Worker};
use std::sync::atomic::{AtomicBool, Ordering};

use std::marker::Send;
use std::marker::Sync;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use std_semaphore::Semaphore;

pub struct OrderedParallelIterator<O> {
    scheduler_thread: Option<JoinHandle<()>>,
    tasks: Stealer<JoinHandle<O>>,
    semaphore: Arc<Semaphore>,
    running: Arc<AtomicBool>,
}

impl<O> OrderedParallelIterator<O> {
    pub fn new<PC, XC, P, X, I>(producer_ctor: PC, xform_ctor: XC) -> Self
    where
        PC: 'static + Send + FnOnce() -> P,
        XC: 'static + Send + Sync + Fn() -> X,
        X: FnMut(I) -> O,
        I: 'static + Send,
        O: 'static + Send,
        P: IntoIterator<Item = I>,
    {
        let semaphore = Arc::new(Semaphore::new(num_cpus::get() as isize));
        let (tx, rx) = bounded(num_cpus::get());
        let semaphore_copy = semaphore.clone();
        let xform_ctor = Arc::new(xform_ctor);
        let running_flag = Arc::new(AtomicBool::new(true));
        let running = running_flag.clone();
        let scheduler_thread = Some(thread::spawn(move || {
            let tasks = Worker::new_fifo();
            let mut first = true;
            for e in producer_ctor() {
                semaphore_copy.acquire();
                let xform_ctor = xform_ctor.clone();
                let worker_thread = thread::spawn(move || {
                    let mut xform = xform_ctor();
                    xform(e)
                });
                tasks.push(worker_thread);
                if first {
                    let stealer = tasks.stealer();
                    tx.send(stealer).unwrap();
                    first = false;
                }
            }
            running_flag.store(false, Ordering::Relaxed);
            if first {
                // means empty range
                let stealer = tasks.stealer();
                tx.send(stealer).unwrap();
            }
        }));

        let tasks = rx.recv().unwrap();

        Self {
            scheduler_thread,
            tasks,
            semaphore,
            running,
        }
    }
}

impl<T> Iterator for OrderedParallelIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.semaphore.release();
        loop {
            let item = self.tasks.steal();
            match item {
                Steal::Success(x) => {
                    return Some(x.join().expect("Cannot get data from thread"));
                }
                Steal::Empty => {
                    if !self.running.load(Ordering::Relaxed) {
                        break;
                    }
                }
                Steal::Retry => (),
            }
        }

        self.scheduler_thread
            .take()
            .unwrap()
            .join()
            .expect("The scheduler thread has paniced.");

        None
    }
}

#[cfg(test)]
mod tests {

    fn run_me(x: usize) -> usize {
        x + 1
    }

    #[test]
    fn it_works() {
        let mut iterator = crate::OrderedParallelIterator::new(|| 0..10, || run_me);
        for i in 0..10 {
            assert_eq!(iterator.next(), Some(i + 1));
        }
    }

    #[test]
    fn empty() {
        for _ in crate::OrderedParallelIterator::new(|| 0..0, || run_me) {
            panic!("Must not reach this point");
        }
    }

}
