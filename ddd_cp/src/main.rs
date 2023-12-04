extern crate serde;
extern crate serde_yaml;

use std::rc::{Rc, Weak};
use std::cell::RefCell;

use std::fs::File;
use std::io::BufWriter;
use serde::{Serialize, Deserialize};

#[typetag::serde(tag = "type")]
pub trait Item {
    fn name(&self) -> String;
    fn namespace(&self) -> String;
}

#[derive(Serialize, Deserialize)]
pub struct Node {
    item: Box<dyn Item>,
    #[serde(skip_serializing)]
    me: Option<Weak<RefCell<Node>>>,
    #[serde(skip_serializing)]
    parent: Option<Weak<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(item: Box<dyn Item>) -> Rc<RefCell<Self>> {
        let node = Rc::new(RefCell::new(Node {
            item,
            me: None,
            parent: None,
            children: Vec::new(),
        }));

        node.borrow_mut().me = Some(Rc::downgrade(&node));

        node
    }

    fn add_child(&mut self, item: Box<dyn Item>) -> Rc<RefCell<Node>> {
        let child = Node::new(item);
        child.borrow_mut().parent = self.me.clone();
        self.children.push(Rc::clone(&child));
        child
    }

    fn traverse_up<F, P>(&self, on_node: F, stop_predicate: P)
    where
        F: Fn(&Node),
        P: Fn(&Node) -> bool,
    {
        if stop_predicate(self) {
            return;
        }

        on_node(self);

        if let Some(parent_weak) = &self.parent {
            if let Some(parent) = parent_weak.upgrade() {
                parent.borrow().traverse_up(on_node, stop_predicate);
            }
        }
    }

    fn traverse_down<F, P>(&self, on_node: &F, stop_predicate: &P)
    where
        F: Fn(&Node),
        P: Fn(&Node) -> bool,
    {
        if stop_predicate(self) {
            return;
        }

        on_node(self);

        for child in &self.children {
            let child = child.borrow();
            child.traverse_down(on_node, stop_predicate);
        }
    }

    fn find_parent(&self, condition: impl Fn(&dyn Item) -> bool) -> Option<Weak<RefCell<Node>>> {
        let mut current = self.parent.clone();
        while let Some(node) = current.clone() {
            let node = node.upgrade().unwrap();
            if condition(node.borrow().item.as_ref()) {
                return current;
            }
            current = node.borrow().parent.clone();
        }
        None
    }

    fn find_child<F>(&self, predicate: &F) -> Option<Rc<RefCell<Node>>>
    where
        F: Fn(&Node) -> bool,
    {
        for child_rc in &self.children {
            let child = child_rc.borrow();
            if predicate(&*child) {
                return Some(Rc::clone(child_rc));
            }

            if let Some(found) = child.find_child(predicate) {
                return Some(found);
            }
        }

        None
    }

    fn filter_and_collect(&self, predicate: impl Fn(&dyn Item) -> bool) -> Vec<Rc<RefCell<Node>>> {
        self.children.iter()
            .filter(|node| predicate(node.borrow().item.as_ref()))
            .cloned()
            .collect()
    }

    fn serialize_to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(&self)
    }

    fn write_to_yaml_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut serializer = serde_yaml::Serializer::new(writer);
        self.serialize(&mut serializer)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct SimpleItem {
    name: String,
    namespace: String,
}

#[typetag::serde]
impl Item for SimpleItem {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn namespace(&self) -> String {
        self.namespace.clone()
    }
}

fn create_tree(depth: usize, breadth: usize) -> Rc<RefCell<Node>> {
    let node = Rc::new(RefCell::new(Node {
        item: Box::new(SimpleItem { name: format!("Node 0"), namespace: format!("Namespace 0") }),
        me: None,
        parent: None,
        children: Vec::new(),
    }));

    create_children(Rc::clone(&node), depth, breadth, 1);

    node
}

fn create_children(parent: Rc<RefCell<Node>>, depth: usize, breadth: usize, current_depth: usize) {
    if current_depth > depth {
        return;
    }

    for i in 0..breadth {
        let child = parent.borrow_mut().add_child(
            Box::new(SimpleItem { name: format!("Node {}", current_depth * breadth + i), namespace: format!("Namespace {}", current_depth * breadth + i) })
        );
        create_children(child, depth, breadth, current_depth + 1);
    }
}

fn main() {
    let root = create_tree(3, 3);
    /*
    root.borrow().traverse_up(
        |node| println!("Visiting node: {:?}", node.item.name()),
        |node| node.item.name() == "Stop Node"
    );
    let _ = root.borrow().find_parent(|item| item.name() == "Node 0");
    let _ = root.borrow().filter_and_collect(|item| item.name().contains("Node"));

     */

    root.borrow().write_to_yaml_file("test.yaml").unwrap();
}