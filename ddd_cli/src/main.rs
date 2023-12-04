extern crate ambassador;
use std::any::Any;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};
use ambassador::{delegatable_trait, Delegate};

#[delegatable_trait]
trait DslItem {
    fn name_set(&mut self, item: &'static str);
    fn name_get(&self) -> &str;
    fn namespace_set(&mut self, item: &'static str);
    fn namespace_get(&self) -> &str;
    fn me_set(&mut self, item: Rc<RefCell<dyn DslItem>>);
    fn me_get(&self) -> Option<Rc<RefCell<dyn DslItem>>>;
    fn parent_set(&mut self, item: Rc<RefCell<dyn DslItem>>);
    fn parent_get(&self) -> Option<Rc<RefCell<dyn DslItem>>>;
    fn child_put(&mut self, name: &str, value: Rc<RefCell<dyn DslItem>>);
    fn child_get(&self, name: &str) -> Option<Rc<RefCell<dyn DslItem>>>;
    fn as_any(&self) -> &dyn Any;
}

struct Item {
    name: Option<&'static str>,
    namespace: Option<&'static str>,
    me: Option<Rc<RefCell<dyn DslItem>>>,
    parent: Option<Rc<RefCell<dyn DslItem>>>,
    children: Option<HashMap<String,Rc<RefCell<dyn DslItem>>>>,
}

impl DslItem for Item {
    fn name_set(&mut self, value: &'static str) {
        self.name = Some(value);
    }

    fn name_get(&self) -> &str {
        self.name.unwrap()
    }
    
    fn namespace_set(&mut self, value: &'static str) {
        self.namespace = Some(value);
    }

    fn namespace_get(&self) -> &str {
        self.namespace.unwrap()
    }

    fn me_set(&mut self, value: Rc<RefCell<dyn DslItem>>) {
        self.me = Some(value);
    }
 
    fn me_get(&self) -> Option<Rc<RefCell<dyn DslItem>>> {
         self.me.clone()
    }

    fn parent_set(&mut self, value: Rc<RefCell<dyn DslItem>>) {
       self.parent = Some(value);
    }

    fn parent_get(&self) -> Option<Rc<RefCell<dyn DslItem>>> {
        self.parent.clone()
    }

    fn child_put(&mut self, name: &str, value: Rc<RefCell<dyn DslItem>>) {
        value.borrow_mut().parent_set(self.me_get().unwrap());
        match &mut self.children {
            Some(children) => {
                children.insert(name.to_string(), value);
            },
            None => {
                self.children = Some(HashMap::new());
                self.children.as_mut().unwrap().insert(name.to_string(), value);
            }
        }
    }

    fn child_get(&self, name: &str) -> Option<Rc<RefCell<dyn DslItem>>> {
        match &self.children {
            Some(children) => {
                if let Some(v) = children.get(name) {
                    Some(v.clone())
                } else {
                    None
                }
            },
            None => {
                None
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl RcRefCellItem
    for Rc<RefCell<dyn DslItem>>
{
    fn ancestor (
        self: &'_ Rc<RefCell<dyn DslItem>>,
        distance: usize,
    ) -> Rc<RefCell<dyn DslItem>>
    {
        if distance == 0 {
            Rc::clone(self)
        } else {
            if let Some(parent) = &self.borrow().parent_get() {
                parent
                    .ancestor(distance - 1)
            } else {
                todo!() // handle errors later
            }
        }
    }
}
trait RcRefCellItem {
    fn ancestor (
        self: &'_ Self,
        distance: usize,
    ) -> Self
    ;
}

#[derive(Delegate)]
#[delegate(DslItem, target = "item")]
struct DynamicItem {
    item: Item,
    values: HashMap<String,Rc<RefCell<dyn DslItem>>>,
}

#[derive(Delegate)]
#[delegate(DslItem, target = "item")]
struct Type {
    item: Item,
    ty: Option<String>,
}

fn main ()
{
    let root = wrap_as_rc_item(Item {
        name: Some("root"),
        namespace: None,
        me: None,
        parent: None,
        children: None,
    });

    let child =  wrap_as_rc_item(Item {
        name: Some("child"),
        namespace: None,
        me: None,
        parent: None,
        children: None,
    });

    let child2 = wrap_as_rc_item(
        DynamicItem {
            item: Item {
                name: Some("child2"),
                namespace: None,
                me: None,
                parent: None,
                children: None,
            },
            values: HashMap::new(),
        });

    root.borrow_mut().child_put("child", child.clone());
    child.borrow_mut().child_put("child2", child2.clone());

    let binding = child2.borrow();
    let di = binding.as_any().downcast_ref::<DynamicItem>().expect("failed to downcast");
    
    let n = di.item.name.unwrap();

    println!("{:?}", child.borrow().parent_get().unwrap().borrow().name_get());
    println!("{:?}", child2.borrow().parent_get().unwrap().borrow().name_get());
    println!("{:?}", n);
    

    /*
    root.borrow_mut().put("foo", 1);
    child.borrow_mut().put("bar", 2);
    child2.borrow_mut().put("bar", 3);

    assert_eq!(root.borrow().get("foo").unwrap(), 1);
    assert_eq!(child.borrow().get("bar").unwrap(), 2);
    assert_eq!(child2.borrow().get("bar").unwrap(), 3);

    root.borrow_mut().put("new", 2);
    
    assert_eq!(child.ancestor(1).borrow().get("new").unwrap(), 2);
    */
}

fn wrap_as_rc_item<T: DslItem + 'static>(value: T) -> Rc<RefCell<dyn DslItem>> {
    let root = Rc::new(RefCell::new(value));
    root.borrow_mut().me_set(root.clone());
    root
}