use std::mem::ManuallyDrop;
use std::ptr;

/// Min-heap or max-heap selector for [`Heap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeapType {
    /// Root is always the smallest element.
    Min,
    /// Root is always the largest element.
    Max,
}

impl HeapType {
    /// `true` if `a` must yield its spot to `b`.
    #[inline]
    fn worse<T: PartialOrd>(self, a: &T, b: &T) -> bool {
        match self {
            HeapType::Min => a > b,
            HeapType::Max => a < b,
        }
    }
}

/// A min/max binary heap backed by a `Vec<T>`.
///
/// Only requires `PartialOrd`, so it works directly with floats. Values
/// that aren't comparable to each other (e.g. `NaN`) are never reordered
/// relative to one another, but nothing panics.
#[derive(Debug)]
pub struct Heap<T> {
    data: Vec<T>,
    heap_type: HeapType,
}

impl<T: PartialOrd> Heap<T> {
    /// Below this size, `extract` uses [`Heap::float_down_to_bottom`];
    /// above it, plain [`Heap::float_down`]. See `extract` and
    /// `benches/heap_benchmark.rs`.
    const SIFT_TO_BOTTOM_THRESHOLD: usize = 20_000;

    /// Creates an empty heap of the given type.
    pub fn new(heap_type: HeapType) -> Self {
        Self {
            data: Vec::new(),
            heap_type,
        }
    }

    /// Alias for [`Heap::from_vec`].
    #[deprecated(since = "0.1.0", note = "use `from_vec` instead")]
    pub fn from_array(heap_type: HeapType, input: Vec<T>) -> Self {
        Self::from_vec(heap_type, input)
    }

    /// Builds a heap from an existing `Vec<T>` in `O(n)`, cheaper than
    /// inserting each element one by one (`O(n log n)`).
    pub fn from_vec(heap_type: HeapType, input: Vec<T>) -> Self {
        let mut heap = Self {
            data: input,
            heap_type,
        };

        let size = heap.data.len();
        if size > 1 {
            for i in (0..size / 2).rev() {
                heap.float_down(i);
            }
        }

        heap
    }

    /// Sift-down: sinks the element at `pos` until the heap invariant
    /// holds. Used by `from_vec`'s heapify and by `extract` on large
    /// heaps. Stops as soon as the invariant holds, unlike
    /// [`Heap::float_down_to_bottom`].
    fn float_down(&mut self, pos: usize) {
        let len = self.data.len();
        let heap_type = self.heap_type;

        // SAFETY: `pos < len` is guaranteed by every caller (`from_vec`
        // only calls it with indices below `size`, `extract` with `0` on
        // a non-empty heap).
        let mut hole = unsafe { Hole::new(&mut self.data, pos) };
        let mut child = 2 * hole.pos() + 1;

        while child < len {
            let right = child + 1;
            // SAFETY: both indices are `< len`, i.e. within the slice,
            // and neither equals the hole's current position because
            // child/right are strictly below the hole's subtree root.
            if right < len && unsafe { heap_type.worse(hole.get(child), hole.get(right)) } {
                child = right;
            }

            // SAFETY: `child < len` and `child != hole.pos()`.
            if !unsafe { heap_type.worse(hole.element(), hole.get(child)) } {
                break;
            }

            // SAFETY: same as above.
            unsafe { hole.move_to(child) };
            child = 2 * hole.pos() + 1;
        }
    }

    /// Sift-up: lifts the element at `pos` toward `start` until the heap
    /// invariant holds. Used by `insert` (`start == 0`) and by
    /// [`Heap::float_down_to_bottom`] to settle an element it sank too far.
    fn float_up(&mut self, start: usize, pos: usize) {
        let heap_type = self.heap_type;

        // SAFETY: every caller passes a valid in-bounds index.
        let mut hole = unsafe { Hole::new(&mut self.data, pos) };

        while hole.pos() > start {
            let parent = (hole.pos() - 1) / 2;
            // SAFETY: `parent < hole.pos() < len` and `parent != hole.pos()`.
            if !unsafe { heap_type.worse(hole.get(parent), hole.element()) } {
                break;
            }
            // SAFETY: same as above.
            unsafe { hole.move_to(parent) };
        }
    }

