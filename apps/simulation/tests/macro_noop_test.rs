#![cfg(not(feature = "dev-tools"))]
#![allow(unused_variables)]

use speciate::time_system;

#[test]
fn test_macro_compiles_to_nothing_without_dev_tools() {
    let dummy = ();
    time_system!(dummy, "movement");
    assert!(true);
}

#[test]
fn test_macro_accepts_any_first_arg_without_dev_tools() {
    let x = 42;
    time_system!(x, "perception");
    time_system!("literal", "behavior");
    time_system!((1, 2, 3), "anything");
}
