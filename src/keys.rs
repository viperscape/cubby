extern crate rand;

use rand::random;

pub type NodeId = u32;
pub type KeySize = usize; // I'd prefer u32 instead
pub type NodeKey = (KeySize,NodeId); // must match for valid key

pub trait Id {
    fn nid(&self) -> NodeId;
    fn idx(&self) -> KeySize; // entity idx
}
impl Id for NodeKey {
    fn nid(&self) -> NodeId { self.1 }
    fn idx(&self) -> KeySize { self.0 }
}



#[derive(Clone)]
pub struct Node<T> {
    id: Option<NodeId>, // none represents a dead-node
    parent: Option<NodeKey>,
    pub data: T,
}

impl<T> Node<T> {
    pub fn new (d: T) -> Node<T> {
        let id = random::<u32>();
        Node { 
            id: Some(id),
            parent: None,
            data: d,
        }
    }
    pub fn id(&self) -> &Option<NodeId> { &self.id }
    pub fn destroy(&mut self) { self.id = None; }
}


pub struct NodeManager<T> {
    nodes: Vec<Node<T>>,
    dead: Vec<KeySize>,
}

impl<T> NodeManager<T> {
    pub fn new (max: usize) -> NodeManager<T> {
        let v = Vec::with_capacity(max);
        NodeManager { nodes: v,
                      dead: vec!(), }
    }
    
    pub fn add(&mut self, n: Node<T>) -> NodeKey {
        let nid = n.id().unwrap();
        let idx;
        
        if let Some(_idx) = self.dead.pop() {
            idx = _idx;
            self.nodes[idx] = n;
        }
        else {
            idx = self.nodes.len();
            self.nodes.push(n);
        }

        (idx,nid)
    }

    pub fn remove(&mut self, key: NodeKey) -> bool {
        let node = &mut self.nodes[key.idx()];
        if let &Some(id) = node.id() {
            if id==key.nid() {
                node.destroy();
                self.dead.push(key.idx());
                return true;
            }
        }

        false
    }

    pub fn get(&self, key: NodeKey) -> Option<&Node<T>> {
        let node = &self.nodes[key.idx()];
        if let &Some(id) = node.id() {
            if id==key.nid() { return Some(node) }
        }

        None
    }

    pub fn get_mut(&mut self, key: NodeKey) -> Option<&mut Node<T>> {
        let node = &mut self.nodes[key.idx()];
        if let &Some(id) = node.id() {
            if id==key.nid() { return Some(node) }
        }

        None
    }

    pub fn set_parent(&mut self, key: NodeKey, pkey: NodeKey) {
        if let Some(node) = self.get_mut(key) {
            node.parent = Some(pkey);
        }
    }

    /// gets the node's parent node
    pub fn get_parent(&mut self, key: NodeKey) -> Option<&Node<T>> {
        if let Some(node) = self.get(key) {
            if let Some(pkey) = node.parent {
                return self.get(pkey);
            }
        }

        None
    }

    /// gets the node's parent key
    pub fn get_pkey(&mut self, key: NodeKey) -> Option<NodeKey> {
        if let Some(node) = self.get(key) {
            return node.parent
        }

        None
    }

 /*   /// update scene, only updates parent nodes once
    pub fn update(&mut self, dt: &f64) {
        let mut frame: HashSet<NodeKey> = HashSet::new();
        let mut keys = vec!(); 
        for (i,n) in self.nodes.iter().enumerate() {
            if let &Some(id) = n.id() {
                keys.push((i,id));
            }
        }

        for key in keys {
            if frame.contains(&key) { continue }
            self.updater(key,dt,&mut frame);
        }
    }*/

    pub fn box_iter<'a> (&'a self) -> Box<Iterator<Item=&'a Node<T>> + 'a> {
        Box::new(self.nodes.iter().filter(|n| n.id().is_some()))
    }
}
