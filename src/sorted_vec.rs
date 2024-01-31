//! https://gitlab.com/spearman/sorted-vec/-/blob/master/src/partial.rs
//! Sorted vectors of types implementing `PartialOrd`.
//!
//! It is a runtime panic if an incomparable element is compared.

//! Sorted vectors.
//!
//! [Repository](https://gitlab.com/spearman/sorted-vec)
//!
//! - `SortedVec` -- sorted from least to greatest, may contain duplicates
//! - `ReverseSortedVec` -- sorted from greatest to least, may contain

#![allow(dead_code)]

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

/// Forward sorted vector
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SortedVec<T: Ord + Send> {
    #[serde(bound(deserialize = "T : serde::Deserialize <'de> + Debug"))]
    vec: Vec<T>,
}

/// Value returned when find_or_insert is used.
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum FindOrInsert {
    /// Contains a found index
    Found(usize),

    /// Contains an inserted index
    Inserted(usize),
}

/// Converts from the binary_search result type into the FindOrInsert type
impl From<Result<usize, usize>> for FindOrInsert {
    fn from(result: Result<usize, usize>) -> Self {
        match result {
            Result::Ok(value) => FindOrInsert::Found(value),
            Result::Err(value) => FindOrInsert::Inserted(value),
        }
    }
}

impl FindOrInsert {
    /// Get the index of the element that was either found or inserted.
    pub fn index(&self) -> usize {
        match self {
            FindOrInsert::Found(value) | FindOrInsert::Inserted(value) => *value,
        }
    }

    /// If an equivalent element was found in the container, get the value of
    /// its index. Otherwise get None.
    pub fn found(&self) -> Option<usize> {
        match self {
            FindOrInsert::Found(value) => Some(*value),
            FindOrInsert::Inserted(_) => None,
        }
    }

    /// If the provided element was inserted into the container, get the value
    /// of its index. Otherwise get None.
    pub fn inserted(&self) -> Option<usize> {
        match self {
            FindOrInsert::Found(_) => None,
            FindOrInsert::Inserted(value) => Some(*value),
        }
    }

    /// Returns true if the element was found.
    pub fn is_found(&self) -> bool {
        matches!(self, FindOrInsert::Found(_))
    }

    /// Returns true if the element was inserted.
    pub fn is_inserted(&self) -> bool {
        matches!(self, FindOrInsert::Inserted(_))
    }
}

//
//  impl SortedVec
//

