# csheap
Simple heap implementation over a vector.

```rust
    // Create a new heap instance for u32 elements.   
    let mut heap = Heap::<u32>::new(HeapType::Max);

    // Create a new heap instance from an u32 vector.
    // this take the ownership of the vector.
    let mut heap = Heap::<u32>::from_vec(HeapType::Min, some_vector);

    // To avoid this, clone the vector.
    let mut heap = Heap::<u32>::from_vec(HeapType::Min, some_vector.clone());
```

There are two basic operations: 
- ``insert``: Insert a new element in the heap.
- ``extract``: Remove an element and return this based on the type of the heap.

Heaps comes in two flavors: ``Min`` and ``Max``. 
- ``Min``: ``extract`` always take first the minimal element. 
- ``Max``: ``extract`` always take first the maximal element. 

```rust
    // Create a heap that always return the maximal element
    let mut heap = Heap::<u32>::new(HeapType::Max);

    heap.insert(1u32); 
    heap.insert(2u32);

    heap.extract();     // returns 2
```