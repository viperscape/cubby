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
    fn with_mut<W, F:FnOnce(&mut Vec<usize>)->W> (&self, f:F) -> Result<W,EntErr> {
        let v = (*self).write();
        match v {
            Ok(_) => Ok(f(&mut *v.unwrap())), 
            Err(_) => Err(EntErr::Invalid),
        }
    }
}

pub fn build<T:Send+Sync> (s:usize) -> CubbyRwLock<T> {
    Cubby::new(|n| RwLock::new(n),
               |v| RwLock::new(v),
               s)
}

pub type CubbyRwLock<T> = Cubby<RwLock<Ent<T>>,RwLock<Vec<usize>>,T>;
