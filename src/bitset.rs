use smallvec::SmallVec;
use std::fmt;

/// A BitSet implementation optimized for cocktail ingredients with no size limit
/// Uses SmallVec to avoid heap allocations for typical cocktail sizes (2-6 ingredients)
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BitSet {
    chunks: SmallVec<[u64; 3]>,
}

const BITS_PER_CHUNK: usize = 64;

impl BitSet {
    /// Create a new empty BitSet
    #[inline]
    pub fn new() -> Self {
        BitSet {
            chunks: SmallVec::new(),
        }
    }

    /// Create a BitSet with a single element
    #[inline]
    pub fn singleton(element: u32) -> Self {
        let mut bitset = BitSet::new();
        bitset.insert(element);
        bitset
    }

    /// Create a BitSet from an iterator of elements
    #[inline]
    pub fn bitset_from_iter<I: IntoIterator<Item = i32>>(iter: I) -> Self {
        let mut bitset = BitSet::new();
        for element in iter {
            debug_assert!(element >= 0);
            bitset.insert(element as u32);
        }
        bitset
    }

    /// Ensure the BitSet has enough chunks to store the given element
    #[inline]
    fn ensure_capacity(&mut self, element: u32) {
        let chunk_index = (element as usize) / BITS_PER_CHUNK;
        if chunk_index >= self.chunks.len() {
            self.chunks.resize(chunk_index + 1, 0);
        }
    }

    /// Insert an element
    #[inline]
    pub fn insert(&mut self, element: u32) {
        self.ensure_capacity(element);
        let chunk_index = (element as usize) / BITS_PER_CHUNK;
        let bit_index = (element as usize) % BITS_PER_CHUNK;
        self.chunks[chunk_index] |= 1u64 << bit_index;
    }

    /// Check if the set contains an element
    #[inline]
    pub fn contains(&self, element: u32) -> bool {
        let chunk_index = (element as usize) / BITS_PER_CHUNK;
        if chunk_index >= self.chunks.len() {
            return false;
        }
        let bit_index = (element as usize) % BITS_PER_CHUNK;
        (self.chunks[chunk_index] & (1u64 << bit_index)) != 0
    }

    /// Get the number of elements in the set
    #[inline]
    pub fn len(&self) -> usize {
        self.chunks
            .iter()
            .map(|chunk| chunk.count_ones() as usize)
            .sum()
    }

