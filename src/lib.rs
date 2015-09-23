#![feature(test)]
#![feature(drain)]

extern crate rand;
use std::marker::PhantomData;

pub struct Cubby<C,V,T> {
    ents: Box<[C]>,
    dead: V,
    _pd: PhantomData<T>,
}

impl<C:BackendC<T>,V:BackendV,T:Send+Sync> Cubby<C,V,T> {
    pub fn new<F: Fn(Ent<T>) -> C, F2: Fn(Vec<usize>) -> V> 
        (f:F,f2:F2, s:usize) -> Cubby<C,V,T> {
        let mut c = vec!();
        for _ in (0..s) { 
            let r = f((0,None));
            c.push(r);
        }

        let mut v = vec!();
        for n in (0..s).rev() {
            v.push(n);
        }

        Cubby { ents: c.into_boxed_slice(),
                dead: f2(v),
                _pd: PhantomData }
    }

    pub fn with<W, F: Fn(&T) -> W> (&self, e: Eid, f: F) -> Result<W,EntErr> {
        self.ents[e.0]
            .with(|r|
                  if r.0 == e.1 {
                      if let Some(ref r) = r.1 {
                          Ok(f(r))
                      }
                      else { Err(EntErr::NoData) }
                  }
                  else { Err(EntErr::Invalid) }
                  ).unwrap()
    }

    pub fn with_mut<W, F: FnOnce(&mut T) -> W> (&self, e: Eid, f: F) -> Result<W,EntErr> {
        self.ents[e.0].
            with_mut(|mut w|
                     if w.0 == e.1 {
                         if let Some(ref mut w) = w.1 {
                             Ok(f(w))
                         }
                         else { Err(EntErr::NoData) }
                     }
                     else { Err(EntErr::Invalid) }
                     ).unwrap()
    }

    pub fn add (&self, i: T) -> Result<Eid,EntErr> {
        let d = try!(self.dead.with_mut(|mut v| v.pop()));
        
        if let Some(idx) = d {
            let rid = rand::random::<u64>();
            self.ents[idx]
                .with_mut(|mut w|
                          *w = (rid,Some(i)));
            Ok((idx,rid))
        }
        else { Err(EntErr::Maxed) }
    }

    pub fn remove (&self, e:Eid) -> bool {
        self.ents[e.0]
            .with_mut(|mut w|
                      if w.0 == e.1 {
                          w.0 = 0;
                          if self.dead.with_mut(|mut v| v.push(e.0)).is_ok() {
                              true
                          }
                          else{false}
                      }
                      else { false }
                      ).unwrap()
    }

    pub fn find<F: FnMut(&T) -> bool> (&self, mut f: F) -> Vec<Eid> {
        let mut v = vec!();
        let mut b = false;
        for (i,e) in self.ents.iter().enumerate() {
            e.with(|rl|
                   if rl.0 > 0 {
                       if let Some(ref r) = rl.1 {
                           if f(r) { v.push((i,rl.0)); }
                       }
                       else { b=true; } //quit at first None
                   });
            if b {break}
        }
        v
    }

    pub fn first<F: Fn(&T) -> bool> (&self, f: F) -> Option<Eid> {
        let mut b = false;
        let mut rv = None;
        for (i,e) in self.ents.iter().enumerate() {
            e.with(|r|
                   if r.0 > 0 {
                       if let Some(ref v) = r.1 {
                           if f(v) { rv = Some((i,r.0));b=true; }
                       }
                       else { b=true; } //quit at first None
                   });

            if b {break;}
        }
            rv
    }

    // todo: consider bool, like a 'while' filter
    // also consider impl collection style iterator
    pub fn each<F: FnMut(&T) -> Option<EntErr>> (&self, mut f: F) {
        let mut b = false;
        for e in self.ents.iter() {
            e.with(|rl|
                   if rl.0 > 0 {
                       if let Some(ref r) = rl.1 {
                           if let Some(r) = f(r) {
                               match r {
                                   EntErr::Break => {b=true;}, //escape hatch
                                   _ => (),
                               }
                           }
                       }
                       else { b=true; } //quit at first None
                   });
            if b { break; }
        }
    }

