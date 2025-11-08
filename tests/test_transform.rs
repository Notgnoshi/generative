use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_rotate_about_origin() {
    let input = b"POINT(1 0)\n";
    let expected = "POINT(-1 0.00000000000000012246467991473532)\n";

    let output = tool("transform")
        .arg("--center=origin")
        .arg("--rotation=180")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_rotate_about_geom_bbox_center() {
    let input = b"\
        LINESTRING(-1 0, 1 0)\n\
        LINESTRING(-1 1, 1 1)\n\
    ";
    let expected = "\
        LINESTRING(-0.00000000000000006123233995736766 -1,0.00000000000000006123233995736766 1)\n\
        LINESTRING(0 0,0.00000000000000011102230246251565 2)\n\
    ";
    let output = tool("transform")
        .arg("--center=each-geometry")
        .arg("--rotation=90")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_rotate_about_collection_center() {
    let input = b"\
        LINESTRING(-1 0, 1 0)\n\
        LINESTRING(-1 1, 1 1)\n\
    ";
    let expected = "\
        LINESTRING(0.49999999999999994 -0.5,0.5000000000000001 1.5)\n\
        LINESTRING(-0.5 -0.49999999999999994,-0.4999999999999999 1.5)\n\
    ";
    let output = tool("transform")
        .arg("--center=whole-collection")
        .arg("--rotation=90")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_scale() {
    let input = b"\
        POINT(0 0)\n\
        POINT(1 0)\n\
        POINT(0 2)\n\
    ";
    let expected = "\
        POINT(0 0)\n\
        POINT(2 0)\n\
        POINT(0 4)\n\
    ";
    let output = tool("transform")
        .arg("--scale=2")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_offset() {
    let input = b"\
        POINT(0 0)\n\
    ";
    let expected = "\
        POINT(1 -1)\n\
    ";
    let output = tool("transform")
        .arg("--offset-x=1")
        .arg("--offset-y=-1")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_skew_x() {
    let input = b"\
        LINESTRING(0 0, 0 1)\n\
    ";
    let expected = "\
        LINESTRING(0 0,0.9999999999999999 1)\n\
    ";
    let output = tool("transform")
        .arg("--skew-x=45")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_polar() {
    let input = b"\
        LINESTRING(0 0, 0 1)\n\
    ";
    let expected = "\
        LINESTRING(0 0,1 1.5707963267948966)\n\
    ";
    let output = tool("transform")
        .arg("--to-polar")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn fit_to_range() {
    let input = b"\
        POINT(0 0)\n\
        POINT(2 0)\n\
        POINT(4 0)\n\
    ";
    let expected = "\
        POINT(-4 0)\n\
        POINT(-3 0)\n\
        POINT(-2 0)\n\
    ";
    let output = tool("transform")
        .arg("--offset-x=-1") // -1, 1, -3
        .arg("--range1=-4,-2") // -4, -3, -2
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}
