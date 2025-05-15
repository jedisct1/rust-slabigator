#![doc = include_str!("../README.md")]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cast_possible_truncation)]

//! # Slabigator
//!
//! A high-performance linked list with fixed capacity that doesn't perform dynamic memory
//! allocations after initialization. It provides O(1) operations for adding, removing,
//! and accessing elements.
//!
//! ## Overview
//!
//! Slabigator is designed for scenarios where memory allocation predictability is critical
//! and you need stable references to elements. It allocates all memory upfront and provides
//! slot numbers as stable references to elements.
//!
//! ## Key Features
//!
//! - Fixed capacity - no allocations during operations
//! - O(1) push_front operation
//! - O(1) pop_back operation
//! - O(1) removal of any element by slot
//! - Slots provide stable references to elements
//! - Implements useful Rust traits like `FromIterator` and `Extend`
//!
//! ## Basic Usage
//!
//! ```rust
//! use slabigator::Slab;
//!
//! // Create a slab with capacity for 3 elements
//! let mut slab = Slab::with_capacity(3).unwrap();
//!
//! // Push elements to the front (returns slot numbers)
//! let a = slab.push_front("a").unwrap();
//! let b = slab.push_front("b").unwrap();
//! let c = slab.push_front("c").unwrap();
//!
//! // Access by slot
//! assert_eq!(slab.get(a).unwrap(), &"a");
//! assert_eq!(slab.get(b).unwrap(), &"b");
//! assert_eq!(slab.get(c).unwrap(), &"c");
//!
//! // Remove an element
//! slab.remove(b).unwrap();
//! assert_eq!(slab.len(), 2);
//!
//! // Iterate (order is from head to tail)
//! let elements: Vec<_> = slab.iter().collect();
//! assert_eq!(elements, vec![&"c", &"a"]);
//!
//! // Pop from the back (FIFO queue behavior)
//! assert_eq!(slab.pop_back().unwrap(), "a");
//! assert_eq!(slab.pop_back().unwrap(), "c");
//! assert!(slab.is_empty());
//! ```
//!
//! ## Advanced Features
//!
//! ### Default capacity
//!
//! ```rust
//! use slabigator::Slab;
//!
//! // Creates a slab with default capacity (16)
//! let slab: Slab<i32> = Slab::default();
//! assert_eq!(slab.capacity(), 16);
//! ```
//!
//! ### Creating from an iterator
//!
//! ```rust
//! use slabigator::Slab;
//!
//! let values = vec![1, 2, 3, 4, 5];
//! let slab: Slab<_> = values.iter().copied().collect();
//!
//! assert_eq!(slab.len(), 5);
//! ```
//!
//! ### Extending a slab
//!
//! ```rust
//! use slabigator::Slab;
//!
//! let mut slab = Slab::with_capacity(5).unwrap();
//! slab.push_front(1).unwrap();
//! slab.push_front(2).unwrap();
//!
//! // Extend with more elements
//! slab.extend(vec![3, 4, 5]);
//! assert_eq!(slab.len(), 5);
//! ```

use std::{iter::Iterator, mem::MaybeUninit};

#[cfg(all(feature = "slot_u64", not(feature = "slot_usize")))]
/// Slot type used for element references.
/// This is u64 when the `slot_u64` feature is enabled.
pub type Slot = u64;
#[cfg(feature = "slot_usize")]
/// Slot type used for element references.
/// This is usize when the `slot_usize` feature is enabled.
pub type Slot = usize;
#[cfg(not(any(
    all(feature = "slot_u64", not(feature = "slot_usize")),
    feature = "slot_usize"
)))]
/// Slot type used for element references.
/// This is u32 by default or when the `slot_u32` feature is enabled.
pub type Slot = u32;

const NUL: Slot = Slot::MAX;

