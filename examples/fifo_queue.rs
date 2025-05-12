use slabigator::Slab;

/// A simple FIFO (First-In-First-Out) queue implementation
/// using Slabigator as the underlying storage mechanism.
struct FifoQueue<T> {
    slab: Slab<T>,
}

impl<T: Clone> FifoQueue<T> {
    /// Creates a new queue with the given capacity
    fn new(capacity: usize) -> Result<Self, slabigator::Error> {
        Ok(Self {
            slab: Slab::with_capacity(capacity)?,
        })
    }

    /// Adds an item to the queue
    fn enqueue(&mut self, item: T) -> Result<(), slabigator::Error> {
        self.slab.push_front(item)?;
        Ok(())
    }

    /// Removes and returns the oldest item from the queue
    fn dequeue(&mut self) -> Option<T> {
        self.slab.pop_back()
    }

    /// Returns the number of items in the queue
    fn len(&self) -> usize {
        self.slab.len()
    }

    /// Returns true if the queue is empty
    fn is_empty(&self) -> bool {
        self.slab.is_empty()
    }

    /// Returns true if the queue is full
    #[allow(dead_code)]
    fn is_full(&self) -> bool {
        self.slab.len() == self.slab.capacity()
    }

    /// Clears all items from the queue
    fn clear(&mut self) {
        self.slab.clear();
    }
}

fn main() {
    // Create a queue with capacity of 5
    let mut queue = FifoQueue::new(5).expect("Failed to create queue");

    // Enqueue some items
    for i in 1..=5 {
        println!("Enqueuing: {}", i);
        queue.enqueue(i).expect("Queue should have space");
    }

    // Try to enqueue when full
    if let Err(e) = queue.enqueue(6) {
        println!("As expected, can't enqueue to full queue: {}", e);
    }

    // Dequeue some items
    while let Some(item) = queue.dequeue() {
        println!("Dequeued: {}", item);
    }

    println!("Queue is now empty: {}", queue.is_empty());

    // Demonstrate reusing queue
    println!("Reusing queue...");
    for i in 10..=12 {
        println!("Enqueuing: {}", i);
        queue.enqueue(i).expect("Queue should have space");
    }

    println!("Queue length: {}", queue.len());

    // Clear the queue
    queue.clear();
    println!("After clear, queue is empty: {}", queue.is_empty());
}
