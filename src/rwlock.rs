use super::{Ent,BackendC,BackendV,Cubby};
use std::sync::{RwLock};
use std::mem;

impl<T:Send+Sync> BackendC<T> for RwLock<Ent<T>> {
    fn get (&self) -> &Ent<T> {
        let v = (*self).read();
        unsafe { mem::transmute(&*v.unwrap()) }
    }

    fn get_mut (&self) -> &mut Ent<T> {
        let v = (*self).write();
        unsafe { mem::transmute(&mut *v.unwrap()) }
    }
}

impl BackendV for RwLock<Vec<usize>> {
    fn get_mut (&self) -> &mut Vec<usize> {
        let v = (*self).write();
        unsafe { mem::transmute(&mut *v.unwrap()) }
    }
}

pub fn build<T:Send+Sync> (s:usize) -> Cubby<RwLock<Ent<T>>,RwLock<Vec<usize>>,T> {
    Cubby::new(|n| RwLock::new(n),
               |v| RwLock::new(v),
               s)
}