/// A fixed-capacity linked list that doesn't perform dynamic memory allocations after initialization.
///
/// # Overview
///
/// `Slab<D>` is a specialized data structure that allocates all of its memory upfront and provides
/// stable slot numbers as references to elements. This makes it ideal for performance-critical
/// applications where:
///
/// - Memory allocation patterns need to be predictable
/// - You need stable references to elements
/// - Fast O(1) operations are required
/// - You know the maximum capacity in advance
///
/// # Core Operations
///
/// - **`push_front`**: Add elements to the head of the list in O(1) time
/// - **`pop_back`**: Remove and return an element from the tail in O(1) time
/// - **remove**: Delete any element by its slot number in O(1) time
/// - **`get/get_mut`**: Access any element by its slot number in O(1) time
///
/// # Memory Behavior
///
/// The slab allocates all memory during creation with `with_capacity()`. No further allocations
/// occur during subsequent operations. This provides predictable memory usage and avoids
/// allocation-related performance issues.
///
/// # Implementation Details
///
/// Internally, the slab maintains:
/// - A vector of elements
/// - A linked list structure for tracking the order of elements
/// - A free list for quick reuse of slots
/// - A bitmap for validating slot access (when not using `releasefast` feature)
///
/// # Examples
///
/// ## Basic Operations
///
/// ```
/// use slabigator::Slab;
///
/// // Create a new slab with capacity for 3 elements
/// let mut slab = Slab::with_capacity(3).unwrap();
///
/// // Push elements to the front - each operation returns a slot number
/// // The slot numbers are stable references that won't change
/// // even when other elements are added or removed
/// let slot_a = slab.push_front("a").unwrap();
/// let slot_b = slab.push_front("b").unwrap();
/// let slot_c = slab.push_front("c").unwrap();
///
/// // Slab is now full
/// assert!(slab.is_full());
/// assert_eq!(slab.len(), 3);
///
/// // Access elements by slot - these are direct lookups
/// assert_eq!(slab.get(slot_a).unwrap(), &"a");
/// assert_eq!(slab.get(slot_b).unwrap(), &"b");
/// assert_eq!(slab.get(slot_c).unwrap(), &"c");
///
/// // Remove an element by slot
/// slab.remove(slot_b).unwrap();
/// assert_eq!(slab.len(), 2);
/// #[cfg(not(feature = "releasefast"))]
/// assert!(slab.get(slot_b).is_err()); // Slot b is no longer valid
///
/// // Pop elements from the back (FIFO order)
/// let value = slab.pop_back().unwrap();
/// assert_eq!(value, "a");
/// assert_eq!(slab.len(), 1);
///
/// let value = slab.pop_back().unwrap();
/// assert_eq!(value, "c");
/// assert!(slab.is_empty());
/// ```
///
/// ## Using as a FIFO Queue
///
/// ```
/// use slabigator::Slab;
///
/// let mut queue = Slab::with_capacity(10).unwrap();
///
/// // Enqueue items (push to front)
/// queue.push_front("first").unwrap();
/// queue.push_front("second").unwrap();
/// queue.push_front("third").unwrap();
///
/// // Dequeue items (pop from back) - FIFO order
/// assert_eq!(queue.pop_back().unwrap(), "first");
/// assert_eq!(queue.pop_back().unwrap(), "second");
/// assert_eq!(queue.pop_back().unwrap(), "third");
/// ```
///
/// ## Using for Object Pooling
///
/// ```
/// use slabigator::Slab;
///
/// #[derive(Debug, PartialEq)]
/// struct GameObject {
///     id: u32,
///     active: bool,
/// }
///
/// // Create a pool of game objects
/// let mut pool = Slab::with_capacity(100).unwrap();
///
/// // Create and add objects to the pool
/// let slot1 = pool.push_front(GameObject { id: 1, active: true }).unwrap();
/// let slot2 = pool.push_front(GameObject { id: 2, active: true }).unwrap();
///
/// // Deactivate an object (by mutating it)
/// if let Ok(object) = pool.get_mut(slot1) {
///     object.active = false;
/// }
///
/// // Remove an object from the pool when no longer needed
/// pool.remove(slot2).unwrap();
/// ```
#[derive(Debug)]
pub struct Slab<D: Sized> {
    vec_next: Vec<Slot>,
    vec_prev: Vec<Slot>,
    free_head: Slot,
    head: Slot,
    tail: Slot,
    len: usize,
    data: Vec<MaybeUninit<D>>,
    #[cfg(not(feature = "releasefast"))]
    bitmap: Vec<u8>,
}

/// Error types that can occur during Slab operations.
///
/// The Slab API follows a design philosophy where operations that could fail return
/// a `Result<T, Error>` rather than panicking. This allows error handling to be more
/// explicit and gives the caller control over how to handle error conditions.
///
/// # Examples
///
/// ```
/// use slabigator::{Slab, Error};
///
/// let mut slab = Slab::with_capacity(1).unwrap();
/// slab.push_front("only element").unwrap();
///
/// // Attempt to add another element when slab is full
/// match slab.push_front("one too many") {
///     Ok(_) => println!("Element added successfully"),
///     Err(Error::Full) => println!("Cannot add element: slab is full"),
///     Err(_) => println!("Other error occurred"),
/// }
///
/// // Attempt to access an invalid slot
/// match slab.get(999) {
///     Ok(_) => println!("Element retrieved"),
///     Err(Error::InvalidSlot) => println!("Invalid slot"),
///     Err(_) => println!("Other error occurred"),
/// }
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Error {
    /// Returned when the requested capacity during creation is too large for the
    /// selected slot type. This occurs when the capacity would cause the slot index
    /// to exceed the maximum value for the slot type (u32, u64, or usize).
    TooLarge,

    /// Returned when attempting to add an element to a slab that already contains
    /// its maximum capacity of elements. Check `is_full()` before adding elements
    /// or handle this error to implement graceful fallbacks.
    Full,

    /// Returned when:
    /// - Accessing a slot that is out of bounds (>= capacity)
    /// - Accessing a slot that doesn't contain an element (was never set or was removed)
    /// - Attempting to remove an element from a slot that is invalid
    ///
    /// When not using the `releasefast` feature, all slot validity is checked.
    InvalidSlot,

    /// Returned when attempting to access or remove elements from an empty slab.
    /// Check `is_empty()` before these operations or handle this error appropriately.
    Empty,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::TooLarge => write!(f, "Capacity is too large for the slot type"),
            Error::Full => write!(f, "Slab is full and cannot accept more elements"),
            Error::InvalidSlot => write!(f, "Invalid slot or slot doesn't contain an element"),
            Error::Empty => write!(f, "Slab is empty"),
        }
    }
}

