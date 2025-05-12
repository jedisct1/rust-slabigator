use slabigator::Slab;

#[test]
fn test_from_iterator() {
    // Create a vector of values
    let input = vec![1, 2, 3, 4, 5];

    // Collect into a slab
    let slab: Slab<_> = input.into_iter().collect();

    // Verify the size
    assert_eq!(slab.len(), 5);

    // In the current implementation, elements maintain their original order
    let expected = vec![1, 2, 3, 4, 5];

    // Verify elements have the expected order
    let output: Vec<_> = slab.iter().copied().collect();
    assert_eq!(output, expected);
}

#[test]
fn test_extend_functionality() {
    let mut slab = Slab::with_capacity(5).unwrap();
    slab.push_front(1).unwrap();
    slab.push_front(2).unwrap();

    // Extend with more elements
    slab.extend(vec![3, 4, 5]);

    // Verify size
    assert_eq!(slab.len(), 5);

    // Verify order after extending - elements are in reverse order of insertion
    // since push_front is used internally by extend
    let items: Vec<_> = slab.iter().copied().collect();
    assert_eq!(items, vec![5, 4, 3, 2, 1]);
}

#[test]
fn test_clear_operation() {
    let mut slab = Slab::with_capacity(3).unwrap();

    // Add elements
    let _a = slab.push_front(4).unwrap();
    let _b = slab.push_front(5).unwrap();
    let _c = slab.push_front(6).unwrap();

    assert_eq!(slab.len(), 3);
    assert!(slab.is_full());

    // Clear the slab
    slab.clear();

    // Verify it's empty
    assert_eq!(slab.len(), 0);
    assert!(slab.is_empty());
    assert!(!slab.is_full());

    // Add new elements
    let x = slab.push_front(7).unwrap();
    let y = slab.push_front(8).unwrap();
    let z = slab.push_front(9).unwrap();

    // Verify slab state
    assert_eq!(slab.len(), 3);
    assert!(slab.is_full());

    // Verify access to new elements
    assert_eq!(*slab.get(x).unwrap(), 7);
    assert_eq!(*slab.get(y).unwrap(), 8);
    assert_eq!(*slab.get(z).unwrap(), 9);
}
