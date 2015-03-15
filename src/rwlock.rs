use super::{Ent,EntErr,BackendC,BackendV,Cubby};
use std::sync::{RwLock};
use std::mem;

impl<T:Send+Sync> BackendC<T> for RwLock<Ent<T>> {
    fn with<W, F:FnMut(&Ent<T>)->W> (&self,mut f:F) -> Result<W,EntErr> { 
        let v = (*self).read();
        match v {
            Ok(_) => Ok(f(&*v.unwrap())), //mutexguard is funny in a match, unwrap v instead
            Err(_) => Err(EntErr::Invalid),
        }
    }

    fn with_mut<W, F:FnOnce(&mut Ent<T>)->W> (&self, f:F) -> Result<W,EntErr> {
        let v = (*self).write();
        match v {
            Ok(_) => Ok(f(&mut *v.unwrap())), 
            Err(_) => Err(EntErr::Invalid),
        }
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
