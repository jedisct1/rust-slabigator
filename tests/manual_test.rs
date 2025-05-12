use slabigator::Slab;

#[test]
fn test_manual_construction() {
    // Create a new empty slab
    let mut slab = Slab::with_capacity(5).unwrap();

    // Push elements manually
    slab.push_front(1).unwrap();
    slab.push_front(2).unwrap();
    slab.push_front(3).unwrap();
    slab.push_front(4).unwrap();
    slab.push_front(5).unwrap();

    // Verify size
    assert_eq!(slab.len(), 5);

    // When iterating, elements should come out in reverse order of insertion
    let output: Vec<_> = slab.iter().copied().collect();
    assert_eq!(output, vec![5, 4, 3, 2, 1]);
}
