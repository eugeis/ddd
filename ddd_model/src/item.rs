use ddd_derives::AsDslItem;

#[derive(AsDslItem)]
struct Item {
    name: String,
    namespace: String,
    desc: String,
    //parent: Box<dyn DslItem>,
    internal: bool,
    //derived_from: Box<dyn DslItem>,
    //derived_items: Vec<Box<dyn DslItem>>,
    derived_as_type: String,
    initialized: bool,
}

#[derive(AsDslItem)]
struct Attribute {
    nullable: bool,
    multi: bool,
    name_non_fluent: String,
}
