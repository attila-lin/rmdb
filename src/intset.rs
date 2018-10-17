use std::collections::TreeMap;
use std::collections::tree_map::Entries;  // This took forever
use std::num::One;
use std::num::Saturating;

// Equivalent to Set<T>, but only works for Ints, and may be more efficient
// if the set contains large contiguous ranges.  Each range is stored as a
// single entry instead of storing each member individually.
struct IntSet<T> {
  // These correspond to ranges of high->low.  The ranges are closed (inclusive)
  // on both ends: [low, high].  Half-open ranges would make the algorithms
  // simpler but then we wouldn't be able to specify a range that includes
  // the max value.
  //
  // This representation allows for efficient enumeration and set membership
  // test.
  ints: TreeMap<T, T>
}

impl<T: Int + std::fmt::Show> IntSet<T> {
  pub fn new() -> IntSet<T> {
    IntSet { ints: TreeMap::new() }
  }

  // Returns an iterator to the first range (low, high) such that high >= n.
  // If any range in the set contains n, it will be this one.
  fn find<'a>(&'a self, n: T) -> Entries<'a, T, T> { self.ints.lower_bound(&n) }

  // For testing only: validates that the set of ranges is the same as "ranges."
  fn assert_ranges(&self, ranges: &[(T, T)]) {
    let mut i = 0;
    for (&high, &low) in self.ints.iter() {
      assert_eq!(low, ranges[i].val0());
      assert_eq!(high, ranges[i].val1());
      i += 1;
    }
    assert_eq!(i, ranges.len());
  }

  // Test whether the set contains this value.
  pub fn contains(&self, n: T) -> bool {
    match self.find(n).next() {
      Some((_, low)) => { n >= *low }
      None => { false }
    }
  }

  // Add the single member "n".
  pub fn add(&mut self, n: T) {
    self.add_range(n, n);
  }

  // Add members from the inclusive range [low, high].
  pub fn add_range(&mut self, low: T, high: T) {
    // If we are merging the new range with existing entries; track the list of
    // existing entries we should remove later.  We can't do this in the loop
    // because there's no way to delete entries while iterating over the map.
    let mut to_delete = Vec::new();

    // Find the new range to insert by merging (low, high) with any overlapping
    // ranges.
    let (insert_high, insert_low) = {
      let mut iter = self.find(low.saturating_sub(One::one()));

      match iter.next() {
        None => {
          // No overlapping ranges, add this range verbatim.
          (high, low)
        }
        Some((&this_high, &this_low)) => {
          // We overlap with at least one existing range.
          // Compute the union of all intersecting ranges.
          let new_low = std::cmp::min(low, this_low);
          let mut new_high = std::cmp::max(high, this_high);
          to_delete.push(this_high);

          for (&iter_high, &iter_low) in iter {
            if iter_low > high.saturating_add(One::one()) { break; }
            to_delete.push(iter_high);
            new_high = std::cmp::max(new_high, iter_high)
          }

          (new_high, new_low)
        }
      }
    };

    for high in to_delete.iter() {
      let removed = self.ints.remove(high);
      assert!(removed.is_some())
    }

    let existing = self.ints.insert(insert_high, insert_low);
    assert!(existing.is_none());
  }

  pub fn add_intset(&mut self, set: &IntSet<T>) {
    for (&high, &low) in set.ints.iter() {
      self.add_range(low, high);
    }
  }
}