impl<D: Sized> Slab<D> {
    /// Creates a new slab with the given capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The maximum number of elements the slab can hold.
    ///
    /// # Returns
    ///
    /// * `Ok(Slab<D>)` - A new slab with the requested capacity.
    /// * `Err(Error::TooLarge)` - If the capacity is too large for the slot type.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let slab = Slab::<String>::with_capacity(10).unwrap();
    /// assert_eq!(slab.capacity(), 10);
    /// assert_eq!(slab.len(), 0);
    /// assert!(slab.is_empty());
    /// ```
    pub fn with_capacity(capacity: usize) -> Result<Self, Error> {
        if capacity as Slot == NUL {
            return Err(Error::TooLarge);
        }
        let mut vec_next = Vec::with_capacity(capacity);
        for i in 0..(capacity - 1) {
            vec_next.push(i as Slot + 1);
        }
        vec_next.push(NUL);
        let mut vec_prev = Vec::with_capacity(capacity);
        vec_prev.push(NUL);
        for i in 1..capacity {
            vec_prev.push(i as Slot - 1);
        }
        let mut data = Vec::with_capacity(capacity);
        unsafe { data.set_len(capacity) };

        #[cfg(not(feature = "releasefast"))]
        let bitmap_size = (capacity + 7) / 8; // TODO: Replace with capacity.div_ceil(8) when stable

        Ok(Self {
            vec_next,
            vec_prev,
            free_head: 0,
            head: NUL,
            tail: NUL,
            len: 0,
            data,
            #[cfg(not(feature = "releasefast"))]
            bitmap: vec![0u8; bitmap_size],
        })
    }

    /// Returns the capacity of the slab.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let slab = Slab::<i32>::with_capacity(10).unwrap();
    /// assert_eq!(slab.capacity(), 10);
    /// ```
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Returns the number of elements in the slab.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(10).unwrap();
    /// assert_eq!(slab.len(), 0);
    ///
    /// slab.push_front(42).unwrap();
    /// assert_eq!(slab.len(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the number of elements that can still be stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(3).unwrap();
    /// assert_eq!(slab.free(), 3);
    ///
    /// slab.push_front(42).unwrap();
    /// assert_eq!(slab.free(), 2);
    /// ```
    #[inline]
    #[must_use]
    pub fn free(&self) -> usize {
        self.capacity() - self.len()
    }

