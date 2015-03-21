use super::{Ent,BackendC,BackendV,Cubby,EntErr};
use std::sync::{Mutex};

impl<'a,T:Send+Sync> BackendC<T> for Mutex<Ent<T>> {
    fn with<W, F:FnMut(&Ent<T>)->W> (&self,mut f:F) -> Result<W,EntErr> { 
        let v = (*self).lock();
        match v {
            Ok(_) => Ok(f(&*v.unwrap())), //mutexguard is funny in a match, unwrap v instead
            Err(_) => Err(EntErr::Invalid),
        }
    }

    fn with_mut<W, F:FnOnce(&mut Ent<T>)->W> (&self, f:F) -> Result<W,EntErr> { //&mut Ent<T> {
        let v = (*self).lock();
        match v {
            Ok(_) => Ok(f(&mut *v.unwrap())), 
            Err(_) => Err(EntErr::Invalid),
        }
    }
}

impl BackendV for Mutex<Vec<usize>> {
    fn with_mut<W, F:FnOnce(&mut Vec<usize>)->W> (&self, f:F) -> Result<W,EntErr> {
        let v = (*self).lock();
        match v {
            Ok(_) => Ok(f(&mut *v.unwrap())), 
            Err(_) => Err(EntErr::Invalid),
        }
    }
}

pub fn build<T:Send+Sync> (s:usize) -> CubbyMutex<T> {
    Cubby::new(|n| Mutex::new(n),
               |v| Mutex::new(v),
               s)
}

pub type CubbyMutex<T> = Cubby<Mutex<Ent<T>>,Mutex<Vec<usize>>,T>;
