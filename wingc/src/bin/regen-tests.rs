use std::path::Path;

use wingc::utils::*;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::env::set_current_dir(root).unwrap();
    let tests_dir = Path::new("test-files/");
    let mut tests: Vec<String> = vec!["use wingc::utils::{test, Mode};".into()];
    for entry in std::fs::read_dir(tests_dir).unwrap() {
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
        test(name.as_str(), Mode::Emit);
        tests.push(format!(
            r#"
#[test]
fn {}(){{
    test("{name}", Mode::Test);
}}
"#,
            name.replace("-", "_")
        ))
    }
    std::fs::write(root.join("tests/mod.rs"), tests.join("\n")).unwrap();
    println!("Done regenerating tests!")
}
