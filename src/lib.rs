pub type HeapComparator<T> = fn(a: &T, b: &T) -> bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeapType {
    Min,
    Max,
}

#[derive(Debug)]
pub struct Heap<T> {
    data: Vec<T>,
    comparator: HeapComparator<T>,
}

impl<T: PartialOrd> Heap<T> {
    pub fn new(heap_type: HeapType) -> Self {
        Self {
            data: Vec::new(),
            comparator: Self::get_comparator(heap_type),
        }
    }

    #[deprecated(since = "0.1.0", note = "use `from_vec` instead")]
    pub fn from_array(heap_type: HeapType, input: Vec<T>) -> Self {
        Self::from_vec(heap_type, input)
    }

    pub fn from_vec(heap_type: HeapType, input: Vec<T>) -> Self {
        let mut heap = Self {
            data: input,
            comparator: Self::get_comparator(heap_type),
        };

        let size = heap.data.len();
        if size > 1 {
            for i in (0..size / 2).rev() {
                heap.float_down(i);
            }
        }

        heap
    }

    #[inline]
    const fn parent(index: usize) -> usize {
        (index - 1) / 2
    }

    fn float_down(&mut self, mut index: usize) {
        let len = self.data.len();
        loop {
            let left = 2 * index + 1;
            let right = left + 1;
            let mut candidate = index;

            if left < len && (self.comparator)(&self.data[candidate], &self.data[left]) {
                candidate = left;
            }
            if right < len && (self.comparator)(&self.data[candidate], &self.data[right]) {
                candidate = right;
            }

            if candidate == index {
                break;
            }

            self.data.swap(index, candidate);
            index = candidate;
        }
    }

    fn float_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = Self::parent(index);
            if (self.comparator)(&self.data[parent], &self.data[index]) {
                self.data.swap(parent, index);
                index = parent;
            } else {
                break;
            }
        }
    }

    pub fn insert(&mut self, value: T) {
        self.data.push(value);
        let new_size = self.data.len();
        if new_size > 1 {
            self.float_up(new_size - 1);
        }
    }

    #[must_use]
    pub fn root(&self) -> Option<&T> {
        self.data.first()
    }

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
        self.float_down(0);
        Some(result)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn raw(&self) -> &Vec<T> {
        &self.data
    }

    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn collect(&mut self) -> Vec<T> {
        let mut output = Vec::with_capacity(self.data.len());
        while let Some(value) = self.extract() {
            output.push(value);
        }
        output
    }

    fn get_comparator(heap_type: HeapType) -> HeapComparator<T> {
        match heap_type {
            HeapType::Min => |a, b| a > b,
            HeapType::Max => |a, b| a < b,
        }
    }

    #[allow(dead_code)]
    fn heapify(&mut self, index: usize) -> Option<usize> {
        let mut affected_index = index;

        let l_index = 2 * index + 1;
        let r_index = l_index + 1;

        let data = &mut self.data;
        let comparator = self.comparator;

        if data.get(l_index).is_some() && comparator(&data[affected_index], &data[l_index]) {
            affected_index = l_index;
        }

        if data.get(r_index).is_some() && comparator(&data[affected_index], &data[r_index]) {
            affected_index = r_index;
        }

        if affected_index != index {
            data.swap(index, affected_index);
            Some(affected_index)
        } else {
            None
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

    #[test]
    fn test_heapify() {
        let mut heap = Heap::<u32>::new(HeapType::Max);

        let input = vec![1, 3, 2];
        let expected = [3, 1, 2];

        heap.data = input;
        heap.heapify(0);

        let output = heap.raw();

        assert_eq!(*output, expected);
    }

    #[test]
    fn test_insert_min() {
        let mut heap = Heap::<u32>::new(HeapType::Min);

        heap.insert(10);
        assert_eq!(*heap.raw(), vec![10]);

        heap.insert(11);
        assert_eq!(*heap.raw(), vec![10, 11]);

        heap.insert(9);
        assert_eq!(*heap.raw(), vec![9, 11, 10]);

        heap.insert(5);
        assert_eq!(*heap.raw(), vec![5, 9, 10, 11]);

        heap.insert(6);
        assert_eq!(*heap.raw(), vec![5, 6, 10, 11, 9]);

        let mut expected = vec![5, 6, 10, 11, 9];
        expected.sort();
        assert_eq!(expected, heap.collect());
    }

    #[test]
    fn test_insert_max() {
        let mut heap = Heap::<u32>::new(HeapType::Max);

        heap.insert(10);
        assert_eq!(*heap.raw(), vec![10]);

        heap.insert(11);
        assert_eq!(*heap.raw(), vec![11, 10]);

        heap.insert(9);
        assert_eq!(*heap.raw(), vec![11, 10, 9]);

        heap.insert(5);
        assert_eq!(*heap.raw(), vec![11, 10, 9, 5]);

        heap.insert(6);
        assert_eq!(*heap.raw(), vec![11, 10, 9, 5, 6]);

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
        assert!(heap.raw().is_empty());
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
    fn test_duplicates() {
        let mut heap = Heap::<u32>::new(HeapType::Min);
        for _ in 0..5 {
            heap.insert(7);
        }
        assert_eq!(heap.collect(), vec![7; 5]);
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
                let mut expected = input.clone();
                expected.sort();
                assert_eq!(heap.collect(), expected);

                // insert max
                let mut heap = Heap::<u32>::new(HeapType::Max);
                for value in &input {
                    heap.insert(*value);
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
