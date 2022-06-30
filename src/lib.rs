use std::{iter::Iterator, mem::MaybeUninit};

type Slot = u32;

const NUL: Slot = Slot::MAX;

/// A linked list that doesn't do dynamic allocations.
#[derive(Debug)]
pub struct Slab<D: Sized> {
    vec_next: Vec<Slot>,
    vec_prev: Vec<Slot>,
    free_head: Slot,
    head: Slot,
    tail: Slot,
    len: usize,
    data: Vec<MaybeUninit<D>>,
}

/// An error.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Error {
    /// Too large.
    TooLarge,
    /// List is full.
    Full,
    /// Slot is invalid.
    InvalidSlot,
    /// Slab is empty.
    Empty,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::TooLarge => write!(f, "Too large"),
            Error::Full => write!(f, "Full"),
            Error::InvalidSlot => write!(f, "Invalid slot"),
            Error::Empty => write!(f, "Empty"),
        }
    }
}

impl<D: Sized> Slab<D> {
    /// Create a new list with the given capacity.
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
        Ok(Self {
            vec_next,
            vec_prev,
            free_head: 0,
            head: NUL,
            tail: NUL,
            len: 0,
            data,
        })
    }

    /// Return the capacity of the list.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Return the length of the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return the number of elements that can still be stored.
    pub fn free(&self) -> usize {
        self.capacity() - self.len()
    }

    /// Return true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return true if the list is full.
    pub fn is_full(&self) -> bool {
        self.free_head == NUL
    }

    /// Prepend an element to the beginning of the list.
    pub fn push_front(&mut self, value: D) -> Result<Slot, Error> {
        let free_slot = self.free_head;
        if free_slot == NUL {
            return Err(Error::Full);
        }
        let prev = self.vec_prev[free_slot as usize];
        let next = self.vec_next[free_slot as usize];
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev as usize], free_slot);
            self.vec_next[prev as usize] = next;
        }
        if next != NUL {
            if !self.is_empty() {
                debug_assert_eq!(self.vec_prev[next as usize], free_slot);
            }
            self.vec_prev[next as usize] = prev;
        }
        if self.head != NUL {
            self.vec_prev[self.head as usize] = free_slot;
        }
        self.free_head = next;
        self.vec_next[free_slot as usize] = self.head;
        self.vec_prev[free_slot as usize] = NUL;
        if self.head == NUL {
            self.tail = free_slot;
        }
        self.head = free_slot;

        self.data[free_slot as usize] = MaybeUninit::new(value);
        self.len += 1;
        debug_assert!(self.len <= self.capacity());
        Ok(free_slot)
    }

    /// Remove an element from the list given its slot.
    pub fn remove(&mut self, slot: Slot) -> Result<(), Error> {
        if slot as usize >= self.capacity() {
            return Err(Error::InvalidSlot);
        }
        unsafe { self.data[slot as usize].assume_init_drop() };
        self.data[slot as usize] = MaybeUninit::uninit();
        let prev = self.vec_prev[slot as usize];
        let next = self.vec_next[slot as usize];
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev as usize], slot);
            self.vec_next[prev as usize] = next;
        }
        if next != NUL {
            if !self.is_empty() {
                debug_assert_eq!(self.vec_prev[next as usize], slot);
            }
            self.vec_prev[next as usize] = prev;
        }
        if self.tail == slot {
            self.tail = prev;
        }
        if self.head == slot {
            self.head = next;
        }
        self.vec_prev[slot as usize] = NUL;
        self.vec_next[slot as usize] = self.free_head;
        if self.free_head != NUL {
            self.vec_prev[self.free_head as usize] = slot;
        }
        self.free_head = slot;
        debug_assert!(self.len > 0);
        self.len -= 1;
        Ok(())
    }

    /// Remove and return the tail element of the list.
    pub fn pop_back(&mut self) -> Option<D> {
        let slot = self.tail;
        if slot == NUL {
            return None;
        }
        let value = unsafe { self.data[slot as usize].assume_init_read() };
        unsafe { self.data[slot as usize].assume_init_drop() };
        self.data[slot as usize] = MaybeUninit::uninit();
        let prev = self.vec_prev[slot as usize];
        debug_assert_eq!(self.vec_next[slot as usize], NUL);
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev as usize], slot);
            self.vec_next[prev as usize] = NUL;
        }
        self.tail = prev;
        if self.head == slot {
            self.head = NUL;
        }
        self.vec_prev[slot as usize] = NUL;
        self.vec_next[slot as usize] = self.free_head;
        if self.free_head != NUL {
            self.vec_prev[self.free_head as usize] = slot;
        }
        self.free_head = slot;
        debug_assert!(self.len > 0);
        self.len -= 1;
        Some(value)
    }

    /// Remove and return a reference to the tail element of the list.
    pub fn pop_back_ref(&mut self) -> Option<&D> {
        let slot = self.tail;
        if slot == NUL {
            return None;
        }
        let value = unsafe { self.data[slot as usize].assume_init_ref() };
        let prev = self.vec_prev[slot as usize];
        debug_assert_eq!(self.vec_next[slot as usize], NUL);
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev as usize], slot);
            self.vec_next[prev as usize] = NUL;
        }
        self.tail = prev;
        if self.head == slot {
            self.head = NUL;
        }
        self.vec_prev[slot as usize] = NUL;
        self.vec_next[slot as usize] = self.free_head;
        if self.free_head != NUL {
            self.vec_prev[self.free_head as usize] = slot;
        }
        self.free_head = slot;
        debug_assert!(self.len > 0);
        self.len -= 1;
        Some(value)
    }

    /// Remove and return a mutable reference to the tail element of the list.
    pub fn pop_back_ref_mut(&mut self) -> Option<&mut D> {
        let slot = self.tail;
        if slot == NUL {
            return None;
        }
        let value = unsafe { self.data[slot as usize].assume_init_mut() };
        let prev = self.vec_prev[slot as usize];
        debug_assert_eq!(self.vec_next[slot as usize], NUL);
        if prev != NUL {
            debug_assert_eq!(self.vec_next[prev as usize], slot);
            self.vec_next[prev as usize] = NUL;
        }
        self.tail = prev;
        if self.head == slot {
            self.head = NUL;
        }
        self.vec_prev[slot as usize] = NUL;
        self.vec_next[slot as usize] = self.free_head;
        if self.free_head != NUL {
            self.vec_prev[self.free_head as usize] = slot;
        }
        self.free_head = slot;
        debug_assert!(self.len > 0);
        self.len -= 1;
        Some(value)
    }

    /// Iterate over the list.
    pub fn iter(&self) -> SlabIterator<D> {
        SlabIterator {
            list: self,
            slot: None,
        }
    }
}

