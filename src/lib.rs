#![feature(is_sorted)]

macro_rules! get_parent {
    ($index: expr) => {
        ($index - 1) / 2
    };
}

pub type HeapComparator<T> = fn(a: &T, b: &T) -> bool;

#[derive(PartialEq)]
pub enum HeapType {
    Min,
    Max,
}

#[derive(Debug)]
pub struct Heap<T> {
    data: Vec<T>,
    comparator: HeapComparator<T>,
}

impl<T> Heap<T> where T: PartialOrd + std::fmt::Display {
    pub fn new(heap_type: HeapType) -> Self {
        Self {
            data: Vec::new(),
            comparator: Heap::get_comparator(heap_type),
        }
    }

    #[deprecated]
    pub fn from_array(heap_type: HeapType, input: Vec<T>) -> Self {
        Heap::from_vec(heap_type, input)
    }

    pub fn from_vec(heap_type: HeapType, input: Vec<T>) -> Self {
        let mut heap = Self {
            data: input,
            comparator: Self::get_comparator(heap_type),
        };

        let size = heap.data.len();
        if size == 0 {
            return heap;
        }

        let mut i: usize = (size / 2) - 1;
        loop {
            let mut do_more = heap.heapify(i);
            while do_more.is_some() {
                do_more = heap.heapify(do_more.unwrap());
            }
            if i != 0 {
                i = i - 1;
            } else {
                break;
            }
        }

        heap
    }

    fn float_down(&mut self, index: usize) {
        let mut do_more = self.heapify(index);
        while do_more.is_some() {
            do_more = self.heapify(do_more.unwrap());
        }
    }

    fn float_up(&mut self, index: usize) {
        let mut _index = get_parent!(index);
        let mut do_more = self.heapify(_index);

        while _index != 0 && do_more.is_some() {
            _index = get_parent!(_index);
            do_more = self.heapify(_index);
        }
    }

    pub fn insert(&mut self, value: T) {
        self.data.push(value);
        let new_size = self.data.len();
        if new_size > 1 {
            self.float_up(new_size - 1);
        }
    }

    pub fn root(&self) -> Option<&T> {
        self.data.get(0)
    }

    pub fn extract(&mut self) -> Option<T> {
        if self.data.is_empty() {
            None
        } else if self.data.len() == 1 {
            Some(self.data.remove(0))
        } else {
            let last_index = self.data.len() - 1;
            self.data.swap(0, last_index);

            let result = self.data.remove(last_index);

            self.float_down(0);

            Some(result)
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn raw(&self) -> &Vec<T> {
        &self.data
    }

    pub fn collect(&mut self) -> Vec<T> {
        let mut output: Vec<T> = Vec::new();
        for _ in 0..self.data.len() {
            output.push(self.extract().unwrap());
        }
        output
    }

    fn get_comparator(heap_type: HeapType) -> HeapComparator<T> {
        match heap_type {
            HeapType::Min => |a: &T, b: &T| {
                a > b
            },
            HeapType::Max => |a: &T, b: &T| -> bool {
                a < b
            }
        }
    }

    fn heapify(&mut self, index: usize) -> Option<usize> {
        let mut affected_index: usize = index;   // set parent as max element.

        let l_index = 2 * index + 1;        // get left child
        let r_index = 2 * index + 2;        // get right child

        let data = &mut self.data;          // to short
        let comparator = self.comparator;

        //  determinate max index
        if !data.get(l_index).is_none() && comparator(&data[affected_index], &data[l_index]) {
            affected_index = l_index;
        }

        if !data.get(r_index).is_none() && comparator(&data[affected_index], &data[r_index]) {
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
}