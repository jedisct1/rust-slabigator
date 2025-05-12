use slabigator::Slab;
use std::time::Instant;

fn main() {
    // Parameters
    const CAPACITY: usize = 10_000;
    const OPERATIONS: usize = 100_000;

    println!("Benchmarking Slabigator with capacity {}", CAPACITY);
    println!("------------------------------------------");

    // Create a slab with the specified capacity
    let mut slab = Slab::with_capacity(CAPACITY).expect("Failed to create slab");

    // Benchmark push_front
    let start = Instant::now();
    let mut slots = Vec::with_capacity(CAPACITY);

    for i in 0..CAPACITY {
        slots.push(slab.push_front(i).expect("Failed to push"));
    }

    let push_time = start.elapsed();
    println!(
        "push_front: {:?} total, {:?} per operation",
        push_time,
        push_time / CAPACITY as u32
    );

    // Benchmark get
    let start = Instant::now();
    let mut sum = 0;

    for _ in 0..OPERATIONS {
        let idx = fastrand::usize(0..slots.len());
        if let Ok(value) = slab.get(slots[idx]) {
            sum += *value;
        }
    }

    let get_time = start.elapsed();
    println!(
        "get: {:?} total, {:?} per operation",
        get_time,
        get_time / OPERATIONS as u32
    );
    println!("(Sum: {} - just to prevent optimization)", sum);

    // Benchmark remove and push (reuse)
    let start = Instant::now();

    for i in 0..OPERATIONS.min(CAPACITY) {
        let idx = i % slots.len();
        let _ = slab.remove(slots[idx]);
        slots[idx] = slab.push_front(i).expect("Failed to push after remove");
    }

    let reuse_time = start.elapsed();
    println!(
        "remove+push_front: {:?} total, {:?} per operation",
        reuse_time,
        reuse_time / OPERATIONS.min(CAPACITY) as u32
    );

    // Benchmark iteration
    let start = Instant::now();
    let mut sum = 0;

    for _ in 0..100 {
        for value in slab.iter() {
            sum += *value;
        }
    }

    let iter_time = start.elapsed();
    println!(
        "iteration: {:?} total, {:?} per full iteration",
        iter_time,
        iter_time / 100
    );
    println!("(Sum: {} - just to prevent optimization)", sum);
}
