use slabigator::Slab;

#[test]
fn test_correct_ordering() {
    // Create a vector for testing
    let values = [1, 2, 3, 4, 5];

    // Create a slab
    let mut slab = Slab::with_capacity(5).unwrap();

    // Insert elements using push_front
    for value in values.iter().copied() {
        slab.push_front(value).unwrap();
    }

    // Because we're using push_front, the order should be reversed
    // when we iterate through the slab
    let output: Vec<i32> = slab.iter().copied().collect();

    // Items should be in reverse order of insertion
    let expected = vec![5, 4, 3, 2, 1];
    assert_eq!(output, expected);
}
