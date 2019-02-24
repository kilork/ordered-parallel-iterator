use crossbeam::channel::bounded;
use crossbeam::deque::{Steal, Stealer, Worker};

use std::marker::Send;
use std::marker::Sync;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

use std_semaphore::Semaphore;

pub struct OrderedParallelIterator<O> {
    iterator_thread: Option<JoinHandle<()>>,
    scheduler_thread: Option<JoinHandle<()>>,
    tasks: Stealer<JoinHandle<O>>,
    semaphore: Arc<Semaphore>,
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
        let (tx, jobs_rx) = bounded(num_cpus::get());
        let semaphore_copy = semaphore.clone();
        let iterator_thread = Some(thread::spawn(move || {
            for e in producer_ctor() {
                semaphore_copy.acquire();
                tx.send(e).expect("Producer thread failed to send job.");
            }
        }));

        let xform_ctor = Arc::new(xform_ctor);
        let (tx, rx) = std::sync::mpsc::channel();
        let scheduler_thread = Some(thread::spawn(move || {
            let tasks = Worker::new_fifo();
            let stealer = tasks.stealer();
            tx.send(stealer).unwrap();
            for e in jobs_rx {
                let xform_ctor = xform_ctor.clone();
                let worker_thread = thread::spawn(move || {
                    let mut xform = xform_ctor();
                    xform(e)
                });
                tasks.push(worker_thread);
            }
        }));

        let tasks = rx.recv().unwrap();

        Self {
            iterator_thread,
            scheduler_thread,
            tasks,
            semaphore,
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
                    return Some(x.join().expect("Cannot get changeset from thread"));
                }
                Steal::Empty => break,
                Steal::Retry => (),
            }
        }

        self.iterator_thread
            .take()
            .unwrap()
            .join()
            .expect("The iterator thread has paniced.");

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
}
