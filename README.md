# cubby

The use cases of cubby are probably very limited. It was created specifically to fill a niche issue where avoiding a mutex wrapped hashmap was important. Threaded Component Entity Systems and streaming socket servers both would benefit from this. Currently cubby is designed to simply provide you a key for your data, you use these keys to access and reference each value where needed. A collision of data is rather rare, though I suppose on some level possible (but probably astronomical).

##### benchmarks

These benchmarks satisfy the one specific goal I was hoping to achieve: a shared hashmap-like collection which is similar in speeds as a regular hashmap but greatly reduced thread contention. Typically I would choose to wrap something like this in a mutex, but this shows poor scalability when multiple threads needs to operate on the hashmap. cubby solves this by wrapping the elements of what would be the key-value in a mutex but keeping the collection itself external and fully visible. 

Below threaded benchmarks are as similar as I could manage, and represent creating 1 element each thread and working with the whole collection for 0 ms sleep (specific to the threaded benchmarks) with 10 threads staggered. The initial collection size before benchmarking is 1000 elements. 

cubby really shines when the whole collection is accessed from multiple threads, since in cubby this actually means individual element access-locks versus a master mutex style for the whole collection. You can see that even an immutable iteration on the collection is about the same speed as cubby (I'm very skeptical of this though). Benchmarks stating single are basic single-thread iteration of the collection, read-only.

```
test tests::bench_cubby        ... bench:       124 ns/iter (+/- 5)
test tests::bench_cubby_single ... bench:   9636278 ns/iter (+/- 238219)
test tests::bench_cubby_thread ... bench:  17790041 ns/iter (+/- 654473)
test tests::bench_hmap         ... bench:       101 ns/iter (+/- 1)
test tests::bench_hmap_single  ... bench:   9553934 ns/iter (+/- 167170)
test tests::bench_hmap_thread  ... bench:  98592221 ns/iter (+/- 2513362)
test tests::bench_imm_single   ... bench:   9565646 ns/iter (+/- 269270)
test tests::bench_imm_thread   ... bench:  16600285 ns/iter (+/- 490992)
test tests::bench_rwl_single   ... bench:   9553587 ns/iter (+/- 101141)
test tests::bench_rwl_thread   ... bench:  98262907 ns/iter (+/- 867130)
```
