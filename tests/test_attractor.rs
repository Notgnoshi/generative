use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_attractor_simple() {
    let expected = "\
        POINT(1 2)\n\
        POINT(2 4)\n\
        POINT(3 6)\n\
    ";

    let output = tool("attractor")
        .arg("--initial-x=0")
        .arg("--initial-y=0")
        .arg("--math=let x_new = x + 1.0;")
        .arg("--math=let y_new = y + 2.0;")
        .arg("--iterations=3")
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}
