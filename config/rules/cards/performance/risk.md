### Highest-performance correct implementation
Prefer the highest-performance correct implementation. Then simplify for readability and reviewability, but never trade meaningful performance for brevity or superficial simplicity.

### Bounded work
Avoid unbounded work on growing inputs. Add pagination, limits, early exits, batching, streaming, or explicit workload bounds.

### N+1 work
Avoid nested per-item database, network, filesystem, or expensive computation for list/batch paths.
Require batching, pagination, or an explicit bound.

### Workload validation
Validate or cap workload size before allocating, sorting, logging, serializing, or spawning work proportional to user-controlled input.

### Unsafe concurrency
Avoid unbounded fan-out, shared mutable state races, blocking calls in async paths, and missing backpressure.

### Avoid discarded full work
Do not compute or sort results that will be discarded when a bounded/top-N algorithm is available.
Example: select the top slice, then sort only the kept slice.

### Grounded performance judgment
Read referenced target code before judging performance risk. Do not infer N+1 or unbounded-work risk from plan wording alone when target code is available.