    /// Returns `true` if the slab contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(10).unwrap();
    /// assert!(slab.is_empty());
    ///
    /// slab.push_front(42).unwrap();
    /// assert!(!slab.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the slab cannot hold any more elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(2).unwrap();
    /// assert!(!slab.is_full());
    ///
    /// slab.push_front(1).unwrap();
    /// slab.push_front(2).unwrap();
    /// assert!(slab.is_full());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.free_head == NUL
    }

    /// Returns a reference to an element given its slot number.
    ///
    /// # Safety
    ///
    /// If the crate is compiled with the `releasefast` feature (which is not the
    /// case by default), `get()` should never be called on a slot index that
    /// was not set.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot number of the element to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(&D)` - A reference to the element.
    /// * `Err(Error::InvalidSlot)` - If the slot is invalid or doesn't contain an element.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(10).unwrap();
    /// let slot = slab.push_front("hello").unwrap();
    ///
    /// assert_eq!(slab.get(slot).unwrap(), &"hello");
    /// ```
    pub fn get(&self, slot: Slot) -> Result<&D, Error> {
        if slot.as_index() >= self.capacity() {
            return Err(Error::InvalidSlot);
        }
        #[cfg(not(feature = "releasefast"))]
        {
            if !self.bitmap_get(slot) {
                return Err(Error::InvalidSlot);
            }
        }
        Ok(unsafe { self.data[slot.as_index()].assume_init_ref() })
    }

    /// Returns a mutable reference to an element given its slot number.
    ///
    /// # Safety
    ///
    /// If the crate is compiled with the `releasefast` feature (which is not the
    /// case by default), `get_mut()` should never be called on a slot index that
    /// was not set.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot number of the element to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(&mut D)` - A mutable reference to the element.
    /// * `Err(Error::InvalidSlot)` - If the slot is invalid or doesn't contain an element.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(10).unwrap();
    /// let slot = slab.push_front("hello").unwrap();
    ///
    /// *slab.get_mut(slot).unwrap() = "world";
    /// assert_eq!(slab.get(slot).unwrap(), &"world");
    /// ```
    pub fn get_mut(&mut self, slot: Slot) -> Result<&mut D, Error> {
        if slot.as_index() >= self.capacity() {
            return Err(Error::InvalidSlot);
        }
        #[cfg(not(feature = "releasefast"))]
        {
            if !self.bitmap_get(slot) {
                return Err(Error::InvalidSlot);
            }
        }
        Ok(unsafe { self.data[slot.as_index()].assume_init_mut() })
    }

    /// Prepends an element to the beginning of the slab.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to prepend.
    ///
    /// # Returns
    ///
    /// * `Ok(Slot)` - The slot number of the newly added element.
    /// * `Err(Error::Full)` - If the slab is full.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(3).unwrap();
    ///
    /// let a = slab.push_front("a").unwrap();
    /// let b = slab.push_front("b").unwrap();
    /// let c = slab.push_front("c").unwrap();
    ///
    /// // Elements are in reverse order of insertion
    /// let mut iter = slab.iter();
    /// assert_eq!(iter.next(), Some(&"c"));
    /// assert_eq!(iter.next(), Some(&"b"));
    /// assert_eq!(iter.next(), Some(&"a"));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn push_front(&mut self, value: D) -> Result<Slot, Error> {
        let free_slot = self.free_head;
        if free_slot == NUL {
            return Err(Error::Full);
        }
        let prev = self.vec_prev[free_slot.as_index()];
        let next = self.vec_next[free_slot.as_index()];
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev.as_index()], free_slot);
            self.vec_next[prev.as_index()] = next;
        }
        if next != NUL {
            if !self.is_empty() {
                debug_assert_eq!(self.vec_prev[next.as_index()], free_slot);
            }
            self.vec_prev[next.as_index()] = prev;
        }
        if self.head != NUL {
            self.vec_prev[self.head.as_index()] = free_slot;
        }
        self.free_head = next;
        self.vec_next[free_slot.as_index()] = self.head;
        self.vec_prev[free_slot.as_index()] = NUL;
        if self.head == NUL {
            self.tail = free_slot;
        }
        self.head = free_slot;

        self.data[free_slot.as_index()] = MaybeUninit::new(value);
        self.len += 1;
        debug_assert!(self.len <= self.capacity());
        #[cfg(not(feature = "releasefast"))]
        {
            self.bitmap_set(free_slot);
        }
        Ok(free_slot)
    }

    /// Removes an element from the slab given its slot.
    ///
    /// # Safety
    ///
    /// If the crate is compiled with the `releasefast` feature (which is not the
    /// case by default), `remove()` should never be called on a slot index that
    /// was already removed.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot number of the element to remove.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the element was successfully removed.
    /// * `Err(Error::InvalidSlot)` - If the slot is invalid or doesn't contain an element.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(3).unwrap();
    /// let a = slab.push_front("a").unwrap();
    /// let b = slab.push_front("b").unwrap();
    /// let c = slab.push_front("c").unwrap();
    ///
    /// assert_eq!(slab.len(), 3);
    ///
    /// slab.remove(b).unwrap();
    /// assert_eq!(slab.len(), 2);
    ///
    /// // The element at slot `b` is no longer accessible
    /// #[cfg(not(feature = "releasefast"))]
    /// assert!(slab.get(b).is_err());
    /// ```
    pub fn remove(&mut self, slot: Slot) -> Result<(), Error> {
        if slot.as_index() >= self.capacity() {
            return Err(Error::InvalidSlot);
        }
        #[cfg(not(feature = "releasefast"))]
        {
            if !self.bitmap_get(slot) {
                return Err(Error::InvalidSlot);
            }
        }
        unsafe { self.data[slot.as_index()].assume_init_drop() };
        self.data[slot.as_index()] = MaybeUninit::uninit();
        let prev = self.vec_prev[slot.as_index()];
        let next = self.vec_next[slot.as_index()];
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev.as_index()], slot);
            self.vec_next[prev.as_index()] = next;
        }
        if next != NUL {
            if !self.is_empty() {
                debug_assert_eq!(self.vec_prev[next.as_index()], slot);
            }
            self.vec_prev[next.as_index()] = prev;
        }
        if self.tail == slot {
            self.tail = prev;
        }
        if self.head == slot {
            self.head = next;
        }
        self.vec_prev[slot.as_index()] = NUL;
        self.vec_next[slot.as_index()] = self.free_head;
        if self.free_head != NUL {
            self.vec_prev[self.free_head.as_index()] = slot;
        }
        self.free_head = slot;
        debug_assert!(self.len > 0);
        self.len -= 1;
        #[cfg(not(feature = "releasefast"))]
        {
            self.bitmap_unset(slot);
        }
        Ok(())
    }

    /// Removes and returns the tail element of the slab.
    ///
    /// # Returns
    ///
    /// * `Some(D)` - The removed element.
    /// * `None` - If the slab is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(3).unwrap();
    /// slab.push_front("a").unwrap();
    /// slab.push_front("b").unwrap();
    /// slab.push_front("c").unwrap();
    ///
    /// assert_eq!(slab.pop_back(), Some("a"));
    /// assert_eq!(slab.pop_back(), Some("b"));
    /// assert_eq!(slab.pop_back(), Some("c"));
    /// assert_eq!(slab.pop_back(), None);
    /// ```
    pub fn pop_back(&mut self) -> Option<D> {
        let slot = self.tail;
        if slot == NUL {
            return None;
        }
        let value = unsafe { self.data[slot.as_index()].assume_init_read() };
        self.data[slot.as_index()] = MaybeUninit::uninit();
        let prev = self.vec_prev[slot.as_index()];
        debug_assert_eq!(self.vec_next[slot.as_index()], NUL);
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev.as_index()], slot);
            self.vec_next[prev.as_index()] = NUL;
        }
        self.tail = prev;
        if self.head == slot {
            self.head = NUL;
        }
        self.vec_prev[slot.as_index()] = NUL;
        self.vec_next[slot.as_index()] = self.free_head;
        if self.free_head != NUL {
            self.vec_prev[self.free_head.as_index()] = slot;
        }
        self.free_head = slot;
        debug_assert!(self.len > 0);
        self.len -= 1;
        #[cfg(not(feature = "releasefast"))]
        {
            self.bitmap_unset(slot);
        }
        Some(value)
    }

    /// Removes and returns a reference to the tail element of the slab.
    ///
    /// # Returns
    ///
    /// * `Some(&D)` - A reference to the removed element.
    /// * `None` - If the slab is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(2).unwrap();
    /// slab.push_front("a").unwrap();
    /// slab.push_front("b").unwrap();
    ///
    /// let last = slab.pop_back_ref();
    /// assert_eq!(last, Some(&"a"));
    /// ```
    pub fn pop_back_ref(&mut self) -> Option<&D> {
        let slot = self.tail;
        if slot == NUL {
            return None;
        }
        let value = unsafe { self.data[slot.as_index()].assume_init_ref() };
        let prev = self.vec_prev[slot.as_index()];
        debug_assert_eq!(self.vec_next[slot.as_index()], NUL);
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev.as_index()], slot);
            self.vec_next[prev.as_index()] = NUL;
        }
        self.tail = prev;
        if self.head == slot {
            self.head = NUL;
        }
        self.vec_prev[slot.as_index()] = NUL;
        self.vec_next[slot.as_index()] = self.free_head;
        if self.free_head != NUL {
            self.vec_prev[self.free_head.as_index()] = slot;
        }
        self.free_head = slot;
        debug_assert!(self.len > 0);
        self.len -= 1;
        Some(value)
    }

    /// Removes and returns a mutable reference to the tail element of the slab.
    ///
    /// # Returns
    ///
    /// * `Some(&mut D)` - A mutable reference to the removed element.
    /// * `None` - If the slab is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(2).unwrap();
    /// slab.push_front("a").unwrap();
    /// slab.push_front("b").unwrap();
    ///
    /// let last = slab.pop_back_ref_mut();
    /// assert_eq!(last, Some(&mut "a"));
    /// ```
    pub fn pop_back_ref_mut(&mut self) -> Option<&mut D> {
        let slot = self.tail;
        if slot == NUL {
            return None;
        }
        let value = unsafe { self.data[slot.as_index()].assume_init_mut() };
        let prev = self.vec_prev[slot.as_index()];
        debug_assert_eq!(self.vec_next[slot.as_index()], NUL);
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev.as_index()], slot);
            self.vec_next[prev.as_index()] = NUL;
        }
        self.tail = prev;
        if self.head == slot {
            self.head = NUL;
        }
        self.vec_prev[slot.as_index()] = NUL;
        self.vec_next[slot.as_index()] = self.free_head;
        if self.free_head != NUL {
            self.vec_prev[self.free_head.as_index()] = slot;
        }
        self.free_head = slot;
        debug_assert!(self.len > 0);
        self.len -= 1;
        Some(value)
    }

    /// Returns an iterator over the elements of the slab.
    ///
    /// The iterator yields elements in order from head to tail.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(3).unwrap();
    /// slab.push_front("a").unwrap();
    /// slab.push_front("b").unwrap();
    /// slab.push_front("c").unwrap();
    ///
    /// let mut iter = slab.iter();
    /// assert_eq!(iter.next(), Some(&"c"));
    /// assert_eq!(iter.next(), Some(&"b"));
    /// assert_eq!(iter.next(), Some(&"a"));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[must_use]
    pub fn iter(&self) -> SlabIterator<D> {
        SlabIterator {
            list: self,
            slot: None,
        }
    }

    /// Checks if the slot contains an element.
    ///
    /// This method is only available when not using the `releasefast` feature.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot to check.
    ///
    /// # Returns
    ///
    /// * `true` - If the slot contains an element.
    /// * `false` - If the slot is invalid or doesn't contain an element.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(3).unwrap();
    /// let slot = slab.push_front("hello").unwrap();
    ///
    /// assert!(slab.contains_slot(slot));
    ///
    /// slab.remove(slot).unwrap();
    /// assert!(!slab.contains_slot(slot));
    /// ```
    #[cfg(not(feature = "releasefast"))]
    #[must_use]
    pub fn contains_slot(&self, slot: Slot) -> bool {
        if slot.as_index() >= self.capacity() {
            return false;
        }
        self.bitmap_get(slot)
    }

    #[cfg(not(feature = "releasefast"))]
    #[inline]
    fn bitmap_get(&self, slot: Slot) -> bool {
        (self.bitmap[slot.as_index() / 8] & (1 << (slot.as_index() & 7))) != 0
    }

    #[cfg(not(feature = "releasefast"))]
    #[inline]
    fn bitmap_set(&mut self, slot: Slot) {
        self.bitmap[slot.as_index() / 8] |= 1 << (slot.as_index() & 7);
    }

    #[cfg(not(feature = "releasefast"))]
    #[inline]
    fn bitmap_unset(&mut self, slot: Slot) {
        self.bitmap[slot.as_index() / 8] &= !(1 << (slot.as_index() & 7));
    }

    /// Clears the slab, removing all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(3).unwrap();
    /// slab.push_front("a").unwrap();
    /// slab.push_front("b").unwrap();
    ///
    /// assert_eq!(slab.len(), 2);
    /// slab.clear();
    /// assert_eq!(slab.len(), 0);
    /// assert!(slab.is_empty());
    /// ```
    pub fn clear(&mut self) {
        // Drop all elements
        let mut slot = self.head;
        while slot != NUL {
            let next = self.vec_next[slot.as_index()];
            unsafe { self.data[slot.as_index()].assume_init_drop() };
            self.data[slot.as_index()] = MaybeUninit::uninit();
            #[cfg(not(feature = "releasefast"))]
            {
                self.bitmap_unset(slot);
            }
            slot = next;
        }

        // Reset the slab state
        let capacity = self.capacity();
        for i in 0..(capacity - 1) {
            self.vec_next[i] = i as Slot + 1;
        }
        self.vec_next[capacity - 1] = NUL;

        self.vec_prev[0] = NUL;
        for i in 1..capacity {
            self.vec_prev[i] = i as Slot - 1;
        }

        self.free_head = 0;
        self.head = NUL;
        self.tail = NUL;
        self.len = 0;
    }
}

