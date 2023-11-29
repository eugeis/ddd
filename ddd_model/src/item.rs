use ddd_derives::AsDslItem;

#[derive(AsDslItem)]
struct Item {
    name: String,
    namespace: String,
    desc: String,
    internal: bool,
    derived_as_type: String,
    initialized: bool,
    parent: Box<dyn DslItem>,
    //derived_from: Box<dyn DslItem>,
    //derived_items: Vec<Box<dyn DslItem>>,
}

#[derive(AsDslItem)]
struct Attribute {
    nullable: bool,
    multi: bool,
    name_non_fluent: String,
}
