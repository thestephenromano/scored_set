use std::collections::BTreeMap;
use std::sync::RwLock;

/// A thread-safe, scored, and sorted set of items.
/// The set uses a BTreeMap to store items with their associated scores.
/// Items with the same score are stored in a vector.
pub(crate) struct ScoredSortedSet<T> {
    inner: RwLock<BTreeMap<i32, Vec<T>>>, // Wrap BTreeMap in an RwLock
}

impl<T> ScoredSortedSet<T> {
    /// Creates a new, empty `ScoredSortedSet`.
    pub(crate) fn new() -> Self {
        ScoredSortedSet {
            inner: RwLock::new(BTreeMap::new()),
        }
    }

    /// Adds an item with a given score to the set.
    /// If the score already exists, the item is appended to the vector of items for that score.
    pub(crate) fn add(&self, score: i32, item: T) {
        let mut inner = self.inner.write().unwrap(); // Lock the RwLock for writing
        inner.entry(score).or_insert_with(Vec::new).push(item);
    }

    /// Removes a specified item from the set for a given score.
    /// Returns `true` if the item was successfully removed, `false` otherwise.
    /// If the vector of items for that score becomes empty, the score is removed from the set.
    pub(crate) fn remove(&self, score: i32, item: &T) -> bool
    where
        T: PartialEq + Clone, // Clone trait bound added for item removal
    {
        let mut item_removed = false;
        let mut inner = self.inner.write().unwrap(); // Acquiring a write lock

        if let Some(items) = inner.get_mut(&score) {
            let initial_len = items.len();
            items.retain(|current_item| {
                if current_item == item {
                    item_removed = true; // Item found and will be removed
                    false // Do not retain this item
                } else {
                    true // Retain other items
                }
            });
            if items.is_empty() {
                inner.remove(&score);
            } else if initial_len == items.len() {
                // If the lengths are equal, no item was removed
                item_removed = false;
            }
        }

        item_removed
    }

    /// Updates the score of a specified item.
    /// The item is first removed from the old score and then added to the new score.
    /// If the item does not exist at the old score, no change is made.
    pub(crate) fn update_score(&self, old_score: i32, new_score: i32, item: &T)
    where
        T: PartialEq + Clone,
    {
        let mut inner = self.inner.write().unwrap();

        if let Some(items) = inner.get_mut(&old_score) {
            if let Some(pos) = items.iter().position(|x| x == item) {
                let item = items.remove(pos);
                if items.is_empty() {
                    inner.remove(&old_score);
                }
                inner.entry(new_score).or_insert_with(Vec::new).push(item);
            }
        }
    }

    /// Retrieves a clone of the items associated with a given score.
    /// Returns `None` if the score does not exist in the set.
    pub(crate) fn get(&self, score: i32) -> Option<Vec<T>>
    where
        T: Clone, // Ensure T can be cloned
    {
        let inner = self.inner.read().unwrap(); // Lock the RwLock for reading
        inner.get(&score).cloned() // Clone the result to avoid borrowing issues
    }

    /// Returns a vector containing the top `n` highest scores and their associated items.
    /// The vector is sorted in descending order of scores.
    fn highest_scores(&self, n: usize) -> Vec<(i32, Vec<T>)>
    where
        T: Clone, // Ensure T can be cloned
    {
        let inner = self.inner.read().unwrap();
        inner
            .iter()
            .rev() // Reverse iterator to start from the highest score
            .take(n) // Take the n highest scores
            .map(|(&score, items)| (score, items.clone())) // Clone items to avoid borrowing issues
            .collect()
    }

    /// Retrieves the highest score and its associated items.
    /// Returns `None` if the set is empty.
    fn highest_score(&self) -> Option<(i32, Vec<T>)>
    where
        T: Clone, // Ensure T can be cloned
    {
        let inner = self.inner.read().unwrap();
        inner
            .iter()
            .rev()
            .next()
            .map(|(&score, items)| (score, items.clone()))
    }

    /// Retrieves the lowest score and its associated items.
    /// Returns `None` if the set is empty.
    fn lowest_score(&self) -> Option<(i32, Vec<T>)>
    where
        T: Clone, // Ensure T can be cloned
    {
        let inner = self.inner.read().unwrap();
        inner
            .iter()
            .next()
            .map(|(&score, items)| (score, items.clone()))
    }

    /// Returns a vector containing all the scores in the set in ascending order.
    fn all_scores(&self) -> Vec<i32> {
        let inner = self.inner.read().unwrap();
        inner.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ScoredSortedSet;

    #[test]
    fn test_add_and_get() {
        let set: ScoredSortedSet<String> = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());

        let items = set.get(10).unwrap();
        assert_eq!(items, vec!["Alice".to_string()]);
    }

