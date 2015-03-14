use super::{Ent,BackendC,BackendV,Cubby};
use std::sync::{Mutex};
use std::mem;

impl<T:Send+Sync> BackendC<T> for Mutex<Ent<T>> {
    fn get (&self) -> &Ent<T> {
        let v = (*self).lock();
        unsafe { mem::transmute(&*v.unwrap()) }
    }

    fn get_mut (&self) -> &mut Ent<T> {
        let v = (*self).lock();
        unsafe { mem::transmute(&mut *v.unwrap()) }
    }
}

impl BackendV for Mutex<Vec<usize>> {
    fn get_mut (&self) -> &mut Vec<usize> {
        let v = (*self).lock();
        unsafe { mem::transmute(&mut *v.unwrap()) }
    }
}

pub fn build<T:Send+Sync> (s:usize) -> Cubby<Mutex<Ent<T>>,Mutex<Vec<usize>>,T> {
    Cubby::new(|n| Mutex::new(n),
               |v| Mutex::new(v),
               s)
}
