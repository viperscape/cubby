#![feature(collections)]
use std::marker::{PhantomData};
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

    pub fn get(&self, e:Eid) -> Result<&T,EntErr> {
        let r = self.ents[e.0].get();
        if r.0 == e.1 {
            if let Some(ref r) = r.1 {
                Ok(r)
            }
            else { Err(EntErr::NoData) }
        }
        else { Err(EntErr::Invalid) }
    }

    pub fn get_mut(&self, e:Eid) -> Result<&mut T,EntErr> {
        let mut w = self.ents[e.0].get_mut();
        if w.0 == e.1 {
            if let Some(ref mut w) = w.1 {
                Ok(w)
            }
            else { Err(EntErr::NoData) }
        }
        else { Err(EntErr::Invalid) }
    }

    pub fn add (&self, i: T) -> Result<Eid,EntErr> {
        let d = self.dead.get_mut(); 
        if let Some(idx) = d.pop() {
            let rid = rand::random::<u64>();
            let mut w = self.ents[idx].get_mut();
            *w = (rid,Some(i));
            Ok((idx,rid))
        }
        else { Err(EntErr::Maxed) }
    }

    pub fn remove (&self, e:Eid) -> bool {
        let mut w = self.ents[e.0].get_mut();
        if w.0 == e.1 {
            w.0 = 0;
            self.dead.get_mut().push(e.0);
            true
        }
        else { false }
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
    fn get(&self) -> &Ent<T>;
    fn get_mut(&self) -> &mut Ent<T>;
}
pub trait BackendV {
    fn get_mut(&self) -> &mut Vec<usize>;
}

pub mod rwlock;
pub mod mutex;