impl<D> Default for Slab<D> {
    /// Creates a new empty slab with a default capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let slab: Slab<i32> = Slab::default();
    /// assert!(slab.is_empty());
    /// assert_eq!(slab.capacity(), 16);
    /// ```
    fn default() -> Self {
        Self::with_capacity(16).expect("Default capacity should always be valid")
    }
}

impl<D> Drop for Slab<D> {
    fn drop(&mut self) {
        let mut slot = self.head;
        while slot != NUL {
            let next = self.vec_next[slot.as_index()];
            unsafe { self.data[slot.as_index()].assume_init_drop() };
            slot = next;
        }
    }
}

// Cast Slot to usize for indexing
trait SlotIndex {
    fn as_index(&self) -> usize;
}

impl SlotIndex for u32 {
    #[inline]
    fn as_index(&self) -> usize {
        *self as usize
    }
}

impl SlotIndex for u64 {
    #[inline]
    fn as_index(&self) -> usize {
        *self as usize
    }
}

impl SlotIndex for usize {
    #[inline]
    fn as_index(&self) -> usize {
        *self
    }
}

impl<D> core::ops::Index<Slot> for Slab<D> {
    type Output = D;

    fn index(&self, slot: Slot) -> &Self::Output {
        unsafe { self.data[slot.as_index()].assume_init_ref() }
    }
}

