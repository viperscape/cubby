#![feature(collections)]
use std::marker::{PhantomData,PhantomFn};
extern crate rand;

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
        let d = self.dead.get_mut(); 
        if let Some(idx) = d.pop() {
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
                          self.dead.get_mut().push(e.0);
                          true
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
        for (i,e) in self.ents.iter().enumerate() {
            let r = e.with(|r|
                           if r.0 > 0 {
                               if let Some(ref v) = r.1 {
                                   if f(v) { Ok(Some((i,r.0))) }
                                   else { Ok(None) }
                               }
                               else { Err(()) } //quit at first None
                           }
                           else { Ok(None) } ).ok();
            if r.is_some() { if r.unwrap().ok().is_some() { return r.unwrap().unwrap() } }
            else { break; }
        }
        None
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

pub trait BackendC<T>: PhantomFn<T> {
    fn with<W,F: FnMut(&Ent<T>) -> W> (&self,F) -> Result<W,EntErr>; 
    fn with_mut<W,F: FnOnce(&mut Ent<T>) -> W> (&self,F) -> Result<W,EntErr>; //&mut Ent<T>;
}
pub trait BackendV {
    fn get_mut(&self) -> &mut Vec<usize>;
}


pub mod rwlock;
pub mod mutex;