    /// Sinks the element at `pos` all the way to the bottom, then
    /// `float_up`s it back into place. Fewer comparisons than
    /// [`Heap::float_down`] but touches more memory — used by `extract`
    /// below [`Heap::SIFT_TO_BOTTOM_THRESHOLD`].
    fn float_down_to_bottom(&mut self, pos: usize) {
        let heap_type = self.heap_type;
        let len = self.data.len();
        let start = pos;

        // SAFETY: caller (`extract`) guarantees `pos < len`.
        let mut hole = unsafe { Hole::new(&mut self.data, pos) };
        let mut child = 2 * hole.pos() + 1;

        while child < len {
            let right = child + 1;
            // SAFETY: both indices are `< len` and distinct from the hole.
            if right < len && unsafe { heap_type.worse(hole.get(child), hole.get(right)) } {
                child = right;
            }
            // SAFETY: `child < len` and `child != hole.pos()`.
            unsafe { hole.move_to(child) };
            child = 2 * hole.pos() + 1;
        }

        let final_pos = hole.pos();
        drop(hole);

        self.float_up(start, final_pos);
    }

    /// Inserts a value, preserving the heap invariant. `O(log n)`.
    pub fn insert(&mut self, value: T) {
        self.data.push(value);
        let new_size = self.data.len();
        if new_size > 1 {
            self.float_up(0, new_size - 1);
        }
    }

    /// Peeks at the root (min or max, per [`HeapType`]) without removing it.
    /// `None` if empty.
    #[must_use]
    pub fn root(&self) -> Option<&T> {
        self.data.first()
    }

    /// Removes and returns the root, re-heapifying the rest. `None` if
    /// empty. `O(log n)`.
    #[must_use]
    pub fn extract(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }
        if self.data.len() == 1 {
            return self.data.pop();
        }

        let last = self.data.pop().unwrap();
        let result = std::mem::replace(&mut self.data[0], last);

        if self.data.len() <= Self::SIFT_TO_BOTTOM_THRESHOLD {
            self.float_down_to_bottom(0);
        } else {
            self.float_down(0);
        }

        Some(result)
    }

    /// Returns the element at raw array position `index` (heap order, not
    /// sorted order). Mainly for tests/inspection.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Returns the internal `Vec<T>`.
    #[deprecated(since = "1.0.0", note = "use `as_slice` instead")]
    pub fn raw(&self) -> &Vec<T> {
        &self.data
    }

    /// Returns the heap's contents as a slice, in heap order (not sorted).
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Number of elements in the heap.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// `true` if the heap has no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Extracts every element in heap order (ascending for `Min`,
    /// descending for `Max`) into a `Vec<T>`, leaving the heap empty.
    /// `O(n log n)`.
    pub fn collect(&mut self) -> Vec<T> {
        let mut output = Vec::with_capacity(self.data.len());
        while let Some(value) = self.extract() {
            output.push(value);
        }
        output
    }
}

/// A vacated slot in a slice while its element is "in flight". Moving the
/// hole costs one write (`move_to`) instead of a full swap; the carried
/// element is written back once, on drop.
struct Hole<'a, T> {
    data: &'a mut [T],
    elt: ManuallyDrop<T>,
    pos: usize,
}

impl<'a, T> Hole<'a, T> {
    /// Creates a hole at `pos`, taking the element out of the slice.
    ///
    /// # Safety
    /// `pos` must be a valid index into `data`.
    #[inline]
    unsafe fn new(data: &'a mut [T], pos: usize) -> Self {
        debug_assert!(pos < data.len());
        // SAFETY: caller guarantees `pos` is in bounds.
        let elt = unsafe { ptr::read(data.get_unchecked(pos)) };
        Hole {
            data,
            elt: ManuallyDrop::new(elt),
            pos,
        }
    }

    /// Current position of the hole.
    #[inline]
    fn pos(&self) -> usize {
        self.pos
    }

    /// The element that was taken out of the slice to create the hole.
    #[inline]
    fn element(&self) -> &T {
        &self.elt
    }

    /// Returns a reference to the element at `index`.
    ///
    /// # Safety
    /// `index` must be a valid, in-bounds index different from `self.pos`.
    #[inline]
    unsafe fn get(&self, index: usize) -> &T {
        debug_assert!(index != self.pos);
        debug_assert!(index < self.data.len());
        // SAFETY: caller guarantees `index` is in bounds and not the hole.
        unsafe { self.data.get_unchecked(index) }
    }

    /// Moves the hole to `index`, copying the element currently there into
    /// the hole's old position.
    ///
    /// # Safety
    /// `index` must be a valid, in-bounds index different from `self.pos`.
    #[inline]
    unsafe fn move_to(&mut self, index: usize) {
        debug_assert!(index != self.pos);
        debug_assert!(index < self.data.len());
        // SAFETY: both `index` and `self.pos` are in bounds and distinct,
        // so the regions don't overlap.
        unsafe {
            let ptr = self.data.as_mut_ptr();
            ptr::copy_nonoverlapping(ptr.add(index), ptr.add(self.pos), 1);
        }
        self.pos = index;
    }
}

