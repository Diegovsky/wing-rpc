use std::path::Path;

use wingc::utils::*;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = root.join("test-files/");
    let tests_dir_relative_to_test = Path::new("test-files/");
    let mut tests: Vec<String> = vec!["use wingc::utils::{test, Mode};".into()];
    for entry in std::fs::read_dir(tests_dir.as_path()).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        if !name.ends_with(".wing") {
            continue;
        }
        let name = entry
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        test(tests_dir.as_path(), name.as_str(), Mode::Emit);
        tests.push(format!(
            r#"
#[test]
fn {name}(){{
    test({tests_dir_relative_to_test:?}, "{name}", Mode::Test);
}}
"#
        ))
    }
    std::fs::write(root.join("tests/mod.rs"), tests.join("\n")).unwrap();
}
