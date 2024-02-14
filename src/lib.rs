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

impl<T> Heap<T> where T: PartialOrd + std::fmt::Display  {
    pub fn new(heap_type: HeapType) -> Self {
        Self {
            data: Vec::new(),
            comparator: Heap::get_comparator(heap_type),
        }
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

    pub fn insert(&mut self, value: T) {
        self.data.push(value);
        if self.data.len() == 1 {
            return;
        }
        self.up_head(self.data.len() - 1);
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
            self.down_head(0);

            Some(result)
        }
    }

    pub fn from_array(heap_type: HeapType, input: Vec<T>) -> Self {
        let mut heap = Self {
            data: input,
            comparator: Self::get_comparator(heap_type)
        };
        heap.normalize();
        heap
    }

    pub fn normalize(&mut self) {
        let mut i: usize = self.data.len() / 2;
        loop {
            eprintln!("{}", "top");
            let affected = self.heapify(i);
            if !affected.is_none() && i != 0 {
                let parent = (i - 1) / 2;
                eprintln!("{}", parent);
                self.heapify(parent);
            }
            i = i - 1;
            if i == 0 {
                break;
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn raw(&self) -> &Vec<T> {
        &self.data
    }

    fn heapify(&mut self, index: usize) -> Option<usize> {
        let mut max_index: usize = index;   // set parent as max element.

        let l_index = 2 * index + 1;        // get left child
        let r_index = 2 * index + 2;        // get right child

        let data = &mut self.data;          // to short
        let comparator = self.comparator;

        //  determinate max index
        if !data.get(l_index).is_none() && comparator(&data[max_index], &data[l_index]) {
            max_index = l_index;
        }

        if !data.get(r_index).is_none() && comparator(&data[max_index], &data[r_index]) {
            max_index = r_index;
        }

        if max_index != index {
            data.swap(index, max_index);
            Some(max_index)
        } else {
            None
        }
    }

    fn up_head(&mut self, index: usize) {
        let index = (index - 1) / 2;        // get parent.
        self.heapify(index);
        // go up!!
        if index != 0 {
            self.up_head(index);
        }
    }

    fn down_head(&mut self, index: usize) {
        let affected_index = self.heapify(index);
        match affected_index {
            None => return,
            Some(index) => {
                if index != (self.data.len() - 1) {
                    self.down_head(index)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Heap, HeapType};

    #[test]
    fn test_insert() {
        let mut heap = Heap::<u32>::new(HeapType::Max);
        let elements: u32 = 100;

        for i in 0..elements {
            heap.insert(i);
        }
        let mut last: u32 = heap.extract().unwrap();
        for _ in elements..0 {
            let current: u32 = heap.extract().unwrap();
            assert!(current > last);
            last = current;
        }

        let mut heap = Heap::<u32>::new(HeapType::Min);
        for i in 0..elements {
            heap.insert(i);
        }
        let mut last: u32 = heap.extract().unwrap();
        for _ in elements..0 {
            let current: u32 = heap.extract().unwrap();
            assert!(current < last);
            last = current;
        }

        let mut heap = Heap::<u32>::new(HeapType::Max);
        heap.insert(10);
        heap.insert(4);
        heap.insert(33);
        heap.insert(46);
        heap.insert(3);
        assert!(heap.root().unwrap() == &46u32);

        let mut heap = Heap::<u32>::new(HeapType::Min);
        heap.insert(10);
        heap.insert(4);
        heap.insert(33);
        heap.insert(46);
        heap.insert(3);
        assert!(heap.root().unwrap() == &3u32);

        let vec: Vec<u32> = vec![1, 7, 3, 4, 5, 6, 8];
        let heap = Heap::from_array(HeapType::Max, vec);
        let expected = 8u32;
        let result = heap.root().unwrap();
        assert_eq!(expected, *result);
    }
}