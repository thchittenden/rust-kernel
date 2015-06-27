use alloc::boxed::Box;
use collections::link::{SingleLink, HasSingleLink};
use collections::hashmap::{HashMap, HasKey};
use collections::string::String;
logger_init!(Trace);

#[derive(Debug)]
struct Node {
    key: String,
    val: usize,
    link: SingleLink<Node>,
}

impl Node {
    fn new(key: String, val: usize) -> Node {
        Node { key: key, val: val, link: SingleLink::new() }   
    }
}

impl HasSingleLink<Node> for Node {
    fn slink(&self) -> &SingleLink<Node> {
        &self.link
    }
    fn slink_mut(&mut self) -> &mut SingleLink<Node> {
        &mut self.link
    }
}

impl HasKey<String> for Node {
    fn get_key(&self) -> &String {
        &self.key
    }
}

pub fn test() {
    trace!("\ntesting hashmap");
    let mut map = HashMap::new().unwrap();
    trace!("creating n1");
    let node = Node::new(String::from_str("n1"), 1001);
    trace!("boxing n1");
    let n1 = Box::new(node).unwrap();
    trace!("creating n2");
    let n2 = Box::new(Node::new(String::from_str("n2"), 2002)).unwrap();
    trace!("creating n3");
    let n3 = Box::new(Node::new(String::from_str("n3"), 3003)).unwrap();
    
    trace!("inserting n1: {:?}", n1);
    map.insert(n1);
    trace!("inserting n2: {:?}", n2);
    map.insert(n2);
    trace!("inserting n3: {:?}", n3);
    map.insert(n3);

    trace!("map.count() == {}", map.count());
    for key in map.iter_keys() {
        let val = map.lookup(key).unwrap();
        trace!("key: {}, val: {:?}", key, val);
    }

    trace!("map 2");
    let mut map = HashMap::new().unwrap();
    let n1 = Box::new(Node::new(String::from_str("dev"), 100)).unwrap();
    map.insert(n1);
    trace!("count: {}", map.count());
    for key in map.iter_keys() {
        trace!("key: {}", key);
    }



    

}
