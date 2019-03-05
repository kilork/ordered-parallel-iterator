# ordered-parallel-iterator

This crate provides an iterator over task result which performs tasks in parallel returning completed tasks in order of source range iterator. It can be useful if you need to process some data in parallel but need to have results in order of appeareance (FIFO).

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

for i in OrderedParallelIterator::new(|| 0..10, || run_me) {
    println!("Result from iterator: {}", i);
}
```