use slabigator::Slab;

// Import the internal slot type - we'll use u32 which is the default
type Slot = u32;
use std::collections::VecDeque;

/// A simple object pool implementation using Slabigator.
/// This demonstrates how Slabigator can be used for efficient
/// object reuse without dynamic allocations.
struct ObjectPool<T> {
    slab: Slab<T>,
    free_slots: VecDeque<Slot>,
}

impl<T: Clone + Default> ObjectPool<T> {
    /// Creates a new object pool with the specified capacity.
    fn new(capacity: usize) -> Result<Self, slabigator::Error> {
        Ok(Self {
            slab: Slab::with_capacity(capacity)?,
            free_slots: VecDeque::with_capacity(capacity),
        })
    }

    /// Acquires an object from the pool. If no objects are available,
    /// creates a new one (if capacity allows).
    fn acquire(&mut self) -> Result<(Slot, &mut T), slabigator::Error> {
        if let Some(slot) = self.free_slots.pop_front() {
            // Reuse an existing slot
            Ok((slot, self.slab.get_mut(slot).unwrap()))
        } else {
            // Create a new object
            let slot = self.slab.push_front(T::default())?;
            Ok((slot, self.slab.get_mut(slot).unwrap()))
        }
    }

    /// Returns an object to the pool for future reuse.
    fn release(&mut self, slot: Slot) {
        if self.slab.get(slot).is_ok() {
            self.free_slots.push_back(slot);
        }
    }

    /// Returns the number of objects currently in use.
    fn in_use(&self) -> usize {
        self.slab.len() - self.free_slots.len()
    }

    /// Returns the total capacity of the pool.
    fn capacity(&self) -> usize {
        self.slab.capacity()
    }

    /// Resets the pool, returning all objects to the free list.
    fn reset(&mut self) {
        // Clear the free slots list
        self.free_slots.clear();

        // For this example, simply clear the slab
        self.slab.clear();
    }
}

// Demo using a simple bullet structure for a game
#[derive(Clone, Default)]
struct Bullet {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    active: bool,
}

impl Bullet {
    fn initialize(&mut self, x: f32, y: f32, velocity_x: f32, velocity_y: f32) {
        self.x = x;
        self.y = y;
        self.velocity_x = velocity_x;
        self.velocity_y = velocity_y;
        self.active = true;
    }

    fn update(&mut self) {
        self.x += self.velocity_x;
        self.y += self.velocity_y;
    }
}

fn main() {
    // Create a bullet pool with capacity for 100 bullets
    let mut bullet_pool = ObjectPool::<Bullet>::new(100).expect("Failed to create pool");

    println!(
        "Created bullet pool with capacity: {}",
        bullet_pool.capacity()
    );

    // Simulate firing 25 bullets
    let mut active_bullets = Vec::new();
    for i in 0..25 {
        let (slot, bullet) = bullet_pool.acquire().expect("Pool should have capacity");

        // Initialize the bullet with some example values
        let angle = (i as f32) * 0.25;
        bullet.initialize(0.0, 0.0, angle.cos() * 5.0, angle.sin() * 5.0);

        active_bullets.push(slot);
    }

    println!("Fired 25 bullets. Bullets in use: {}", bullet_pool.in_use());

    // Simulate updating bullets for 5 frames
    for frame in 1..=5 {
        println!("Frame {}", frame);

        // Update all active bullets
        for &slot in &active_bullets {
            if let Ok(bullet) = bullet_pool.slab.get_mut(slot) {
                bullet.update();
                println!("  Bullet at position: ({:.1}, {:.1})", bullet.x, bullet.y);
            }
        }

        // Every other frame, return some bullets to the pool
        if frame % 2 == 0 && !active_bullets.is_empty() {
            let returned = active_bullets.len() / 2;
            println!("  Returning {} bullets to pool", returned);

            for _ in 0..returned {
                if let Some(slot) = active_bullets.pop() {
                    bullet_pool.release(slot);
                }
            }

            println!("  Bullets in use: {}", bullet_pool.in_use());
        }
    }

    // Reset the pool
    bullet_pool.reset();
    println!("Pool reset. Bullets in use: {}", bullet_pool.in_use());
}
