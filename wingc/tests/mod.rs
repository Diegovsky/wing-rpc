use wingc::utils::{Mode, test};

#[test]
fn simple() {
    test("simple", Mode::Test);
}

#[test]
fn composite() {
    test("composite", Mode::Test);
}
