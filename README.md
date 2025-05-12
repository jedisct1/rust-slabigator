[![CI](https://github.com/jedisct1/rust-slabigator/actions/workflows/ci.yml/badge.svg)](https://github.com/jedisct1/rust-slabigator/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/slabigator.svg)](https://crates.io/crates/slabigator)
[![Documentation](https://docs.rs/slabigator/badge.svg)](https://docs.rs/slabigator)

# Slabigator

A high-performance linked list that doesn't perform dynamic memory allocations after initialization. It allocates all necessary memory upfront with a fixed capacity, making it ideal for performance-critical applications where memory allocation patterns need to be predictable.

## Features

Slabigator was designed to do a few things extremely well:

- **Add to the head** of the list in O(1) time - Returns a stable slot number for future reference
- **Pop from the tail** of the list in O(1) time
- **Delete any element** given its slot number in O(1) time
- **Fixed memory allocation** - No dynamic allocations after initialization

It's designed to be:
- **Fast**: O(1) operations with minimal overhead
- **Predictable**: No memory allocations during operations
- **Simple**: Small, focused API for specific use cases
- **Maintainable**: Small codebase with zero dependencies

## Usage

```rust
use slabigator::Slab;

// Create a new slab with a capacity of 3 elements
let mut slab = Slab::with_capacity(3).unwrap();

// Add elements to the front - each operation returns a slot number
let slot_a = slab.push_front("a").unwrap();
let slot_b = slab.push_front("b").unwrap();
let slot_c = slab.push_front("c").unwrap();

// Slab is now full (capacity = 3)
assert!(slab.is_full());
assert_eq!(slab.len(), 3);

// Access elements directly by their slots
assert_eq!(slab.get(slot_a).unwrap(), &"a");
assert_eq!(slab.get(slot_b).unwrap(), &"b");
assert_eq!(slab.get(slot_c).unwrap(), &"c");

// Remove an element by its slot
slab.remove(slot_b).unwrap();
assert_eq!(slab.len(), 2);

// Pop elements from the back (FIFO behavior)
assert_eq!(slab.pop_back().unwrap(), "a");
assert_eq!(slab.pop_back().unwrap(), "c");
assert!(slab.is_empty());
```

## When to Use Slabigator

Slabigator is ideal for scenarios where:

- You need to maintain a list with stable references to elements
- Memory allocation predictability is critical
- You need fast (O(1)) operations for all common list operations
- You know the maximum size of the list in advance
- You need a simple FIFO queue with the ability to remove arbitrary elements

Common use cases include:
- Real-time systems where allocation jitter is problematic
- Memory-constrained environments
- High-performance queues for task management
- Cache implementations with fixed capacity
- Game development for object pools

## Cargo Features

Slabigator comes with several feature flags to customize its behavior:

- `releasefast`: Assumes that `remove()` will always be called with a valid index. This saves memory by removing validation checks, but must be used with extreme caution. Disabled by default.
- `slot_u32`: Uses `u32` as the slot type (default)
- `slot_u64`: Uses `u64` as the slot type
- `slot_usize`: Uses `usize` as the slot type

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
slabigator = "0.9"
```

## Trait Implementations

Slabigator implements several useful Rust traits:

- `Default`: Creates a slab with a default capacity of 16
- `FromIterator<T>`: Creates a slab from any iterator
- `Extend<T>`: Extends a slab with elements from an iterator
- `Index<Slot>` and `IndexMut<Slot>`: Allows direct indexing with `[]` syntax
- `IntoIterator` for `&Slab<T>`: Enables iteration with `for` loops
