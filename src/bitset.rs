use std::fmt;

/// A BitSet implementation optimized for cocktail ingredients (up to 128 items)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitSet {
    bits: u128,
}

impl BitSet {
    /// Create a new empty BitSet
    #[inline]
    pub const fn new() -> Self {
        BitSet { bits: 0 }
    }

    /// Create a BitSet with a single element
    #[inline]
    pub const fn singleton(element: u32) -> Self {
        debug_assert!(element < 128);
        BitSet {
            bits: 1u128 << element,
        }
    }

    /// Create a BitSet from an iterator of elements
    #[inline]
    pub fn bitset_from_iter<I: IntoIterator<Item = i32>>(iter: I) -> Self {
        let mut bits = 0u128;
        for element in iter {
            debug_assert!((0..128).contains(&element));
            bits |= 1u128 << element;
        }
        BitSet { bits }
    }

    /// Insert an element
    #[inline]
    pub fn insert(&mut self, element: u32) {
        debug_assert!(element < 128);
        self.bits |= 1u128 << element;
    }

    /// Check if the set contains an element
    #[inline]
    pub const fn contains(&self, element: u32) -> bool {
        debug_assert!(element < 128);
        (self.bits & (1u128 << element)) != 0
    }

    /// Get the number of elements in the set
    #[inline]
    pub const fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }

    /// Check if the set is empty
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Union of two sets
    #[inline]
    pub const fn union(&self, other: &Self) -> Self {
        BitSet {
            bits: self.bits | other.bits,
        }
    }

    /// Intersection of two sets
    #[inline]
    pub const fn intersection(&self, other: &Self) -> Self {
        BitSet {
            bits: self.bits & other.bits,
        }
    }

    /// Difference of two sets (self - other)
    #[inline]
    pub const fn difference(&self, other: &Self) -> Self {
        BitSet {
            bits: self.bits & !other.bits,
        }
    }

    /// Check if this set is a subset of another
    #[inline]
    pub const fn is_subset(&self, other: &Self) -> bool {
        (self.bits & !other.bits) == 0
    }

    /// Check if this set is a superset of another
    #[inline]
    pub const fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    /// Iterator over the elements in the set
    pub fn iter(&self) -> BitSetIter {
        BitSetIter { bits: self.bits }
    }

    /// Get the raw bits representation
    #[inline]
    pub const fn bits(&self) -> u128 {
        self.bits
    }
}

impl Default for BitSet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

/// Iterator over elements in a BitSet
pub struct BitSetIter {
    bits: u128,
}

impl Iterator for BitSetIter {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            None
        } else {
            let index = self.bits.trailing_zeros();
            self.bits &= !(1u128 << index);
            Some(index)
        }
    }
}

impl std::ops::BitOr for BitSet {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(&rhs)
    }
}

impl std::ops::BitOr<&BitSet> for BitSet {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: &BitSet) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitOr<BitSet> for &BitSet {
    type Output = BitSet;

    #[inline]
    fn bitor(self, rhs: BitSet) -> Self::Output {
        self.union(&rhs)
    }
}

impl std::ops::BitOr<&BitSet> for &BitSet {
    type Output = BitSet;

    #[inline]
    fn bitor(self, rhs: &BitSet) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitAnd for BitSet {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(&rhs)
    }
}

impl std::ops::BitAnd<&BitSet> for BitSet {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: &BitSet) -> Self::Output {
        self.intersection(rhs)
    }
}

impl std::ops::BitAnd<BitSet> for &BitSet {
    type Output = BitSet;

    #[inline]
    fn bitand(self, rhs: BitSet) -> Self::Output {
        self.intersection(&rhs)
    }
}

impl std::ops::BitAnd<&BitSet> for &BitSet {
    type Output = BitSet;

    #[inline]
    fn bitand(self, rhs: &BitSet) -> Self::Output {
        self.intersection(rhs)
    }
}

impl std::ops::Sub for BitSet {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(&rhs)
    }
}

impl std::ops::Sub<&BitSet> for BitSet {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &BitSet) -> Self::Output {
        self.difference(rhs)
    }
}

impl std::ops::Sub<BitSet> for &BitSet {
    type Output = BitSet;

    #[inline]
    fn sub(self, rhs: BitSet) -> Self::Output {
        self.difference(&rhs)
    }
}

impl std::ops::Sub<&BitSet> for &BitSet {
    type Output = BitSet;

    #[inline]
    fn sub(self, rhs: &BitSet) -> Self::Output {
        self.difference(rhs)
    }
}

impl std::cmp::Ord for BitSet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bits.cmp(&other.bits)
    }
}

impl std::cmp::PartialOrd for BitSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut set1 = BitSet::new();
        set1.insert(1);
        set1.insert(3);
        set1.insert(5);

        let mut set2 = BitSet::new();
        set2.insert(3);
        set2.insert(5);
        set2.insert(7);

        assert_eq!(set1.len(), 3);
        assert!(set1.contains(1));
        assert!(!set1.contains(7));

        let union = set1.union(&set2);
        assert_eq!(union.len(), 4);
        assert!(union.contains(1));
        assert!(union.contains(7));

        let intersection = set1.intersection(&set2);
        assert_eq!(intersection.len(), 2);
        assert!(intersection.contains(3));
        assert!(intersection.contains(5));

        let difference = set1.difference(&set2);
        assert_eq!(difference.len(), 1);
        assert!(difference.contains(1));
        assert!(!difference.contains(3));
    }

    #[test]
    fn test_subset() {
        let mut set1 = BitSet::new();
        set1.insert(1);
        set1.insert(3);

        let mut set2 = BitSet::new();
        set2.insert(1);
        set2.insert(3);
        set2.insert(5);

        assert!(set1.is_subset(&set2));
        assert!(!set2.is_subset(&set1));
        assert!(set2.is_superset(&set1));
    }
}
