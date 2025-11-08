use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_smooth_unit_square() {
    let input = b"POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))\n";

    let expected = "\
        POLYGON((0 0.25,0 0.75,0.25 1,0.75 1,1 0.75,1 0.25,0.75 0,0.25 0,0 0.25))\n\
    ";

    let output = tool("smooth")
        .arg("--iterations=1")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}