    #[test]
    fn test_remove() {
        let set: ScoredSortedSet<String> = ScoredSortedSet::new();
        set.add(15, "Bob".to_string());
        set.add(15, "Charlie".to_string());

        set.remove(15, &"Bob".to_string());

        let items = set.get(15).unwrap();
        assert_eq!(items, vec!["Charlie".to_string()]);
    }

    #[test]
    fn test_remove_nonexistent() {
        let set: ScoredSortedSet<String> = ScoredSortedSet::new();
        set.add(20, "Dave".to_string());

        // Attempt to remove an item that doesn't exist
        set.remove(20, &"Eve".to_string());

        let items = set.get(20).unwrap();
        assert_eq!(items, vec!["Dave".to_string()]);
    }

    #[test]
    fn test_get_nonexistent() {
        let set: ScoredSortedSet<i32> = ScoredSortedSet::new();

        // Attempt to get items for a score that has no items
        let items = set.get(25);
        assert!(items.is_none());
    }

    #[test]
    fn test_multiple_scores() {
        let set: ScoredSortedSet<String> = ScoredSortedSet::new();
        set.add(30, "Fred".to_string());
        set.add(40, "George".to_string());

        let items_30 = set.get(30).unwrap();
        assert_eq!(items_30, vec!["Fred".to_string()]);

        let items_40 = set.get(40).unwrap();
        assert_eq!(items_40, vec!["George".to_string()]);
    }

    #[test]
    fn test_multiple_items_same_score() {
        let set: ScoredSortedSet<String> = ScoredSortedSet::new();
        set.add(50, "Hannah".to_string());
        set.add(50, "Ian".to_string());

        let items = set.get(50).unwrap();
        assert_eq!(items, vec!["Hannah".to_string(), "Ian".to_string()]);
    }

