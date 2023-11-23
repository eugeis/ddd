use ddd_derives::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: Option<String>,
}

fn b() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        //.current_dir("..".to_owned())
        .build()
        .unwrap();

    assert_eq!(command.executable, "cargo");
}
