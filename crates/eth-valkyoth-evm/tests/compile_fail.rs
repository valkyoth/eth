//! Compile-fail coverage for execution admission typestates.

#[test]
fn execution_admission_typestates_are_not_interchangeable() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/classified_envelope_request.rs");
}