impl<T: Ord + Send> SortedVec<T> {
    #[inline]
    pub fn new() -> Self {
        SortedVec { vec: Vec::new() }
    }
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        SortedVec {
            vec: Vec::with_capacity(capacity),
        }
    }
    /// Uses `sort_unstable()` to sort in place.
    #[inline]
    pub fn from_unsorted(mut vec: Vec<T>) -> Self {
        vec.sort_unstable();
        SortedVec { vec }
    }
    /// Insert an element into sorted position, returning the order index at which
    /// it was placed.
    pub fn insert(&mut self, element: T) -> usize {
        let insert_at = match self.binary_search(&element) {
            Ok(insert_at) | Err(insert_at) => insert_at,
        };
        self.vec.insert(insert_at, element);
        insert_at
    }
    /// Find the element and return the index with `Ok`, otherwise insert the
    /// element and return the new element index with `Err`.
    pub fn find_or_insert(&mut self, element: T) -> FindOrInsert {
        self.binary_search(&element)
            .map_err(|insert_at| {
                self.vec.insert(insert_at, element);
                insert_at
            })
            .into()
    }
    /// Same as insert, except performance is O(1) when the element belongs at the
    /// back of the container. This avoids an O(log(N)) search for inserting
    /// elements at the back.
    #[inline]
    pub fn push(&mut self, element: T) -> usize {
        if let Some(last) = self.vec.last() {
            let cmp = element.cmp(last);
            if cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal {
                // The new element is greater than or equal to the current last element,
                // so we can simply push it onto the vec.
                self.vec.push(element);
                self.vec.len() - 1
            } else {
                // The new element is less than the last element in the container, so we
                // cannot simply push. We will fall back on the normal insert behavior.
                self.insert(element)
            }
        } else {
            // If there is no last element then the container must be empty, so we
            // can simply push the element and return its index, which must be 0.
            self.vec.push(element);
            0
        }
    }
    /// Reserves additional capacity in the underlying vector.
    /// See std::vec::Vec::reserve.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
    }
    /// Same as find_or_insert, except performance is O(1) when the element
    /// belongs at the back of the container.
    pub fn find_or_push(&mut self, element: T) -> FindOrInsert {
        if let Some(last) = self.vec.last() {
            let cmp = element.cmp(last);
            if cmp == std::cmp::Ordering::Equal {
                FindOrInsert::Found(self.vec.len() - 1)
            } else if cmp == std::cmp::Ordering::Greater {
                self.vec.push(element);
                return FindOrInsert::Inserted(self.vec.len() - 1);
            } else {
                // The new element is less than the last element in the container, so we
                // need to fall back on the regular find_or_insert
                return self.find_or_insert(element);
            }
        } else {
            // If there is no last element then the container must be empty, so we can
            // simply push the element and return that it was inserted.
            self.vec.push(element);
            FindOrInsert::Inserted(0)
        }
    }
    #[inline]
    pub fn remove_item(&mut self, item: &T) -> Option<T> {
        match self.vec.binary_search(item) {
            Ok(remove_at) => Some(self.vec.remove(remove_at)),
            Err(_) => None,
        }
    }
    /// Panics if index is out of bounds
    #[inline]
    pub fn remove_index(&mut self, index: usize) -> T {
        self.vec.remove(index)
    }
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.vec.pop()
    }
    #[inline]
    pub fn clear(&mut self) {
        self.vec.clear()
    }
    #[inline]
    pub fn dedup(&mut self) {
        self.vec.dedup();
    }
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq<K>,
    {
        self.vec.dedup_by_key(key);
    }
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> std::vec::Drain<T>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.vec.drain(range)
    }
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.vec.retain(f)
    }
    /// NOTE: to_vec() is a slice method that is accessible through deref, use
    /// this instead to avoid cloning
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.vec
    }
    /// Apply a closure mutating the sorted vector and use `sort_unstable()`
    /// to re-sort the mutated vector
    pub fn mutate_vec<F, O>(&mut self, f: F) -> O
    where
        F: FnOnce(&mut Vec<T>) -> O,
    {
        let res = f(&mut self.vec);
        self.vec.sort_unstable();
        res
    }
    /// The caller must ensure that the provided vector is already sorted.
    #[inline]
    pub unsafe fn from_sorted(vec: Vec<T>) -> Self {
        SortedVec { vec }
    }
    /// Unsafe access to the underlying vector. The caller must ensure that any
    /// changes to the values in the vector do not impact the ordering of the
    /// elements inside, or else this container will misbehave.
    pub unsafe fn get_unchecked_mut_vec(&mut self) -> &mut Vec<T> {
        &mut self.vec
    }

    /// Perform sorting on the input sequence when deserializing with `serde`.
    ///
    /// Use with `#[serde(deserialize_with = "SortedVec::deserialize_unsorted")]`:
    /// ```text
    /// #[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    /// pub struct Foo {
    ///   #[serde(deserialize_with = "SortedVec::deserialize_unsorted")]
    ///   pub v : SortedVec <u64>
    /// }
    /// ```
    pub fn deserialize_unsorted<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de>,
    {
        let v = Vec::deserialize(deserializer)?;
        Ok(SortedVec::from_unsorted(v))
    }

    // fn parse_vec<'de, D>(deserializer: D) -> Result<Vec<T>, D::Error>
    // where
    //     D: serde::Deserializer<'de>,
    //     T: serde::Deserialize<'de> + Debug,
    // {
    //     use serde::de::Error;
    //     // deserializer.deserialize_seq();
    //     // Deserializer::deserialize_seq(deserializer);
    //     // SeqDeserializer::new(deserializer);
    //     let mut v = Vec::deserialize(deserializer)?;
    //     dbg!(&v);
    //     // if !IsSorted::is_sorted(&mut v.iter_mut()) {
    //     //     Err(D::Error::custom("input sequence is not sorted"))
    //     // } else {
    //     Ok(v)
    //     // }
    // }
}
impl<T: Ord + Send> Default for SortedVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Ord + Send> From<Vec<T>> for SortedVec<T> {
    fn from(unsorted: Vec<T>) -> Self {
        Self::from_unsorted(unsorted)
    }
}

impl<T: Ord + Send> std::ops::Deref for SortedVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.vec
    }
}

impl<T: Ord + Send> std::ops::DerefMut for SortedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}
impl<T: Ord + Send> Extend<T> for SortedVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for t in iter {
            let _ = self.insert(t);
        }
    }
}
impl<T: Ord + Hash + Send> Hash for SortedVec<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let v: &Vec<T> = self.as_ref();
        v.hash(state);
    }
}

