use ordered_parallel_iterator::OrderedParallelIterator;

fn run_me(x: usize) -> usize {
    eprintln!("start {}", x);
    sleep_secs(30 - x);
    eprintln!("end {}", x);
    x
}

fn sleep_secs(secs: usize) {
    std::thread::sleep(std::time::Duration::from_secs(secs as u64));
}

fn main() {
    for i in crate::OrderedParallelIterator::new(|| 0..30, || run_me) {
        eprintln!("consume {}", i);
        sleep_secs(i);
        eprintln!("wake up and took next value");
    }
}
