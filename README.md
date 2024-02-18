# csheap
Min and max heap implementation over a vector. This is a efficient implementation 
of the ADT priority queue.

```rust
// Create a new heap instance for u32 elements.   
let mut heap = Heap::<u32>::new(HeapType::Max);

// Create a new heap instance from an u32 vector.
// Will take the ownership of the vector.
let mut heap = Heap::<u32>::from_vec(HeapType::Min, some_vector);

// To avoid taking the ownership of the vector you can, 
// for example, clone the vector.
let mut heap = Heap::<u32>::from_vec(HeapType::Min, some_vector.clone());
```

There are two basic operations: 
- ``insert``: Insert an element.
- ``extract``: Remove and return the element in the root node.

Heaps comes in two flavors: ``Min`` and ``Max``. 
- ``Min``: ``extract`` always take the min element. 
- ``Max``: ``extract`` always take the max element. 

```rust
// Create a heap that always return the maximal element
let mut heap = Heap::<u32>::new(HeapType::Max);

heap.insert(1u32); 
heap.insert(2u32);

heap.extract();     // returns 2
```