// Quick test to verify select_nth_unstable_by semantics
// Question: If we want the K smallest elements, do we call select_nth_unstable_by(k, ...) or select_nth_unstable_by(k-1, ...)?

fn main() {
    // Test case: We want the 3 smallest elements from [9, 2, 7, 1, 5, 3, 8, 4, 6]
    // Expected smallest 3: [1, 2, 3] (in any order)

    let mut data = vec![9, 2, 7, 1, 5, 3, 8, 4, 6];
    let k = 3;

    println!("Original: {:?}", data);
    println!("We want the {} smallest elements", k);

    // Test with k
    let mut test1 = data.clone();
    test1.select_nth_unstable(k);
    println!("\nAfter select_nth_unstable({}):", k);
    println!("  Full array: {:?}", test1);
    println!("  Element at index {}: {}", k, test1[k]);
    println!("  Elements [0..{}]: {:?}", k, &test1[0..k]);

    // Test with k-1
    let mut test2 = data.clone();
    test2.select_nth_unstable(k - 1);
    println!("\nAfter select_nth_unstable({}):", k - 1);
    println!("  Full array: {:?}", test2);
    println!("  Element at index {}: {}", k - 1, test2[k - 1]);
    println!("  Elements [0..{}]: {:?}", k, &test2[0..k]);

    // Now test the pattern from the code: select_nth(k) then truncate(k)
    let mut test3 = data.clone();
    if test3.len() > k {
        test3.select_nth_unstable(k);
        test3.truncate(k);
    }
    println!("\nAfter select_nth_unstable({}) + truncate({}):", k, k);
    println!("  Remaining: {:?}", test3);
    println!("  Length: {}", test3.len());

    // Compare with the CORRECT approach: select_nth(k-1) then truncate(k)
    let mut test4 = data.clone();
    if test4.len() > k {
        test4.select_nth_unstable(k - 1);
        test4.truncate(k);
    }
    println!("\nAfter select_nth_unstable({}) + truncate({}):", k - 1, k);
    println!("  Remaining: {:?}", test4);
    println!("  Length: {}", test4.len());

    // Verify correctness
    test4.sort();
    println!("\n  Sorted: {:?}", test4);
    println!("  Expected smallest 3: [1, 2, 3]");
}
