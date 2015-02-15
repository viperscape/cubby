use std::rand;
use std::sync::{Arc,Mutex};


pub struct Ent<T: Send> {
    ents: Box<[Mutex<(u64,Option<T>)>]>,
    dead: Mutex<Vec<usize>>,
}

#[derive(Debug)]
pub enum EntErr {
    NoData,
    Invalid,
    Maxed,
}

pub type Eid = (usize,u64);

impl<T: Send> Ent<T> {
    pub fn new (s:usize) -> Ent<T> {
        let mut v = vec!();
        let mut d = vec!();
        for n in range(0,s) {
            v.push(Mutex::new((0,None)));
        }

        // do in reverse
        for n in range(0,s).rev() {
            d.push(n);
        }

        let e = Ent { ents: v.into_boxed_slice(),
                      dead: Mutex::new(d), };

        e
    }

    pub fn add (&self, i: T) -> Result<Eid,EntErr> {
        let d = self.dead.lock().unwrap().pop();
        if let Some(idx) = d {
            let rid = rand::random::<u64>();
            let mut wl = self.ents[idx].lock().unwrap();
            *wl = (rid,Some(i));
            Ok((idx,rid))
        }
        else { Err(EntErr::Maxed) }
    }

    pub fn remove (&self, e:Eid) -> bool {
        let mut wl = self.ents[e.0].lock().unwrap();
        if wl.0 == e.1 {
            wl.0 = 0;
            self.dead.lock().unwrap().push(e.0);
            true
        }
        else { false }
    }

    pub fn with<W, F: Fn(&T) -> W> (&self, e: Eid, f: F) -> Result<W,EntErr> {
        let rl = self.ents[e.0].lock().unwrap();
        if rl.0 == e.1 {
            if let Some(ref r) = rl.1 {
                Ok(f(r))
            }
            else { Err(EntErr::NoData) }
        }
        else { Err(EntErr::Invalid) }
    }

    pub fn with_mut<W, F: Fn(&mut T) -> W> (&self, e: Eid, f: F) -> Result<W,EntErr> {
        let mut wl = self.ents[e.0].lock().unwrap();
        if wl.0 == e.1 {
            if let &mut Some(ref mut w) = &mut wl.1 {
                Ok(f(w))
            }
            else { Err(EntErr::NoData) }
        }
        else { Err(EntErr::Invalid) }
    }

    pub fn each<W, F: Fn(&T) -> W> (&self, f: F) {
        for e in self.ents.iter() {
            let rl = e.lock().unwrap();
            if rl.0 > 0 {
                if let Some(ref r) = rl.1 {
                    f(r);
                }
                else { break; }
            }
        }
    }

    pub fn each_mut<W, F: Fn(&mut T) -> W> (&self, f: F) {
        for e in self.ents.iter() {
            let mut wl = e.lock().unwrap();
            if wl.0 > 0 {
                if let &mut Some(ref mut w) = &mut wl.1 {
                    f(w);
                }
                else { break; }
            }
        }
    }

    //pub fn iter (&self) -> &[Mutex<(u64,Option<T>)>] {
    //    self.ents.as_slice()
    //}
}



#[cfg(test)]
mod tests {
    extern crate test;
    use Ent;

    use std::collections::HashMap;
    use std::sync::mpsc::{channel};
    use std::thread::Thread;

    use std::sync::{Arc,Mutex};
    use std::rand;
    use std::old_io::timer::sleep;
    use std::time::Duration;


    #[test]
    fn test_cubby() {
        let mut e: Ent<u8> = Ent::new(10);
        let rid = e.add(2).unwrap();
        e.remove(rid);
        assert_eq!(e.with(rid,|i| *i),Err("ent removed"));
    }

    #[bench]
    fn bench_cubby(b: &mut test::Bencher) {
        let mut e: Ent<u8> = Ent::new(10);
        
        b.iter(|| {
            let rid = e.add(2).unwrap();
            e.remove(rid);
        });
    }

    #[bench]
    fn bench_cubby_thread(b: &mut test::Bencher) {
        let mut e: Ent<u8> = Ent::new(100);
        let e = Arc::new(e);
        let iters = 10;

        b.iter(|| {
            let (t,r) = channel();
            
            for n in range(0,iters) {
                let e2 = e.clone();
                let t2 = t.clone();

                Thread::spawn(move || {
                    let rid = e2.add(2).unwrap();
                    e2.with(rid,|i| 
                           sleep(Duration::milliseconds(10)));
                    e2.remove(rid);
                    t2.send(true);
                });
            }


            for n in range(0,iters) {
                r.recv();
            }
        });
    }


    #[bench]
    fn bench_hmap(b: &mut test::Bencher) {
        let mut e: HashMap<u64,u8> = HashMap::new();
        
        b.iter(|| {
            let rid = rand::random::<u64>();
            e.insert(rid,2);
            e.remove(&rid);
        });
    }


    #[bench]
    fn bench_hmap_thread(b: &mut test::Bencher) {
        let mut e: HashMap<u64,u8> = HashMap::new();
        let e = Arc::new(Mutex::new(e));
        let iters = 10;

        b.iter(|| {
            let (t,r) = channel();
            
            for n in range(0,iters) {
                let e2 = e.clone();
                let t2 = t.clone();
                Thread::spawn(move || {
                    let rid = rand::random::<u64>();
                    {let mut wl = e2.lock().unwrap();
                     wl.insert(rid,2);}


                    {let rl = e2.lock().unwrap();
                     let i = rl.get(&rid);
                     sleep(Duration::milliseconds(10));}

                    {let mut wl = e2.lock().unwrap();
                     wl.remove(&rid);}

                    t2.send(true);
                });
            }

            for n in range(0,iters) {
                r.recv();
            }
        });
    }
}
