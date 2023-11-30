extern crate ddd_derives;

use ddd_derives::AsDslItem;

#[test]
fn dsl_item() {

    #[derive(AsDslItem)]
    struct Command {
        executable: String,
        current_dir: String,
    }

    let command = DslCommand(|o| {
        o.executable("cargo".to_owned()).current_dir(".".to_owned());
    });
    

    assert_eq!(command.executable_get(), "cargo");

}