# ScoredSortedSet

`ScoredSortedSet` is a thread-safe, scored, and sorted set of items implemented in Rust. It allows you to associate items with integer scores, retrieve items based on their scores, update scores, and query the highest or lowest scores in the set.

## Features

- **Add items** with a specific score.
- **Remove items** based on score and value.
- **Update the score** of an existing item.
- Retrieve the **highest** or **lowest** score and associated items.
- Query the **top N scores** and their associated items.
- Thread-safe operations using `RwLock`.

## Installation

To use `ScoredSortedSet` in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
scored_set = "0.1.0"
```

## Usage

### Example 1: Adding and Retrieving Items

```rust
use scored_set::ScoredSortedSet;

fn main() {
    let set: ScoredSortedSet<String> = ScoredSortedSet::new();
    set.add(10, "Alice".to_string());
    set.add(20, "Bob".to_string());

    // Retrieve items with score 10
    let items = set.get(10).unwrap();
    println!("{:?}", items); // Output: ["Alice"]
}
```

### Example 2: Removing an Item

```rust
use scored_set::ScoredSortedSet;

fn main() {
    let set: ScoredSortedSet<String> = ScoredSortedSet::new();
    set.add(15, "Charlie".to_string());

    // Remove the item
    let removed = set.remove(15, &"Charlie".to_string());
    println!("{}", removed); // Output: true

    // Try to retrieve the removed item
    let items = set.get(15);
    assert!(items.is_none());
}
```

### Example 3: Updating the Score of an Item

```rust
use scored_set::ScoredSortedSet;

fn main() {
    let set = ScoredSortedSet::new();
    set.add(10, "Alice".to_string());

    // Update Alice's score from 10 to 20
    set.update_score(10, 20, &"Alice".to_string());

    assert!(set.get(10).is_none());
    let items = set.get(20).unwrap();
    println!("{:?}", items); // Output: ["Alice"]
}
```

### Example 4: Retrieving the Highest Score

```rust
use scored_set::ScoredSortedSet;

fn main() {
    let set = ScoredSortedSet::new();
    set.add(10, "Alice".to_string());
    set.add(30, "Charlie".to_string());
    set.add(20, "Bob".to_string());

    // Get the highest score
    let highest = set.highest_score().unwrap();
    println!("{:?}", highest); // Output: (30, ["Charlie"])
}
```

### Example 5: Retrieving all Scores

```rust
use scored_set::ScoredSortedSet;

fn main() {
    let set = ScoredSortedSet::new();
    set.add(10, "Alice".to_string());
    set.add(30, "Charlie".to_string());
    set.add(20, "Bob".to_string());

    // Get all scores in ascending order
    let scores = set.all_scores();
    println!("{:?}", scores); // Output: [10, 20, 30]
}
```

### Example 6: Multi-Threaded environment

```rust
use std::sync::{Arc, Barrier};
use std::thread;
use scored_set::ScoredSortedSet;

fn main() {
    // Wrap the ScoredSortedSet in an Arc to share it across threads
    let set = Arc::new(ScoredSortedSet::new());
    let barrier = Arc::new(Barrier::new(3)); // Barrier to synchronize thread completion

    // Cloning the Arc for each thread
    let set1 = Arc::clone(&set);
    let barrier1 = Arc::clone(&barrier);
    let handle1 = thread::spawn(move || {
        set1.add(10, "Alice".to_string());
        barrier1.wait(); // Wait for other threads
    });

    let set2 = Arc::clone(&set);
    let barrier2 = Arc::clone(&barrier);
    let handle2 = thread::spawn(move || {
        set2.add(20, "Bob".to_string());
        barrier2.wait(); // Wait for other threads
    });

    let set3 = Arc::clone(&set);
    let barrier3 = Arc::clone(&barrier);
    let handle3 = thread::spawn(move || {
        set3.update_score(10, 30, &"Alice".to_string());
        barrier3.wait(); // Wait for other threads
    });

    // Wait for all threads to finish
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();

    // Access the set after all threads have completed their operations
    let highest = set.highest_score().unwrap();
    println!("Highest score: {:?}", highest); // Output: (30, ["Alice"])
}
```