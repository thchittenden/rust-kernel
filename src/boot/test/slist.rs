use alloc::boxed::Box;
use collections::link::{SingleLink, HasSingleLink};
use collections::slist::SList;
logger_init!(Trace);

#[derive(Debug)]
struct Node {
    item: usize,
    name: &'static str,
    node: SingleLink<Node>
}

impl HasSingleLink<Node> for Node {
    fn slink(&self) -> &SingleLink<Node> {
        &self.node
    }
    fn slink_mut(&mut self) -> &mut SingleLink<Node> {
        &mut self.node
    }
}

#[inline(never)]
pub fn test() {
    trace!("\ntesting slist");

    let mut list = SList::new();
    let n1 = Box::new(Node { item: 5, name: "n1", node: SingleLink::new() }).unwrap();
    list.push(n1);
    let n2 = Box::new(Node { item: 6, name: "n2", node: SingleLink::new() }).unwrap();
    list.push(n2);
    let n3 = Box::new(Node { item: 7, name: "n3", node: SingleLink::new() }).unwrap();
    list.push(n3);
    trace!("count: {}", list.len());
    assert!(list.len() == 3);
    for elem in &list {
        trace!("node: {:?}", elem);
    }
    
    list.remove_where(|elem| elem.item == 6);
    assert!(list.len() == 2);
    trace!("count: {}", list.len());
    for elem in &list {
        trace!("node: {:?}", elem);
    }

    list.remove_where(|elem| elem.item == 7);
    assert!(list.len() == 1);
    trace!("count: {}", list.len());
    for elem in &list {
        trace!("node: {:?}", elem);
    }

    list.pop();
    assert!(list.len() == 0);
    trace!("count: {}", list.len());
    for elem in &list {
        trace!("node: {:?}", elem);
    }


}
