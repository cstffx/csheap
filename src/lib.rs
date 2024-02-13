use std::fmt::Display;
use std::io;

#[derive(Debug)]
struct Heap<T> {
    data: Vec<T>,
}

impl<T> Heap<T> where T: PartialOrd {

    pub fn new() -> Heap<T> {
        Heap {
            data: Vec::new()
        }
    }

    pub fn insert(&mut self, value: T) {
        self.data.push(value);
        if self.data.len() == 1 {
            return;
        }
        self.up_head(self.data.len() - 1);
    }

    fn extract(&mut self) -> Option<T> {
        if (self.data.is_empty()) {
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

    fn heapify(&mut self, index: usize) -> Option<usize> {
        let mut max_index: usize = index;   // set parent as max element.

        let l_index = 2 * index + 1;        // get left child
        let r_index = 2 * index + 2;        // get right child

        let data = &mut self.data;          // to short

        //  determinate max index
        if !data.get(l_index).is_none() && data[l_index] < data[max_index] {
            max_index = l_index;
        }

        if !data.get(r_index).is_none() && data[r_index] < data[max_index] {
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
                if (index != (self.data.len() - 1)) {
                    self.down_head(index)
                }
            }
        }
    }
}

mod test {
    use crate::Heap;
    #[test]
    fn test_insert() {
        let mut heap = Heap::<u32>::new();
        for i in 0..10 {
            heap.insert(i);
        }
        let mut last: u32 = heap.extract().unwrap();
        for _ in 0..9 {
            let current: u32 = heap.extract().unwrap();
            assert!(current > last);
            last = current;
        }
    }
}