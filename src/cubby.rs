
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