    pub fn each_mut<F: FnMut(&mut T) -> Option<EntErr>> (&self, mut f: F) {
        let mut b = false;
        for e in self.ents.iter() {
            e.with_mut(|wl|
                       if wl.0 > 0 {
                           if let &mut Some(ref mut w) = &mut wl.1 {
                               if let Some(r) = f(w) {
                                   match r {
                                       EntErr::Break => {b=true;}, //escape hatch
                                       _ => (),
                                   }
                               }
                           }
                           else {b=true;}
                       });
            if b {break;}
        }
    }

    
}

#[derive(Debug)]
pub enum EntErr {
    NoData,
    Invalid,
    Maxed,
    Break, //break from an 'each' call
}

pub type Eid = (usize,u64);
pub type Ent<T> = (u64,Option<T>);

pub trait BackendC<T> {
    fn with<W,F: FnMut(&Ent<T>) -> W> (&self,F) -> Result<W,EntErr>; 
    fn with_mut<W,F: FnOnce(&mut Ent<T>) -> W> (&self,F) -> Result<W,EntErr>; //&mut Ent<T>;
}
pub trait BackendV {
    fn with_mut<W,F: FnOnce(&mut Vec<usize>) -> W> (&self,F) -> Result<W,EntErr>;
}


pub mod rwlock;
pub mod mutex;
mod tests;
pub mod keys;

pub use keys::{Node,NodeManager};

use std::collections::HashMap;
use rand::random;

const MAX_NODES: usize = 2000;

extern crate test;
#[test]
fn keys_smoke() {
    let mut mgr = NodeManager::new(MAX_NODES);
    
    let key = mgr.add(Node::new(true));
    assert!(mgr.get(key).is_some());
    
    mgr.remove(key);
    assert!(!mgr.get(key).is_some());
}
#[bench]
fn keys_add_bm(b: &mut test::Bencher) {
    let mut mgr = NodeManager::new(MAX_NODES);
    
    b.iter(|| {
        mgr.add(Node::new(true));
    });
}

#[bench]
fn hmap_add_bm(b: &mut test::Bencher) {
    let mut mgr = HashMap::new();
    
    b.iter(|| {
        mgr.insert(random::<u32>(),true);
    });
}
#[bench]
fn keys_rt_bm(b: &mut test::Bencher) {
    let mut mgr = NodeManager::new(MAX_NODES);
    
    b.iter(|| {
        let key = mgr.add(Node::new(true));
        mgr.remove(key);
    });
}

#[bench]
fn hmap_rt_bm(b: &mut test::Bencher) {
    let mut mgr = HashMap::new();
    
    b.iter(|| {
        let key = random::<u32>();
        mgr.insert(key,true);
        mgr.remove(&key);
    });
}
#[bench]
fn keys_iter_bm(b: &mut test::Bencher) {
    let mut mgr = NodeManager::new(MAX_NODES);
    for n in (0..MAX_NODES) { mgr.add(Node::new(true)); }
    
    b.iter(|| {
        for n in mgr.box_iter() {
            n;
        }
    });
}

#[bench]
fn hmap_iter_bm(b: &mut test::Bencher) {
    let mut mgr = HashMap::new();
    for n in (0..MAX_NODES) {  mgr.insert(random::<u32>(),
                                     true); }
    
    b.iter(|| {
        for n in mgr.iter() {
            n;
        }
    });
}

#[bench]
fn keys_work_bm(b: &mut test::Bencher) {
    let mut mgr = NodeManager::new(MAX_NODES);
    let mut keys = vec!();
    
    b.iter(|| {
        for n in (0..MAX_NODES) {
            let key = mgr.add(Node::new(true));
            keys.push(key);
        }
        for n in mgr.box_iter() {
            n;
        }
        for key in keys.drain(..) {  mgr.remove(key); }
    });
}

#[bench]
fn hmap_work_bm(b: &mut test::Bencher) {
    let mut mgr = HashMap::new();
    let mut keys = vec!();
    
    b.iter(|| {
        for n in (0..MAX_NODES) {
            let key = random::<u32>();
            mgr.insert(key,true);
            keys.push(key);
        }
        for n in mgr.iter() {
            n;
        }
        for key in keys.drain(..) {  mgr.remove(&key); }
    });
}
