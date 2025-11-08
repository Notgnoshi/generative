use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_bundle_simple_points() {
    let input = b"\
        POINT (0 0)\n\
        POINT (1 1)\n\
        POINT (2 2)\n\
    ";

    let expected = "GEOMETRYCOLLECTION(POINT(0 0),POINT(1 1),POINT(2 2))\n";

    let output = tool("bundle").write_stdin(input).captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}