impl<D> core::ops::IndexMut<Slot> for Slab<D> {
    fn index_mut(&mut self, slot: Slot) -> &mut Self::Output {
        unsafe { self.data[slot.as_index()].assume_init_mut() }
    }
}

/// An iterator over the elements of a slab.
///
/// This iterator yields elements from the slab in order from head to tail.
#[derive(Debug)]
pub struct SlabIterator<'a, D> {
    list: &'a Slab<D>,
    slot: Option<Slot>,
}

impl<'a, D> Iterator for SlabIterator<'a, D> {
    type Item = &'a D;

    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.slot.unwrap_or(self.list.head);
        if slot == NUL {
            return None;
        }
        let res = unsafe { self.list.data[slot.as_index()].assume_init_ref() };
        self.slot = Some(self.list.vec_next[slot.as_index()]);
        Some(res)
    }
}

impl<D> ExactSizeIterator for SlabIterator<'_, D> {
    fn len(&self) -> usize {
        self.list.len()
    }
}

impl<'a, D> DoubleEndedIterator for SlabIterator<'a, D> {
    fn next_back(&mut self) -> Option<&'a D> {
        let slot = self.slot.unwrap_or(self.list.tail);
        if slot == NUL {
            return None;
        }
        let res = unsafe { self.list.data[slot.as_index()].assume_init_ref() };
        self.slot = Some(self.list.vec_prev[slot.as_index()]);
        Some(res)
    }
}

