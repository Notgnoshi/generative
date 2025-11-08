use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_geom2graph_round_trip() {
    // Two squares, one inside the other, sharing the (1, 1) corner. Results in seven vertices and
    // eight edges.
    let input = b"\
        POLYGON ((0 0,     0 1,   1 1, 1 0,   0 0))\n\
        POLYGON ((0.5 0.5, 0.5 1, 1 1, 1 0.5, 0.5 0.5))\n\
    ";

    let expected = "\
        0\tPOINT(0 0)\n\
        1\tPOINT(0 1)\n\
        2\tPOINT(0.5 1)\n\
        3\tPOINT(1 1)\n\
        4\tPOINT(1 0.5)\n\
        5\tPOINT(1 0)\n\
        6\tPOINT(0.5 0.5)\n\
        #\n\
        0\t5\n\
        0\t1\n\
        1\t2\n\
        2\t6\n\
        2\t3\n\
        3\t4\n\
        4\t6\n\
        4\t5\n\
    ";

    let output = tool("geom2graph").write_stdin(input).captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);

    // Feed the output back in.
    let output = tool("geom2graph")
        .arg("--graph2geom")
        .write_stdin(expected)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    let expected = "\
        POLYGON((0 0,0 1,0.5 1,0.5 0.5,1 0.5,1 0,0 0))\n\
        POLYGON((0.5 1,1 1,1 0.5,0.5 0.5,0.5 1))\n\
    ";
    assert_eq!(expected, stdout);
}
