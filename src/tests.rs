#[cfg(test)]
mod tests {
    extern crate test;
    extern crate rand;
    use {rwlock,mutex,Ent,EntErr,Cubby,BackendC,BackendV};

    use std::collections::HashMap;
    use std::sync::mpsc::{channel};
    use std::thread::Thread;

    use std::sync::{Arc,Mutex,RwLock};
    use std::thread::sleep_ms as sleep;


    #[test]
    fn test_cubby() {
        let mut c = rwlock::build(10);
        let e = c.add(2).unwrap();
        assert_eq!(c.with(e,|i| *i).unwrap(),2);
    }

    #[test]
    #[should_panic]
    fn test_cubby_remove() {
        let mut c = rwlock::build(10);
        let e = c.add(2).unwrap();
        c.remove(e);

        let r = c.with(e,|i| *i).unwrap();
        assert_eq!(r,2);
    }

    #[test]
    fn test_cubby_mut() {
        let mut c = rwlock::build(10);
        let e = c.add(2).unwrap();
        c.with_mut(e,|i| *i+=1);
        let r = c.with_mut(e,|i| *i).unwrap();
        assert_eq!(r,3);
    }

    #[test]
    fn test_cubby_each() {
        let mut c = rwlock::build(10);
        c.add(2);
        c.add(3);
        let mut v = vec!();
        c.each(|i| {v.push(*i); None});
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_cubby_find() {
        let mut c = rwlock::build(10);
        c.add(2);
        c.add(3);
        let r = c.find(|i| { *i > 0 });
        assert!(r.len() > 1);
    }

    #[test]
    fn test_cubby_first() {
        let mut c = rwlock::build(10);
        c.add(2);
        let odd = c.add(3).unwrap();
        let r = c.first(|i| *i % 2 == 1);
        assert!(r.is_some());
        assert_eq!(odd,r.unwrap());
    }


    // benches

    #[bench]
    fn bench_cubby_rwlock(b: &mut test::Bencher) {
        let mut c = rwlock::build(1);
        
        b.iter(|| {
            let e = c.add(2).unwrap();
            c.first(|n| *n>0);
            c.with_mut(e,|n| *n+=1);
            c.remove(e);
        });
    }
    #[bench]
    fn bench_cubby_mutex(b: &mut test::Bencher) {
        let mut c = mutex::build(1);
        
        b.iter(|| {
            let e = c.add(2).unwrap();
            c.first(|n| *n>0);
            c.with_mut(e,|n| *n+=1);
            c.remove(e);
        });
    }

    #[bench]
    fn bench_hmap_mutex(b: &mut test::Bencher) {
        let mut c = Mutex::new(HashMap::new());
        
        b.iter(|| {
            let rid = rand::random::<u64>();
            c.lock().unwrap().insert(rid,2);

            {let mut wl = c.lock().unwrap();
             let mut r = None;
             for (k,v) in wl.iter() {
                 if *v>0 {r=Some(k);break;}
             }
            }

            {let mut wl = c.lock().unwrap();
             let mut w = wl.get_mut(&rid).unwrap();
             *w += 1;}

            c.lock().unwrap().remove(&rid);
        });
    }
}