    #[test]
    fn update_score_existing_item() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());

        set.update_score(10, 20, &"Alice".to_string());

        assert!(
            set.get(10).is_none(),
            "Item should be removed from the old score"
        );
        assert_eq!(
            set.get(20).unwrap(),
            vec!["Alice".to_string()],
            "Item should exist with the new score"
        );
    }

    #[test]
    fn update_score_nonexistent_item() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());

        // Attempt to update score of an item that doesn't exist at the specified old score
        set.update_score(15, 25, &"Bob".to_string());

        assert!(set.get(15).is_none(), "Old score should not have any items");
        assert!(
            set.get(25).is_none(),
            "New score should not have the item because it didn't exist at the old score"
        );
        assert_eq!(
            set.get(10).unwrap(),
            vec!["Alice".to_string()],
            "Other items should remain unaffected"
        );
    }

    #[test]
    fn update_score_to_existing_score() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());
        set.add(20, "Bob".to_string());

        // Move Alice to the same score as Bob
        set.update_score(10, 20, &"Alice".to_string());

        assert!(
            set.get(10).is_none(),
            "Alice should be removed from the old score"
        );
        let items_with_new_score = set.get(20).unwrap();
        assert!(
            items_with_new_score.contains(&"Alice".to_string()),
            "Alice should be added to the new score"
        );
        assert!(
            items_with_new_score.contains(&"Bob".to_string()),
            "Bob should remain at the new score"
        );
    }

    #[test]
    fn maintain_order_after_update_score() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());
        set.add(10, "Bob".to_string());

        // Update score of Alice, moving her to a new score
        set.update_score(10, 20, &"Alice".to_string());

        let items_with_old_score = set.get(10).unwrap();
        assert_eq!(
            items_with_old_score,
            vec!["Bob".to_string()],
            "Only Bob should remain at the old score"
        );

        let items_with_new_score = set.get(20).unwrap();
        assert_eq!(
            items_with_new_score,
            vec!["Alice".to_string()],
            "Alice should be present with the new score"
        );
    }

    #[test]
    fn highest_scores_more_than_exists() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());
        set.add(20, "Bob".to_string());

        // Request more scores than exist in the set
        let scores = set.highest_scores(5);
        assert_eq!(scores.len(), 2, "Should return only the available scores");
        assert_eq!(scores[0].0, 20, "The highest score should be first");
        assert_eq!(scores[1].0, 10, "The next highest score should be second");
    }

    #[test]
    fn highest_scores_exact_number() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());
        set.add(20, "Bob".to_string());
        set.add(30, "Charlie".to_string());

        // Request exactly the number of scores that exist
        let scores = set.highest_scores(3);
        assert_eq!(scores.len(), 3, "Should return all available scores");
        assert_eq!(scores[0].0, 30, "The highest score should be first");
        assert_eq!(scores[1].0, 20, "The next highest score should be second");
        assert_eq!(scores[2].0, 10, "The lowest score should be last");
    }

    #[test]
    fn highest_scores_ordered_correctly() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());
        set.add(30, "Charlie".to_string());
        set.add(20, "Bob".to_string());

        // Request scores to verify they are ordered correctly
        let scores = set.highest_scores(2);
        assert_eq!(scores.len(), 2, "Should return the top 2 scores");
        assert_eq!(scores[0].0, 30, "The highest score should be first");
        assert_eq!(scores[1].0, 20, "The second highest score should be second");
    }

    #[test]
    fn highest_scores_none_available() {
        let set: ScoredSortedSet<String> = ScoredSortedSet::new();

        // Request scores when none are available
        let scores = set.highest_scores(2);
        assert!(
            scores.is_empty(),
            "Should return an empty vector when no scores are available"
        );
    }

    #[test]
    fn lowest_and_highest_score_empty_set() {
        let set: ScoredSortedSet<String> = ScoredSortedSet::new();

        assert!(
            set.lowest_score().is_none(),
            "Should be None for an empty set"
        );
        assert!(
            set.highest_score().is_none(),
            "Should be None for an empty set"
        );
    }

    #[test]
    fn lowest_and_highest_score_single_item() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());

        let lowest = set.lowest_score().unwrap();
        assert_eq!(lowest.0, 10, "Lowest score should be 10");
        assert_eq!(
            lowest.1,
            vec!["Alice".to_string()],
            "Lowest score item should be Alice"
        );

        let highest = set.highest_score().unwrap();
        assert_eq!(highest.0, 10, "Highest score should also be 10");
        assert_eq!(
            highest.1,
            vec!["Alice".to_string()],
            "Highest score item should also be Alice"
        );
    }

    #[test]
    fn lowest_and_highest_score_multiple_items() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());
        set.add(20, "Bob".to_string());
        set.add(30, "Charlie".to_string());

        let lowest = set.lowest_score().unwrap();
        assert_eq!(lowest.0, 10, "Lowest score should be 10");
        assert_eq!(
            lowest.1,
            vec!["Alice".to_string()],
            "Lowest score item should be Alice"
        );

        let highest = set.highest_score().unwrap();
        assert_eq!(highest.0, 30, "Highest score should be 30");
        assert_eq!(
            highest.1,
            vec!["Charlie".to_string()],
            "Highest score item should be Charlie"
        );
    }

    #[test]
    fn updating_scores_affects_lowest_and_highest_correctly() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());
        set.add(20, "Bob".to_string());

        set.update_score(10, 5, &"Alice".to_string()); // Update Alice to a lower score

        let lowest = set.lowest_score().unwrap();
        assert_eq!(lowest.0, 5, "After update, lowest score should be 5");
        assert_eq!(
            lowest.1,
            vec!["Alice".to_string()],
            "Lowest score item should now be Alice"
        );

        let highest = set.highest_score().unwrap();
        assert_eq!(highest.0, 20, "Highest score remains 20");
        assert_eq!(
            highest.1,
            vec!["Bob".to_string()],
            "Highest score item remains Bob"
        );
    }

    #[test]
    fn all_scores_empty_set() {
        let set = ScoredSortedSet::<String>::new();
        let scores = set.all_scores();
        assert!(scores.is_empty(), "Expected no scores for an empty set");
    }

    #[test]
    fn all_scores_non_empty_set() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());
        set.add(20, "Bob".to_string());
        set.add(30, "Charlie".to_string());

        let scores = set.all_scores();
        assert_eq!(scores.len(), 3, "Expected three scores in the set");
        assert_eq!(
            scores,
            vec![10, 20, 30],
            "Scores should be in ascending order"
        );
    }

    // This tests the unique nature of scores implicitly
    #[test]
    fn all_scores_with_duplicate_scores() {
        let set = ScoredSortedSet::new();
        set.add(10, "Alice".to_string());
        set.add(10, "Duplicate Alice".to_string()); // Duplicate score
        set.add(20, "Bob".to_string());

        let scores = set.all_scores();
        assert_eq!(scores.len(), 2, "Expected scores to be unique");
        assert_eq!(
            scores,
            vec![10, 20],
            "Scores should be in ascending order and unique"
        );
    }
}
