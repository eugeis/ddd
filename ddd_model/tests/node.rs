#[test]
fn node() {
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