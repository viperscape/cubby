# cubby

The use cases for cubby might be limited, but it was created specifically to fill a niche issue where avoiding a mutex wrapped hashmap was important. Threaded Component Entity Systems and streaming socket servers that have a central data store both would benefit from this. Currently cubby is designed to simply provide you a key for your data, you use these keys to access and reference each value where needed.

The sync primitives for the data is setup with traits so mutex can be swapped out for spinlock, etc, by [implementing your own backends](https://github.com/viperscape/cubby/blob/master/src/rwlock.rs). cubby provides an entity id system with its key access, in the form of a tuple ```(usize for index,random u64 for id)``` which is verified during access; and leverages a fast pre-allocated array with convenience methods built in. Each key accesses a specific entity in the collection, and provides sync access without blocking the collection as a whole. 

==

#### example ####

```rust
extern crate cubby;
use cubby::{rwlock};

fn main () {
    let c = rwlock::build(25);
    let e = c.add(2).unwrap();
    c.with_mut(e,|n| *n+=1);
    let r = c.with(e,|n| *n).unwrap(); //this example copies the value
    assert_eq!(r,3);
}
```

see [tests](https://github.com/viperscape/cubby/blob/master/src/tests.rs) for more examples

==

#### benchmarks ####

```
test tests::tests::bench_cubby_mutex  ... bench:       234 ns/iter (+/- 9)
test tests::tests::bench_cubby_rwlock ... bench:       354 ns/iter (+/- 10)
test tests::tests::bench_hmap_mutex   ... bench:       375 ns/iter (+/- 13)
```
