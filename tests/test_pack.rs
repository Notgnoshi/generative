use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_pack_four_squares() {
    let input = b"\
        POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))\n\
        POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))\n\
        POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))\n\
        POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))\n\
    ";

    let expected = "\
        POLYGON((-0.5 -0.5,-0.5 0.5,0.5 0.5,0.5 -0.5,-0.5 -0.5))\n\
        POLYGON((1.5 -0.5,1.5 0.5,2.5 0.5,2.5 -0.5,1.5 -0.5))\n\
        POLYGON((-0.5 1.5,-0.5 2.5,0.5 2.5,0.5 1.5,-0.5 1.5))\n\
        POLYGON((1.5 1.5,1.5 2.5,2.5 2.5,2.5 1.5,1.5 1.5))\n\
    ";

    let output = tool("pack")
        .arg("--width=4")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}
