use wingc::utils::{test, Mode};

#[test]
fn simple(){
    test("test-files/", "simple", Mode::Test);
}


#[test]
fn composite(){
    test("test-files/", "composite", Mode::Test);
}
