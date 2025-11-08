use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_triangulate_each_geometry() {
    let input = b"\
        POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))\n\
        POLYGON ((2 0, 2 1, 3 1, 3 0, 2 0))\n\
    ";

    let expected = "\
        LINESTRING(1 1,1 0)\n\
        LINESTRING(1 0,0 0)\n\
        LINESTRING(0 0,0 1)\n\
        LINESTRING(0 1,1 1)\n\
        LINESTRING(0 0,1 1)\n\
        LINESTRING(3 1,3 0)\n\
        LINESTRING(3 0,2 0)\n\
        LINESTRING(2 0,2 1)\n\
        LINESTRING(2 1,3 1)\n\
        LINESTRING(2 0,3 1)\n\
    ";

    let output = tool("triangulate")
        .arg("--strategy=each-geometry")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_triangulate_whole_collection() {
    let input = b"\
        POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))\n\
        POLYGON ((2 0, 2 1, 3 1, 3 0, 2 0))\n\
    ";

    // Includes the gap between the two polygons
    let expected = "\
        LINESTRING(3 1,3 0)\n\
        LINESTRING(3 0,2 0)\n\
        LINESTRING(2 0,1 0)\n\
        LINESTRING(1 0,0 0)\n\
        LINESTRING(0 0,0 1)\n\
        LINESTRING(0 1,1 1)\n\
        LINESTRING(1 1,2 1)\n\
        LINESTRING(2 1,3 1)\n\
        LINESTRING(0 0,1 1)\n\
        LINESTRING(1 0,1 1)\n\
        LINESTRING(2 0,1 1)\n\
        LINESTRING(2 0,2 1)\n\
        LINESTRING(2 0,3 1)\n\
    ";

    let output = tool("triangulate")
        .arg("--strategy=whole-collection")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_triangulate_graph() {
    let input = b"\
        POLYGON ((0 0, 0 1, 1 1, 1 0, 0 0))\n\
    ";

    let expected = "\
        0\tPOINT(0 0)\n\
        1\tPOINT(0 1)\n\
        2\tPOINT(1 1)\n\
        3\tPOINT(1 0)\n\
        #\n\
        2\t3\n\
        3\t0\n\
        0\t1\n\
        1\t2\n\
        0\t2\n\
    ";

    let output = tool("triangulate")
        .arg("--strategy=whole-collection")
        .arg("--output-format=tgf")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}
