use wingc::utils::{Mode, test};

#[test]
fn simple() {
    test("test-files/", "simple", Mode::Test);
}

#[test]
fn composite() {
    test("test-files/", "composite", Mode::Test);
}
