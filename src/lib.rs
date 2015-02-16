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
    Break, //break from an 'each' call
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

    pub fn with<W, F: FnMut(&T) -> W> (&self, e: Eid, mut f: F) -> Result<W,EntErr> {
        let rl = self.ents[e.0].lock().unwrap();
        if rl.0 == e.1 {
            if let Some(ref r) = rl.1 {
                Ok(f(r))
            }
            else { Err(EntErr::NoData) }
        }
        else { Err(EntErr::Invalid) }
    }

    // note: this is fnonce to solve a capture issue I had, this may not be the best option, perhaps a second method for fnonce specific captures? (with_cap)?
    pub fn with_mut<W, F: FnOnce(&mut T) -> W> (&self, e: Eid, mut f: F) -> Result<W,EntErr> {
        let mut wl = self.ents[e.0].lock().unwrap();
        if wl.0 == e.1 {
            if let &mut Some(ref mut w) = &mut wl.1 {
                Ok(f(w))
            }
            else { Err(EntErr::NoData) }
        }
        else { Err(EntErr::Invalid) }
    }

    pub fn each<F: FnMut(&T) -> Option<EntErr>> (&self, mut f: F) {
        for e in self.ents.iter() {
            let rl = e.lock().unwrap();
            if rl.0 > 0 {
                if let Some(ref r) = rl.1 {
                    if let Some(r) = f(r) {
                        match r {
                            EntErr::Break => {break;}, //escape hatch
                            _ => (),
                        }
                    }
                }
                else { break; } //quit at first None
            }
        }
    }

    pub fn each_mut<F: FnMut(&mut T) -> Option<EntErr>> (&self, mut f: F) {
        for e in self.ents.iter() {
            let mut wl = e.lock().unwrap();
            if wl.0 > 0 {
                if let &mut Some(ref mut w) = &mut wl.1 {
                    if let Some(r) = f(w) {
                        match r {
                            EntErr::Break => {break;}, //escape hatch
                            _ => (),
                        }
                    }
                }
                else { break; }
            }
        }
    }

    pub fn find<F: FnMut(&T) -> Option<EntErr>> (&self, mut f: F) -> Option<Eid> {
        for (i,e) in self.ents.iter().enumerate() {
            let rl = e.lock().unwrap();
            if rl.0 > 0 {
                if let Some(ref r) = rl.1 {
                    if let Some(r) = f(r) {
                        match r {
                            EntErr::Break => {return Some((i,rl.0));}, //escape hatch
                            _ => (),
                        }
                    }
                }
                else { break; } //quit at first None
            }
        }
        None
    }

    //pub fn iter (&self) -> &[Mutex<(u64,Option<T>)>] {
    //    self.ents.as_slice()
    //}
}



#[cfg(test)]
mod tests {
    extern crate test;
    use Ent;
    use EntErr;

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
        assert_eq!(e.with(rid,|i| *i).unwrap(),2);
    }

    #[test]
    #[should_fail]
    fn test_cubby_remove() {
        let mut e: Ent<u8> = Ent::new(10);
        let rid = e.add(2).unwrap();
        e.remove(rid);

        let r = e.with(rid,|i| *i).unwrap();
        assert_eq!(r,2);
    }

    #[test]
    fn test_cubby_mut() {
        let mut e: Ent<u8> = Ent::new(10);
        let rid = e.add(2).unwrap();
        e.with_mut(rid,|i| *i+=1);
        let r = e.with_mut(rid,|i| *i).unwrap();
        assert_eq!(r,3);
    }

    #[test]
    fn test_cubby_each() {
        let mut e: Ent<u8> = Ent::new(10);
        e.add(2);
        e.add(3);
        let mut v = vec!();
        e.each(|i| {v.push(*i); None});
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_cubby_find() {
        let mut e: Ent<u8> = Ent::new(10);
        e.add(2);
        e.add(3);
        let r = e.find(|i| { if *i == 3 { Some(EntErr::Break) }
                             else {None} });
        assert_eq!(r.is_some(),true);
    }

//

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
