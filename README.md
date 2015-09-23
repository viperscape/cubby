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
test tests::tests::bench_cubby_mutex  ... bench:       226 ns/iter (+/- 9)
test tests::tests::bench_cubby_rwlock ... bench:       245 ns/iter (+/- 10)
test tests::tests::bench_hmap_mutex   ... bench:       390 ns/iter (+/- 13)
```

std::collections hashmap vs cubby::keys (entity manager)
```
running 4 tests
test hmap_add_bm                      ... bench:         237 ns/iter (+/- 27)
test hmap_iter_bm                     ... bench:      15,995 ns/iter (+/- 746)
test hmap_rt_bm                       ... bench:         165 ns/iter (+/- 4)
test hmap_work_bm                     ... bench:     377,134 ns/iter (+/- 11,684)

running 4 tests
test keys_add_bm                      ... bench:          43 ns/iter (+/- 5)
test keys_iter_bm                     ... bench:          24 ns/iter (+/- 0)
test keys_rt_bm                       ... bench:          37 ns/iter (+/- 3)
test keys_work_bm                     ... bench:      71,592 ns/iter (+/- 5,572)
```

Keys example
```
    let mut mgr = NodeManager::new(MAX_NODES);
    
    let key = mgr.add(Node::new(true));
    assert!(mgr.get(key).is_some());
    
    assert!(mgr.get(key).unwrap().data); //value is true
    
    mgr.remove(key);
    assert!(!mgr.get(key).is_some());
```