impl<T> Drop for Hole<'_, T> {
    /// Writes the displaced element back into its final resting place.
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `self.pos` is always a valid in-bounds index.
        unsafe {
            let pos = self.pos;
            ptr::copy_nonoverlapping(&*self.elt, self.data.get_unchecked_mut(pos), 1);
        }
    }
}

#[cfg(test)]
mod test {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    use crate::{Heap, HeapType};

    fn random_vec(size: u32) -> Vec<u32> {
        let mut result: Vec<u32> = (0u32..size).collect();
        let mut rng = thread_rng();

        result.shuffle(&mut rng);
        result
    }

    fn is_valid_heap<T: PartialOrd>(data: &[T], heap_type: HeapType) -> bool {
        for i in 0..data.len() {
            let left = 2 * i + 1;
            let right = left + 1;
            let violates = |child: usize| {
                data.get(child).is_some_and(|c| match heap_type {
                    HeapType::Min => data[i] > *c,
                    HeapType::Max => data[i] < *c,
                })
            };
            if violates(left) || violates(right) {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_insert_min() {
        let mut heap = Heap::<u32>::new(HeapType::Min);

        heap.insert(10);
        assert_eq!(heap.as_slice(), vec![10]);

        heap.insert(11);
        assert_eq!(heap.as_slice(), vec![10, 11]);

        heap.insert(9);
        assert_eq!(heap.as_slice(), vec![9, 11, 10]);

        heap.insert(5);
        assert_eq!(heap.as_slice(), vec![5, 9, 10, 11]);

        heap.insert(6);
        assert_eq!(heap.as_slice(), vec![5, 6, 10, 11, 9]);

        let mut expected = vec![5, 6, 10, 11, 9];
        expected.sort();
        assert_eq!(expected, heap.collect());
    }

    #[test]
    fn test_insert_max() {
        let mut heap = Heap::<u32>::new(HeapType::Max);

        heap.insert(10);
        assert_eq!(heap.as_slice(), vec![10]);

        heap.insert(11);
        assert_eq!(heap.as_slice(), vec![11, 10]);

        heap.insert(9);
        assert_eq!(heap.as_slice(), vec![11, 10, 9]);

        heap.insert(5);
        assert_eq!(heap.as_slice(), vec![11, 10, 9, 5]);

        heap.insert(6);
        assert_eq!(heap.as_slice(), vec![11, 10, 9, 5, 6]);

        let mut expected = vec![5, 6, 10, 11, 9];
        expected.sort();
        expected.reverse();
        assert_eq!(expected, heap.collect());
    }

    #[test]
    fn test_extract_min() {
        let mut heap = Heap::<u32>::new(HeapType::Min);
        let mut input: [u32; 4] = [10, 11, 9, 5];

        for i in input {
            heap.insert(i)
        }

        input.sort();
        for i in 0..input.len() {
            assert_eq!(input[i], heap.extract().unwrap());
        }
    }

    #[test]
    fn test_extract_max() {
        let mut heap = Heap::<u32>::new(HeapType::Max);
        let mut input: [u32; 4] = [10, 11, 9, 5];

        for i in input {
            heap.insert(i)
        }

        input.sort();
        input.reverse();
        for i in 0..input.len() {
            assert_eq!(input[i], heap.extract().unwrap());
        }
    }

    #[test]
    fn test_from_vec_min() {
        let input: Vec<u32> = random_vec(100);
        let mut heap = Heap::<u32>::from_vec(HeapType::Min, input.clone());

        let output = heap.collect();
        let mut expected = input.clone();
        expected.sort();

        assert_eq!(expected, output);
    }

    #[test]
    fn test_from_vec_edge() {
        // From empty vector
        let _ = Heap::<u32>::from_vec(HeapType::Max, Vec::new());
        // Single element
        let heap = Heap::<u32>::from_vec(HeapType::Max, vec![42]);
        assert_eq!(heap.root(), Some(&42));
    }

    #[test]
    fn test_from_vec_max() {
        let input: Vec<u32> = random_vec(100);
        let mut heap = Heap::<u32>::from_vec(HeapType::Max, input.clone());

        let output = heap.collect();
        let mut expected = input.clone();
        expected.sort();
        expected.reverse();

        assert_eq!(expected, output);
    }
    #[test]
    fn test_empty() {
        let mut heap = Heap::<i32>::new(HeapType::Min);
        assert_eq!(heap.root(), None);
        assert_eq!(heap.extract(), None);
        assert_eq!(heap.get(0), None);
        assert!(heap.as_slice().is_empty());
        assert!(heap.as_slice().is_empty());
        assert!(heap.collect().is_empty());
    }

    #[test]
    fn test_root_and_get() {
        let mut heap = Heap::<u32>::new(HeapType::Max);
        heap.insert(5);
        heap.insert(10);
        assert_eq!(heap.root(), Some(&10));
        assert!(heap.get(0).is_some());
        assert!(heap.get(1).is_some());
        assert!(heap.get(100).is_none());
    }

    #[test]
    fn test_duplicates_min() {
        let mut heap = Heap::<u32>::new(HeapType::Min);
        for _ in 0..5 {
            heap.insert(7);
        }
        assert_eq!(heap.collect(), vec![7; 5]);
    }

    #[test]
    fn test_duplicates_max() {
        let mut heap = Heap::<u32>::new(HeapType::Max);
        for _ in 0..5 {
            heap.insert(7);
        }
        assert_eq!(heap.collect(), vec![7; 5]);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut heap = Heap::<u32>::new(HeapType::Min);
        assert_eq!(heap.len(), 0);
        assert!(heap.is_empty());

        heap.insert(1);
        heap.insert(2);
        assert_eq!(heap.len(), 2);
        assert!(!heap.is_empty());

        let _ = heap.extract();
        let _ = heap.extract();
        assert_eq!(heap.len(), 0);
        assert!(heap.is_empty());
    }

    #[test]
    fn test_extract_size_two() {
        let mut heap = Heap::<u32>::new(HeapType::Min);
        heap.insert(5);
        heap.insert(3);
        assert_eq!(heap.extract(), Some(3));
        assert_eq!(heap.extract(), Some(5));
        assert_eq!(heap.extract(), None);
    }

    #[test]
    fn test_from_vec_already_valid_heap() {
        // Already a valid max-heap; from_vec should be a no-op in terms of ordering.
        let input = vec![10, 5, 8, 1, 4, 7, 6];
        assert!(is_valid_heap(&input, HeapType::Max));

        let heap = Heap::<u32>::from_vec(HeapType::Max, input.clone());
        assert!(is_valid_heap(heap.as_slice(), HeapType::Max));
        assert_eq!(heap.root(), Some(&10));
    }

    #[test]
    fn test_floats() {
        let input = vec![3.5f64, 1.2, 9.9, -2.3, 0.0];

        let mut min_heap = Heap::<f64>::from_vec(HeapType::Min, input.clone());
        let mut expected = input.clone();
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(min_heap.collect(), expected);

        let mut max_heap = Heap::<f64>::from_vec(HeapType::Max, input.clone());
        let mut expected = input.clone();
        expected.sort_by(|a, b| b.partial_cmp(a).unwrap());
        assert_eq!(max_heap.collect(), expected);
    }

    #[test]
    fn test_interleaved_insert_extract() {
        let mut heap = Heap::<u32>::new(HeapType::Min);

        heap.insert(5);
        heap.insert(3);
        assert_eq!(heap.extract(), Some(3));
        assert!(is_valid_heap(heap.as_slice(), HeapType::Min));

        heap.insert(1);
        heap.insert(8);
        assert_eq!(heap.extract(), Some(1));
        assert!(is_valid_heap(heap.as_slice(), HeapType::Min));

        heap.insert(2);
        assert_eq!(heap.extract(), Some(2));
        assert_eq!(heap.extract(), Some(5));
        assert!(is_valid_heap(heap.as_slice(), HeapType::Min));

        heap.insert(0);
        assert_eq!(heap.extract(), Some(0));
        assert_eq!(heap.extract(), Some(8));
        assert_eq!(heap.extract(), None);
    }

    #[test]
    fn test_from_vec_sizes() {
        for size in 0..=5 {
            let input: Vec<u32> = (0..size).rev().collect();
            let mut heap = Heap::<u32>::from_vec(HeapType::Min, input.clone());
            let mut expected = input.clone();
            expected.sort();
            assert_eq!(heap.collect(), expected);
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_from_array() {
        let mut heap = Heap::<u32>::from_array(HeapType::Max, vec![1, 2, 3]);
        assert_eq!(heap.collect(), vec![3, 2, 1]);
    }

    #[test]
    fn test_random_property() {
        for &size in &[0u32, 1, 2, 5, 50] {
            for _ in 0..20 {
                let input = random_vec(size);

                // from_vec min
                let mut heap = Heap::<u32>::from_vec(HeapType::Min, input.clone());
                assert!(is_valid_heap(heap.as_slice(), HeapType::Min));
                let mut expected = input.clone();
                expected.sort();
                assert_eq!(heap.collect(), expected);

                // insert max
                let mut heap = Heap::<u32>::new(HeapType::Max);
                for value in &input {
                    heap.insert(*value);
                    assert!(is_valid_heap(heap.as_slice(), HeapType::Max));
                }
                let mut expected = input.clone();
                expected.sort();
                expected.reverse();
                let mut actual = Vec::new();
                while let Some(value) = heap.extract() {
                    actual.push(value);
                }
                assert_eq!(actual, expected);
            }
        }
    }

}