impl<'a, D> IntoIterator for &'a Slab<D> {
    type IntoIter = SlabIterator<'a, D>;
    type Item = &'a D;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D: Clone> FromIterator<D> for Slab<D> {
    /// Creates a slab from an iterator.
    ///
    /// The slab will have a capacity equal to the number of elements in the iterator.
    /// Elements are inserted in reverse order, so that iterating the resulting slab
    /// will produce elements in the same order as the original iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let values = vec![1, 2, 3, 4, 5];
    /// let slab: Slab<_> = values.clone().into_iter().collect();
    ///
    /// // Verify size
    /// assert_eq!(slab.len(), 5);
    ///
    /// // Examine individual slots - elements are stored in reversed order
    /// // from the input sequence since push_front is used internally
    /// assert_eq!(*slab.get(0).unwrap(), 5);
    /// assert_eq!(*slab.get(1).unwrap(), 4);
    /// assert_eq!(*slab.get(2).unwrap(), 3);
    /// assert_eq!(*slab.get(3).unwrap(), 2);
    /// assert_eq!(*slab.get(4).unwrap(), 1);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the iterator contains too many elements for the slot type.
    fn from_iter<I: IntoIterator<Item = D>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (min, max_size) = iter.size_hint();

        // Use max_size if available, otherwise min
        let capacity = max_size.unwrap_or(min);

        // Try to create a slab with the estimated capacity
        let mut slab = Self::with_capacity(capacity).expect("Iterator too large for slab capacity");

        // Insert elements in reverse order (so iterating matches original order)
        let mut vec: Vec<D> = iter.collect();
        while let Some(item) = vec.pop() {
            if slab.push_front(item.clone()).is_err() {
                // If we get here, our size hint was wrong - try to recover by
                // creating a larger slab and moving elements
                let new_capacity = slab.capacity() * 2;
                let mut new_slab = Self::with_capacity(new_capacity)
                    .expect("Iterator too large for slab capacity");

                // Move elements from old slab to new one
                while let Some(old_item) = slab.pop_back() {
                    new_slab
                        .push_front(old_item)
                        .expect("New slab should have enough capacity");
                }

                // Add the current item
                new_slab
                    .push_front(item)
                    .expect("New slab should have enough capacity");

                // Replace with the new slab
                slab = new_slab;
            }
        }

        slab
    }
}

impl<D: Clone> Extend<D> for Slab<D> {
    /// Extends the slab with the elements from an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use slabigator::Slab;
    ///
    /// let mut slab = Slab::with_capacity(10).unwrap();
    /// slab.push_front(1).unwrap();
    /// slab.push_front(2).unwrap();
    ///
    /// slab.extend(vec![3, 4, 5]);
    /// assert_eq!(slab.len(), 5);
    ///
    /// // The slot assignment is not sequential but depends on the internal free list.
    /// // We can verify the order by iterating instead.
    /// let items: Vec<_> = slab.iter().copied().collect();
    /// assert_eq!(items, vec![5, 4, 3, 2, 1]);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the slab doesn't have enough capacity for all elements in the iterator.
    fn extend<I: IntoIterator<Item = D>>(&mut self, iter: I) {
        for item in iter {
            self.push_front(item).expect("Slab full during extend");
        }
    }
}

// Test implementations
#[test]
fn test() {
    let mut slab = Slab::with_capacity(3).unwrap();
    let a = slab.push_front(Box::pin(1)).unwrap();
    let b = slab.push_front(Box::pin(2)).unwrap();
    slab.push_front(Box::pin(3)).unwrap();
    assert_eq!(slab.len(), 3);
    assert!(slab.push_front(Box::pin(4)).is_err());
    slab.remove(a).unwrap();
    slab.remove(b).unwrap();
    assert_eq!(slab.len(), 1);
    let cv = slab.pop_back().unwrap();
    assert_eq!(3, *cv);
}

