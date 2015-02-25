# cubby

The use cases of cubby are probably very limited. It was created specifically to fill a niche issue where avoiding a mutex wrapped hashmap was important. Threaded Component Entity Systems and streaming socket servers both would benefit from this. Currently cubby is designed to simply provide you a key for your data, you use these keys to access and reference each value where needed. A collision of data is rather rare, though I suppose on some level possible (but probably astronomical).

##### benchmarks

These benchmarks satisfy the one specific goal I was hoping to achieve: a shared hashmap-like collection which is similar in speeds as a regular hashmap but greatly reduced thread contention. Typically I would choose to wrap something like this in a mutex, but this shows poor scalability when multiple threads needs to operate on the hashmap. cubby solves this by wrapping the elements of what would be the key-value in a mutex but keeping the collection itself external and fully visible. Below threaded benchmarks are as similar as I could manage, and represent creating data and working with the whole set for 0 ms sleep (specific to the threaded benchmarks) with 10 threads staggered. You'll see that hashmap-bench and rwlock-bench bottlenecks on the mutex types for its collection. On the otherhand, on almost every other benchmark rwlock and master mutex is more performant. cubby really shines when the whole collection is accessed from multiple threads, since in cubby this actually means individual element access-locks versus a master mutex style for the whole collection.

```
test tests::bench_cubby        ... bench:       124 ns/iter (+/- 6)
test tests::bench_cubby_thread ... bench:   3963417 ns/iter (+/- 107369)
test tests::bench_hmap         ... bench:       103 ns/iter (+/- 6)
test tests::bench_hmap_thread  ... bench:  12224305 ns/iter (+/- 235524)
test tests::bench_rwl_thread   ... bench:  12116093 ns/iter (+/- 194273)
```
