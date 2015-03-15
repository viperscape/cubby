use super::{Ent,BackendC,BackendV,Cubby,EntErr};
use std::sync::{Mutex};
use std::mem;

impl<'a,T:Send+Sync> BackendC<T> for Mutex<Ent<T>> {
    fn with<W, F:Fn(&Ent<T>)->W> (&self,f:F) -> Result<W,EntErr> { 
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

//todo: remove me, replace with channels or at least safeness
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
