use pretty_assertions::assert_eq;

use crate::{CommandExt, tool};

#[test]
fn test_simple_geometries() {
    let input = b"\
        POINT (0 0)\n\
        LINESTRING (1 1, 2 2)\n\
        POLYGON ((0 0, 1 0, 1 1, 0 1, 0 0))\n\
    ";

    let expected = "\
        <svg viewBox=\"-12 -12 26 26\" xmlns=\"http://www.w3.org/2000/svg\">\n\
        <style type=\"text/css\">\n\
        svg { stroke:black; stroke-width:2; fill:none; transform:scale(1,-1);}\n\
        </style>\n\
        <circle cx=\"-9\" cy=\"-9\" r=\"1\"/>\n\
        <polyline points=\"1 1 11 11\"/>\n\
        <path d=\"M-9,-9 L1,-9 L1,1 L-9,1 z\" fill-rule=\"evenodd\"/>\n\
        </svg>\
    ";

    let output = tool("wkt2svg")
        .arg("--scale=10")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_cli_styles() {
    let input = b"\
        POINT (0 0)\n\
        LINESTRING (1 1, 2 2)\n\
        POLYGON ((0 0, 1 0, 1 1, 0 1, 0 0))\n\
    ";

    let expected = "\
        <svg viewBox=\"-12 -12 26 26\" xmlns=\"http://www.w3.org/2000/svg\">\n\
        <style type=\"text/css\">\n\
        svg {stroke-dasharray:5; stroke:red; stroke-width:2; fill:blue; transform:scale(1,-1);}\n\
        </style>\n\
        <circle cx=\"-9\" cy=\"-9\" r=\"0.5\"/>\n\
        <polyline points=\"1 1 11 11\"/>\n\
        <path d=\"M-9,-9 L1,-9 L1,1 L-9,1 z\" fill-rule=\"evenodd\"/>\n\
        </svg>\
    ";

    let output = tool("wkt2svg")
        .arg("--scale=10")
        .arg("--point-radius=0.5")
        .arg("--stroke=red")
        .arg("--stroke-dasharray=5")
        .arg("--fill=blue")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}

#[test]
fn test_wkt_styles() {
    let input = b"\
        POINT(0 0)\n\
        POINT(100 100)\n\
        STROKEWIDTH(4)\n\
        STROKEDASHARRAY(6 1)\n\
        POINTRADIUS(20)\n\
        FILL(red)\n\
        POINT(50 50)\n\
    ";

    let expected = "\
        <svg viewBox=\"-453 -453 1006 1006\" xmlns=\"http://www.w3.org/2000/svg\">\n\
        <style type=\"text/css\">\n\
        svg { stroke:black; stroke-width:2; fill:none; transform:scale(1,-1);}\n\
        </style>\n\
        <circle cx=\"-450\" cy=\"-450\" r=\"1\"/>\n\
        <circle cx=\"550\" cy=\"550\" r=\"1\"/>\n\
        <circle cx=\"50\" cy=\"50\" fill=\"red\" r=\"20\" stroke-dasharray=\"6 1\" stroke-width=\"4\"/>\n\
        </svg>\
    ";

    let output = tool("wkt2svg")
        .arg("--scale=10")
        .write_stdin(input)
        .captured_output();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(expected, stdout);
}