    /// Check if the set is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chunks.iter().all(|&chunk| chunk == 0)
    }

    /// Union of two sets
    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        let max_len = self.chunks.len().max(other.chunks.len());
        let mut result_chunks = SmallVec::with_capacity(max_len);

        for i in 0..max_len {
            let a = self.chunks.get(i).copied().unwrap_or(0);
            let b = other.chunks.get(i).copied().unwrap_or(0);
            result_chunks.push(a | b);
        }

        // Remove trailing zero chunks
        while result_chunks.last() == Some(&0) {
            result_chunks.pop();
        }

        BitSet {
            chunks: result_chunks,
        }
    }

    /// Intersection of two sets
    #[inline]
    pub fn intersection(&self, other: &Self) -> Self {
        let min_len = self.chunks.len().min(other.chunks.len());
        let mut result_chunks = SmallVec::with_capacity(min_len);

        for i in 0..min_len {
            result_chunks.push(self.chunks[i] & other.chunks[i]);
        }

        // Remove trailing zero chunks
        while result_chunks.last() == Some(&0) {
            result_chunks.pop();
        }

        BitSet {
            chunks: result_chunks,
        }
    }

    /// Difference of two sets (self - other)
    #[inline]
    pub fn difference(&self, other: &Self) -> Self {
        let mut result_chunks = SmallVec::with_capacity(self.chunks.len());

        for i in 0..self.chunks.len() {
            let other_chunk = other.chunks.get(i).copied().unwrap_or(0);
            result_chunks.push(self.chunks[i] & !other_chunk);
        }

        // Remove trailing zero chunks
        while result_chunks.last() == Some(&0) {
            result_chunks.pop();
        }

        BitSet {
            chunks: result_chunks,
        }
    }

    /// Check if this set is a subset of another
    #[inline]
    pub fn is_subset(&self, other: &Self) -> bool {
        // If self has more chunks than other, check if extra chunks are all zero
        if self.chunks.len() > other.chunks.len() {
            for i in other.chunks.len()..self.chunks.len() {
                if self.chunks[i] != 0 {
                    return false;
                }
            }
        }

        // Check all overlapping chunks
        let min_len = self.chunks.len().min(other.chunks.len());
        for i in 0..min_len {
            if (self.chunks[i] & !other.chunks[i]) != 0 {
                return false;
            }
        }

        true
    }

    /// Check if this set is a superset of another
    #[inline]
    pub fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    /// Union in place (self |= other)
    #[inline]
    pub fn union_assign(&mut self, other: &Self) {
        if other.chunks.len() > self.chunks.len() {
            self.chunks.resize(other.chunks.len(), 0);
        }

        for i in 0..other.chunks.len() {
            self.chunks[i] |= other.chunks[i];
        }

        // Remove trailing zero chunks
        while self.chunks.last() == Some(&0) && !self.chunks.is_empty() {
            self.chunks.pop();
        }
    }

    /// Intersection in place (self &= other)
    #[inline]
    pub fn intersection_assign(&mut self, other: &Self) {
        let min_len = self.chunks.len().min(other.chunks.len());

        // Intersect overlapping chunks
        for i in 0..min_len {
            self.chunks[i] &= other.chunks[i];
        }

        // Clear any chunks beyond other's length
        self.chunks.truncate(min_len);

        // Remove trailing zero chunks
        while self.chunks.last() == Some(&0) && !self.chunks.is_empty() {
            self.chunks.pop();
        }
    }

    /// Difference in place (self -= other)
    #[inline]
    pub fn difference_assign(&mut self, other: &Self) {
        let min_len = self.chunks.len().min(other.chunks.len());

        for i in 0..min_len {
            self.chunks[i] &= !other.chunks[i];
        }

        // Remove trailing zero chunks
        while self.chunks.last() == Some(&0) && !self.chunks.is_empty() {
            self.chunks.pop();
        }
    }

    /// Iterator over the elements in the set
    pub fn iter(&self) -> BitSetIter {
        BitSetIter {
            chunks: &self.chunks,
            chunk_index: 0,
            current_chunk: self.chunks.first().copied().unwrap_or(0),
        }
    }

    /// Get the raw bits representation (for compatibility, returns first chunk or 0)
    #[inline]
    pub fn bits(&self) -> u128 {
        if self.chunks.is_empty() {
            0
        } else if self.chunks.len() == 1 {
            self.chunks[0] as u128
        } else {
            // Combine first two chunks for backward compatibility
            let low = self.chunks[0] as u128;
            let high = self.chunks.get(1).copied().unwrap_or(0) as u128;
            low | (high << 64)
        }
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
pub struct BitSetIter<'a> {
    chunks: &'a [u64],
    chunk_index: usize,
    current_chunk: u64,
}

impl<'a> Iterator for BitSetIter<'a> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_chunk != 0 {
                let bit_index = self.current_chunk.trailing_zeros();
                self.current_chunk &= !(1u64 << bit_index);
                return Some((self.chunk_index * BITS_PER_CHUNK) as u32 + bit_index);
            }

            self.chunk_index += 1;
            if self.chunk_index >= self.chunks.len() {
                return None;
            }
            self.current_chunk = self.chunks[self.chunk_index];
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
        // Compare chunk by chunk, treating missing chunks as 0
        let max_len = self.chunks.len().max(other.chunks.len());
        for i in 0..max_len {
            let a = self.chunks.get(i).copied().unwrap_or(0);
            let b = other.chunks.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
        std::cmp::Ordering::Equal
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

    #[test]
    fn test_large_values() {
        let mut set = BitSet::new();

        // Test values beyond 128
        set.insert(150);
        set.insert(200);
        set.insert(1000);

        assert!(set.contains(150));
        assert!(set.contains(200));
        assert!(set.contains(1000));
        assert!(!set.contains(151));

        assert_eq!(set.len(), 3);

        // Test iterator with large values
        let mut elements: Vec<u32> = set.iter().collect();
        elements.sort();
        assert_eq!(elements, vec![150, 200, 1000]);
    }

    #[test]
    fn test_operations_with_large_values() {
        let mut set1 = BitSet::new();
        set1.insert(10);
        set1.insert(150);
        set1.insert(300);

        let mut set2 = BitSet::new();
        set2.insert(150);
        set2.insert(300);
        set2.insert(400);

        let union = set1.union(&set2);
        assert_eq!(union.len(), 4);
        assert!(union.contains(10));
        assert!(union.contains(400));

        let intersection = set1.intersection(&set2);
        assert_eq!(intersection.len(), 2);
        assert!(intersection.contains(150));
        assert!(intersection.contains(300));

        let difference = set1.difference(&set2);
        assert_eq!(difference.len(), 1);
        assert!(difference.contains(10));
    }
}
