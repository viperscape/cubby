#![feature(test)]
#![feature(drain)]

extern crate rand;

pub mod rwlock;
pub mod mutex;
mod tests;
pub mod keys;
pub mod cubby;

pub use keys::{Node,NodeManager};
pub use cubby::{Ent,EntErr,BackendC,BackendV,Cubby};

use std::collections::HashMap;
use rand::random;

const MAX_NODES: usize = 2000;

extern crate test;
#[test]
fn keys_smoke() {
    let mut mgr = NodeManager::new(MAX_NODES);
    
    let key = mgr.add(Node::new(true));
    assert!(mgr.get(key).is_some());
    
    assert!(mgr.get(key).unwrap().data); //value is true
    
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