impl<D> Drop for Slab<D> {
    fn drop(&mut self) {
        let mut slot = self.head;
        while slot != NUL {
            let next = self.vec_next[slot as usize];
            unsafe { self.data[slot as usize].assume_init_drop() };
            slot = next;
        }
    }
}

impl<D> core::ops::Index<Slot> for Slab<D> {
    type Output = D;

    fn index(&self, slot: Slot) -> &Self::Output {
        unsafe { self.data[slot as usize].assume_init_ref() }
    }
}

impl<D> core::ops::IndexMut<Slot> for Slab<D> {
    fn index_mut(&mut self, slot: Slot) -> &mut Self::Output {
        unsafe { self.data[slot as usize].assume_init_mut() }
    }
}

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
        let res = unsafe { self.list.data[slot as usize].assume_init_ref() };
        self.slot = Some(self.list.vec_next[slot as usize]);
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
        let res = unsafe { self.list.data[slot as usize].assume_init_ref() };
        self.slot = Some(self.list.vec_prev[slot as usize]);
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

#[test]
fn test() {
    let mut slab = Slab::with_capacity(3).unwrap();
    let a = slab.push_front(1).unwrap();
    let b = slab.push_front(2).unwrap();
    slab.push_front(3).unwrap();
    assert_eq!(slab.len(), 3);
    assert!(slab.push_front(4).is_err());
    slab.remove(a).unwrap();
    slab.remove(b).unwrap();
    assert_eq!(slab.len(), 1);
    let cv = slab.pop_back().unwrap();
    assert_eq!(3, cv);
}
