use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_graph_output_formats() {
    let expected = "\
        0\tPOINT(0 0)\n\
        1\tPOINT(1 0)\n\
        2\tPOINT(2 0)\n\
        3\tPOINT(0 1)\n\
        4\tPOINT(1 1)\n\
        5\tPOINT(2 1)\n\
        6\tPOINT(0 2)\n\
        7\tPOINT(1 2)\n\
        8\tPOINT(2 2)\n\
        #\n\
        0\t3\n\
        0\t1\n\
        1\t4\n\
        1\t2\n\
        2\t5\n\
        3\t6\n\
        3\t4\n\
        4\t7\n\
        4\t5\n\
        5\t8\n\
        6\t7\n\
        7\t8\n\
    ";

    let output = tool("grid")
        .arg("--grid-type=quad")
        .arg("--output-format=graph")
        .arg("--width=2")
        .arg("--height=2")
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_lines_output_formats() {
    let expected = "\
        LINESTRING(0 0,0 1)\n\
        LINESTRING(0 0,1 0)\n\
        LINESTRING(1 0,1 1)\n\
        LINESTRING(1 0,2 0)\n\
        LINESTRING(2 0,2 1)\n\
        LINESTRING(0 1,0 2)\n\
        LINESTRING(0 1,1 1)\n\
        LINESTRING(1 1,1 2)\n\
        LINESTRING(1 1,2 1)\n\
        LINESTRING(2 1,2 2)\n\
        LINESTRING(0 2,1 2)\n\
        LINESTRING(1 2,2 2)\n\
    ";

    let output = tool("grid")
        .arg("--grid-type=quad")
        .arg("--output-format=lines")
        .arg("--width=2")
        .arg("--height=2")
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_points_output_formats() {
    let expected = "\
        POINT(0 0)\n\
        POINT(1 0)\n\
        POINT(2 0)\n\
        POINT(0 1)\n\
        POINT(1 1)\n\
        POINT(2 1)\n\
        POINT(0 2)\n\
        POINT(1 2)\n\
        POINT(2 2)\n\
    ";

    let output = tool("grid")
        .arg("--grid-type=quad")
        .arg("--output-format=points")
        .arg("--width=2")
        .arg("--height=2")
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_cells_output_formats() {
    let expected = "\
        POLYGON((1 0,0 0,0 1,1 1,1 0))\n\
        POLYGON((2 0,1 0,1 1,2 1,2 0))\n\
        POLYGON((1 1,0 1,0 2,1 2,1 1))\n\
        POLYGON((2 1,1 1,1 2,2 2,2 1))\n\
    ";

    let output = tool("grid")
        .arg("--grid-type=quad")
        .arg("--output-format=cells")
        .arg("--width=2")
        .arg("--height=2")
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_triangle() {
    let expected = "\
        POINT(0 0)\n\
        POINT(1 0)\n\
        POINT(2 0)\n\
        POINT(0 1)\n\
        POINT(1 1)\n\
        POINT(2 1)\n\
        POINT(0 2)\n\
        POINT(1 2)\n\
        POINT(2 2)\n\
    ";

    let output = tool("grid")
        .arg("--grid-type=quad")
        .arg("--output-format=points")
        .arg("--width=2")
        .arg("--height=2")
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_quad() {
    let expected = "\
        POINT(0 0)\n\
        POINT(1 0)\n\
        POINT(2 0)\n\
        POINT(0 1)\n\
        POINT(1 1)\n\
        POINT(2 1)\n\
        POINT(0 2)\n\
        POINT(1 2)\n\
        POINT(2 2)\n\
    ";

    let output = tool("grid")
        .arg("--grid-type=quad")
        .arg("--output-format=points")
        .arg("--width=2")
        .arg("--height=2")
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_ragged() {
    let expected = "\
        POINT(0 0)\n\
        POINT(1 0)\n\
        POINT(2 0)\n\
        POINT(3 0)\n\
        POINT(0 1)\n\
        POINT(1 1)\n\
        POINT(2 1)\n\
        POINT(3 1)\n\
        POINT(0 2)\n\
        POINT(1 2)\n\
        POINT(2 2)\n\
        POINT(3 2)\n\
    ";

    let output = tool("grid")
        .arg("--grid-type=ragged")
        .arg("--output-format=points")
        .arg("--width=2")
        .arg("--height=2")
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_hexagon() {
    let expected = "\
        POINT(0 0)\n\
        POINT(1 0)\n\
        POINT(1.5 4.330127018922193)\n\
        POINT(-0.5 0.8660254037844386)\n\
        POINT(1.5 0.8660254037844386)\n\
        POINT(2.5 0.8660254037844386)\n\
        POINT(0 1.7320508075688772)\n\
        POINT(1 1.7320508075688772)\n\
        POINT(3 1.7320508075688772)\n\
        POINT(-0.5 2.598076211353316)\n\
        POINT(1.5 2.598076211353316)\n\
        POINT(2.5 2.598076211353316)\n\
        POINT(0 3.4641016151377544)\n\
        POINT(1 3.4641016151377544)\n\
        POINT(3 3.4641016151377544)\n\
        POINT(2.5 4.330127018922193)\n\
    ";

    let output = tool("grid")
        .arg("--grid-type=hexagon")
        .arg("--output-format=points")
        .arg("--width=2")
        .arg("--height=2")
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_radial() {
    let expected = "\
        POINT(1 0)\n\
        POINT(0.766044443118978 0.6427876096865393)\n\
        POINT(0.17364817766693041 0.984807753012208)\n\
        POINT(-0.4999999999999998 0.8660254037844387)\n\
        POINT(-0.9396926207859083 0.3420201433256689)\n\
        POINT(-0.9396926207859085 -0.3420201433256682)\n\
        POINT(-0.5000000000000004 -0.8660254037844384)\n\
        POINT(0.17364817766692997 -0.9848077530122081)\n\
        POINT(0.7660444431189778 -0.6427876096865396)\n\
        POINT(1 0)\n\
        POINT(2 0)\n\
        POINT(1.532088886237956 1.2855752193730785)\n\
        POINT(0.34729635533386083 1.969615506024416)\n\
        POINT(-0.9999999999999996 1.7320508075688774)\n\
        POINT(-1.8793852415718166 0.6840402866513378)\n\
        POINT(-1.879385241571817 -0.6840402866513364)\n\
        POINT(-1.0000000000000009 -1.7320508075688767)\n\
        POINT(0.34729635533385994 -1.9696155060244163)\n\
        POINT(1.5320888862379556 -1.2855752193730792)\n\
        POINT(2 0)\n\
        POINT(0 0)\n\
        POINT(1 0)\n\
        POINT(2 0)\n\
        POINT(0 0)\n\
        POINT(-0.4999999999999998 0.8660254037844387)\n\
        POINT(-0.9999999999999996 1.7320508075688774)\n\
        POINT(0 0)\n\
        POINT(-0.5000000000000004 -0.8660254037844384)\n\
        POINT(-1.0000000000000009 -1.7320508075688767)\n\
    ";

    let output = tool("grid")
        .arg("--grid-type=radial")
        .arg("--output-format=points")
        .arg("--ring-fill-points=2")
        .arg("--width=3") // angular division
        .arg("--height=2") // radius
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}
