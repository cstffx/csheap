# csheap

A min/max heap implementation backed by a vector — an efficient
implementation of the priority queue ADT.

```rust
// Create a new heap instance for u32 elements.
let mut heap = Heap::<u32>::new(HeapType::Max);

// Create a new heap instance from a u32 vector.
// This takes ownership of the vector.
let mut heap = Heap::<u32>::from_vec(HeapType::Min, some_vector);

// To avoid taking ownership of the vector, you can
// clone it first, for example.
let mut heap = Heap::<u32>::from_vec(HeapType::Min, some_vector.clone());
```

There are two basic operations:
- `insert`: insert an element.
- `extract`: remove and return the element at the root node.

Heaps come in two flavors: `Min` and `Max`.
- `Min`: `extract` always returns the minimum element.
- `Max`: `extract` always returns the maximum element.

```rust
// Create a heap that always returns the maximum element.
let mut heap = Heap::<u32>::new(HeapType::Max);

heap.insert(1u32);
heap.insert(2u32);

heap.extract();     // returns 2
```

## `csheap` vs. `std::collections::BinaryHeap`

| | `std::collections::BinaryHeap` | `csheap::Heap` |
|---|---|---|
| Heap kind | Max-heap only (wrap in `std::cmp::Reverse` for min-heap behavior) | `Min` or `Max`, chosen at runtime via `HeapType` |
| Bound on `T` | Requires `Ord` (total order) | Only requires `PartialOrd` |
| Floats | Needs a wrapper (e.g. `ordered-float`) because `f64` isn't `Ord` | Works directly, as long as you don't rely on ordering `NaN` against other values |
| API | `push` / `pop` / `peek` | `insert` / `extract` / `root` |

### Expected performance

`std::BinaryHeap` is faster — it requires `Ord`, which lets the compiler
drop straight to a single CPU comparison instruction, while `csheap` only
requires `PartialOrd` and additionally picks `Min`/`Max` at runtime.
Measured with `cargo bench` (`benches/heap_benchmark.rs`, `u32`, 100k
elements, your numbers will vary by machine and element type):

| Operation | `csheap::Heap` vs. `std::BinaryHeap` |
|---|---|
| Build from an existing `Vec` (heapify) | ~2x slower |
| Extract all elements | ~2–2.3x slower |

Treat this as "same order of magnitude, consistently a bit slower," not
as a different complexity class — both are `O(log n)` per operation.

### When to use which

Use **`std::BinaryHeap`** when:
- Your element type already implements `Ord` (integers, strings, or
  anything with a well-defined total order).
- You only need a max-heap (or are fine wrapping in `std::cmp::Reverse`
  for a min-heap).
- Raw throughput matters and you don't want an extra dependency.

Use **`csheap`** when:
- You need to heap values that are only `PartialOrd`, most commonly
  floats (`f32`/`f64`), without pulling in a wrapper crate like
  `ordered-float`.
- You want to pick `Min` or `Max` at runtime (e.g. the heap type depends
  on a config flag or user input) instead of committing to one at the
  type level.
- The ~2x gap doesn't matter for your workload — e.g. the heap isn't the
  bottleneck, or it holds a modest number of elements.