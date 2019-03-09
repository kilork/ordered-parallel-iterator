# ordered-parallel-iterator

This crate provides an iterator over task results which performs tasks in parallel returning completed tasks in order of source range iterator. It can be useful if you need to process some data in parallel but need to have results in the order of appearance (FIFO).

## Legal

Dual-licensed under MIT or the [UNLICENSE](http://unlicense.org/).

## Installation

Add following dependency to your `Cargo.toml`:

```toml,ignore
[dependencies]
ordered-parallel-iterator = "0.1"
```

## Usage

```rust
use ordered_parallel_iterator::OrderedParallelIterator;

fn run_me(x: usize) -> usize {
    x + 1
}

fn main() {
    for i in OrderedParallelIterator::new(|| 0..10, || run_me) {
        println!("Result from iterator: {}", i);
    }
}
```

In this example each `run_me` call will happen in own thread, but results will be returned sequentially as fast as first will be finished. Count of pending tasks running in parallel bind to count of CPU cores.