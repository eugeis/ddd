extern crate ddd_derives;

use ddd_derives::AsDslItem;

#[test]
fn dsl_item() {
    #[derive(AsDslItem)]
    pub struct Command {
        executable: String,
        args: Vec<String>,
        env: Vec<String>,
        current_dir: Option<String>,
    }

    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        //.current_dir(Some("..".to_owned()))
        .build()
        .unwrap();

    assert_eq!(command.executable, "cargo");
}