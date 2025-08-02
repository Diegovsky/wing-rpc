use wingc::utils::{test, Mode};

#[test]
fn simple(){
    test("simple", Mode::Test);
}


#[test]
fn nested(){
    test("nested", Mode::Test);
}


#[test]
fn composite_simple(){
    test("composite-simple", Mode::Test);
}


#[test]
fn composite(){
    test("composite", Mode::Test);
}
