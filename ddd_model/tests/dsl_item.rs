extern crate ddd_derives;

use ddd_derives::AsDslItem;

#[test]
fn dsl_item() {

    #[derive(AsDslItem)]
    struct Command {
        executable: String,
        current_dir: String,
        b: bool,
    }

    let command = dslCommand(|o| {
        o.executable("cargo").current_dir(".").b(false);
    });
    

    assert_eq!(command.executable_get(), "cargo");

}