# L-System Tools

I envision each stage of the L-System generation process to have its own Python script.
I want each script to take input from stdin, and write plaintext output to stdout in a well-defined format.
This will allow inserting new tools in the middle of a pipeline quite easily.

These scripts will likely need the `sys.path.append` hack.

Scripts:

1. Apply some number of iterations to an axiom with a given set of rules. Take JSON input file from stdin/arg.
2. Consume the LSystem string with a turtle/interpreter to come up with a set of line segments, thicknesses, colors.
    1. What kind of transformations can be applied to the segments? Perturbation, random pruning, color application after the fact?
    2. Simplify any overlapping line segments. What to do with metadata? Average thickness/color? Weighted by length?
    3. Should output the line length so it can be used by an intermediate transformation? Use HTML colors?
3. Render the line segments in:
    1. SVG
    2. OpenGL
4. Stitch together per-frame screenshots to form animations. (Bash + ffmpeg)
5. Plot SVGs with AxiDraw.