/// Reverse-sorted Containers.
///
/// Use these containers to have the vector sorted in the reverse order of its
/// usual comparison.
///
/// Note that objects going into the reverse container needs to be wrapped in
/// std::cmp::Reverse.
///
/// # Examples
///
/// ```
/// use std::cmp::Reverse;
/// use sorted_vec::ReverseSortedVec;
///
/// let mut vec = ReverseSortedVec::<u64>::new();
/// vec.insert(Reverse(10));
/// vec.insert(Reverse(15));
/// assert_eq!(vec.last().unwrap().0, 10);
/// ```
pub type ReverseSortedVec<T> = SortedVec<std::cmp::Reverse<T>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Reverse;

    #[test]
    fn test_sorted_vec() {
        let mut v = SortedVec::new();
        assert_eq!(v.insert(5), 0);
        assert_eq!(v.insert(3), 0);
        assert_eq!(v.insert(4), 1);
        assert_eq!(v.insert(4), 1);
        assert_eq!(v.find_or_insert(4), FindOrInsert::Found(2));
        assert_eq!(v.find_or_insert(4).index(), 2);
        assert_eq!(v.len(), 4);
        v.dedup();
        assert_eq!(v.len(), 3);
        assert_eq!(v.binary_search(&3), Ok(0));
        assert_eq!(
            *SortedVec::from_unsorted(vec![5, -10, 99, -11, 2, 17, 10]),
            vec![-11, -10, 2, 5, 10, 17, 99]
        );
        assert_eq!(
            SortedVec::from_unsorted(vec![5, -10, 99, -11, 2, 17, 10]),
            vec![5, -10, 99, -11, 2, 17, 10].into()
        );
        let mut v = SortedVec::new();
        v.extend(vec![5, -10, 99, -11, 2, 17, 10]);
        assert_eq!(*v, vec![-11, -10, 2, 5, 10, 17, 99]);
        v.mutate_vec(|v| {
            v[0] = 11;
            v[3] = 1;
        });
        assert_eq!(
            v.drain(..).collect::<Vec<i32>>(),
            vec![-10, 1, 2, 10, 11, 17, 99]
        );
    }

    #[test]
    fn test_sorted_vec_push() {
        let mut v = SortedVec::new();
        assert_eq!(v.push(5), 0);
        assert_eq!(v.push(3), 0);
        assert_eq!(v.push(4), 1);
        assert_eq!(v.push(4), 1);
        assert_eq!(v.find_or_push(4), FindOrInsert::Found(2));
        assert_eq!(v.find_or_push(4).index(), 2);
        assert_eq!(v.len(), 4);
        v.dedup();
        assert_eq!(v.len(), 3);
        assert_eq!(v.binary_search(&3), Ok(0));
        assert_eq!(
            *SortedVec::from_unsorted(vec![5, -10, 99, -11, 2, 17, 10]),
            vec![-11, -10, 2, 5, 10, 17, 99]
        );
        assert_eq!(
            SortedVec::from_unsorted(vec![5, -10, 99, -11, 2, 17, 10]),
            vec![5, -10, 99, -11, 2, 17, 10].into()
        );
        let mut v = SortedVec::new();
        v.extend(vec![5, -10, 99, -11, 2, 17, 10]);
        assert_eq!(*v, vec![-11, -10, 2, 5, 10, 17, 99]);
        v.mutate_vec(|v| {
            v[0] = 11;
            v[3] = 1;
        });
        assert_eq!(
            v.drain(..).collect::<Vec<i32>>(),
            vec![-10, 1, 2, 10, 11, 17, 99]
        );
    }

    #[test]
    fn test_reverse_sorted_vec() {
        let mut v = ReverseSortedVec::new();
        assert_eq!(v.insert(Reverse(5)), 0);
        assert_eq!(v.insert(Reverse(3)), 1);
        assert_eq!(v.insert(Reverse(4)), 1);
        assert_eq!(v.find_or_insert(Reverse(6)), FindOrInsert::Inserted(0));
        assert_eq!(v.insert(Reverse(4)), 2);
        assert_eq!(v.find_or_insert(Reverse(4)), FindOrInsert::Found(2));
        assert_eq!(v.len(), 5);
        v.dedup();
        assert_eq!(v.len(), 4);
        assert_eq!(
            *ReverseSortedVec::from_unsorted(Vec::from_iter(
                [5, -10, 99, -11, 2, 17, 10].map(Reverse)
            )),
            Vec::from_iter([99, 17, 10, 5, 2, -10, -11].map(Reverse))
        );
        assert_eq!(
            ReverseSortedVec::from_unsorted(Vec::from_iter(
                [5, -10, 99, -11, 2, 17, 10].map(Reverse)
            )),
            Vec::from_iter([5, -10, 99, -11, 2, 17, 10].map(Reverse)).into()
        );
        let mut v = ReverseSortedVec::new();
        v.extend([5, -10, 99, -11, 2, 17, 10].map(Reverse));
        assert_eq!(v.as_slice(), [99, 17, 10, 5, 2, -10, -11].map(Reverse));
        v.mutate_vec(|v| {
            v[6] = Reverse(11);
            v[3] = Reverse(1);
        });
        assert_eq!(
            v.drain(..).collect::<Vec<Reverse<i32>>>(),
            Vec::from_iter([99, 17, 11, 10, 2, 1, -10].map(Reverse))
        );
    }

    #[test]
    fn test_deserialize() {
        let s = r#"[-11,-10,2,5,10,17,99]"#;
        let _ = dbg!(serde_json::from_str::<SortedVec<i32>>(s)).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_deserialize_unsorted() {
        let s = "[99,-11,-10,2,5,10,17]";
        let _ = serde_json::from_str::<SortedVec<i32>>(s).unwrap();
    }
    #[test]
    fn test_deserialize_reverse() {
        let s = "[99,17,10,5,2,-10,-11]";
        let _ = serde_json::from_str::<ReverseSortedVec<i32>>(s).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_deserialize_reverse_unsorted() {
        let s = "[99,-11,-10,2,5,10,17]";
        let _ = serde_json::from_str::<ReverseSortedVec<i32>>(s).unwrap();
    }
}
