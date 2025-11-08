use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_snap_graph() {
    // A few connected nodes close together, and a few close together nodes that aren't connected.
    let input = b"\
        0\tPOINT(0 0)\n\
        2\tPOINT(0 0.05)\n\
        3\tPOINT(1 1)\n\
        4\tPOINT(1.05 1)\n\
        #\n\
        0\t2\n\
        0\t3\n\
    ";

    let expected = "\
        0\tPOINT(1.05 1)\n\
        1\tPOINT(0 0.05)\n\
        #\n\
        0\t1\n\
    ";

    let output = tool("snap")
        .arg("--input-format=tgf")
        .arg("--tolerance=0.1")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_snap_graph_duplicate_nodes() {
    let input = b"\
        1\tPOINT(0 0)\n\
        2\tPOINT(0 0)\n\
        #\n\
    ";

    let expected = "\
        0\tPOINT(0 0)\n\
        #\n\
    ";

    let output = tool("snap")
        .arg("--input-format=tgf")
        .arg("--tolerance=0.1")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_snap_graph_to_grid() {
    let input = b"\
        0\tPOINT(0 0)\n\
        1\tPOINT(0 0)\n\
        2\tPOINT(0 0.05)\n\
        3\tPOINT(1 1)\n\
        4\tPOINT(1.05 1)\n\
        #\n\
        0\t2\n\
        0\t3\n\
    ";

    let expected = "\
        0\tPOINT(0 0)\n\
        1\tPOINT(1 1)\n\
        #\n\
        0\t1\n\
    ";

    let output = tool("snap")
        .arg("--input-format=tgf")
        .arg("--tolerance=0.5")
        .arg("--strategy=regular-grid") // regular grid with --tolerance spacing
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_snap_geoms() {
    let input = b"\
        POLYGON((0 0, 0 0.99, 0.99 0.99, 0.99 0, 0 0))\n\
        POLYGON((0 0, 0 1, 1 1, 1 0, 0 0))\n\
    ";

    let expected = "\
        POLYGON((0 0,0 1,1 1,1 0,0 0))\n\
        POLYGON((0 0,0 1,1 1,1 0,0 0))\n\
    ";

    let output = tool("snap")
        .arg("--input-format=wkt")
        .arg("--tolerance=0.1")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_snap_geoms_to_grid() {
    // Just a single point (no other points to snap to)
    let input = b"\
        POINT(0.1 0.1)\n\
    ";

    // Still gets snapped to the 0.5 grid
    let expected = "\
        POINT(0 0)\n\
    ";

    let output = tool("snap")
        .arg("--input-format=wkt")
        .arg("--tolerance=0.5")
        .arg("--strategy=regular-grid") // regular grid with --tolerance spacing
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}