#[test]
fn test2() {
    use std::collections::VecDeque;

    use rand::prelude::*;

    let mut rng = rand::rng();
    let capacity = rng.random_range(1..=50);
    let mut slab = Slab::with_capacity(capacity).unwrap();

    let mut c: u64 = 0;
    let mut expected_len: usize = 0;
    let mut deque = VecDeque::with_capacity(capacity);
    for _ in 0..1_000_000 {
        let x = rng.random_range(0..=3);
        match x {
            0 => match slab.push_front(c) {
                Err(_) => {
                    assert!(slab.is_full());
                    assert_eq!(slab.free(), 0);
                }
                Ok(idx) => {
                    deque.push_front(idx);
                    expected_len += 1;
                    assert!(expected_len <= capacity);
                    assert_eq!(slab.free(), capacity - expected_len);
                }
            },
            1 => match slab.pop_back() {
                None => {
                    assert!(slab.is_empty());
                    assert_eq!(slab.free(), capacity);
                }
                Some(_x) => {
                    deque.pop_back().unwrap();
                    expected_len -= 1;
                    assert_eq!(slab.free(), capacity - expected_len);
                    if expected_len == 0 {
                        assert!(slab.is_empty());
                    }
                }
            },
            2 => {
                if slab.is_empty() {
                    continue;
                }
                let deque_len = deque.len();
                if deque_len == 0 {
                    continue;
                }
                let r = rng.random_range(0..deque_len);
                let idx = deque.remove(r).unwrap();
                slab.remove(idx).unwrap();
                expected_len -= 1;
                assert_eq!(slab.free(), capacity - expected_len);
            }
            3 => {
                // Only test slots that we know are valid in the deque
                if deque.is_empty() {
                    continue;
                }
                let r = rng.random_range(0..deque.len());
                let slot = deque[r];

                // Remove from the deque first
                let idx = deque.iter().position(|&x| x == slot).unwrap();
                deque.remove(idx);

                // Then remove from the slab
                slab.remove(slot).unwrap();
                expected_len -= 1;
                assert_eq!(slab.free(), capacity - expected_len);
            }
            _ => unreachable!(),
        }
        assert_eq!(slab.len(), expected_len);
        c += 1;
    }
}

#[test]
fn test_default() {
    let slab: Slab<i32> = Slab::default();
    assert_eq!(slab.capacity(), 16);
    assert_eq!(slab.len(), 0);
    assert!(slab.is_empty());
}

#[test]
fn test_from_iterator() {
    // Create a vector of test values
    let values = vec![1, 2, 3, 4, 5];

    // Create a slab from the vector
    let slab: Slab<i32> = values.clone().into_iter().collect();

    // Verify the length
    assert_eq!(slab.len(), 5);

    // Test the behavior more directly by comparing specific indices

    // The FromIterator implementation uses push_front, which should reverse the order
    // Manual verification of the data through slots
    if slab.len() == 5 {
        assert_eq!(*slab.get(0).unwrap(), 5); // Last element in input, first in slab
        assert_eq!(*slab.get(1).unwrap(), 4);
        assert_eq!(*slab.get(2).unwrap(), 3);
        assert_eq!(*slab.get(3).unwrap(), 2);
        assert_eq!(*slab.get(4).unwrap(), 1); // First element in input, last in slab
    }
}

#[test]
fn test_extend() {
    let mut slab = Slab::with_capacity(5).unwrap();
    slab.push_front(1).unwrap();
    slab.push_front(2).unwrap();

    slab.extend(vec![3, 4, 5]);
    assert_eq!(slab.len(), 5);

    // Collect the items to see what's actually there
    let items: Vec<_> = slab.iter().copied().collect();

    // Elements appear in the reverse order of how they were added
    // push_front(1), push_front(2), then extend with [3,4,5]
    // The extend implementation uses push_front for each element
    assert_eq!(items, vec![5, 4, 3, 2, 1]);
}

#[test]
fn test_clear() {
    let mut slab = Slab::with_capacity(3).unwrap();
    slab.push_front(1).unwrap();
    slab.push_front(2).unwrap();
    slab.push_front(3).unwrap();

    assert_eq!(slab.len(), 3);
    assert!(slab.is_full());

    slab.clear();

    assert_eq!(slab.len(), 0);
    assert!(slab.is_empty());
    assert!(!slab.is_full());

    // Should be able to add elements again
    let a = slab.push_front(4).unwrap();
    let b = slab.push_front(5).unwrap();
    let c = slab.push_front(6).unwrap();

    assert_eq!(slab.len(), 3);
    assert!(slab.is_full());

    // We can get elements by slot
    assert_eq!(*slab.get(a).unwrap(), 4);
    assert_eq!(*slab.get(b).unwrap(), 5);
    assert_eq!(*slab.get(c).unwrap(), 6);
}
