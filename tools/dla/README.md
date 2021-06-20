# Off-Lattice Diffusion Limited Aggregation

Diffusion Limited Aggregation is a generative algorithm where particles are added to a model one-by-one and take a random walk until they run into another particle.
After attaching to another particle, their location is fixed.
Off-lattice means that each particle's coordinates are continuous -- they are not fixed to a lattice.

With appropriate tunable parameters, this forms an organic tree reminiscent of blood vessels or root systems.

```shell
$ cargo run --release --
        --seed 461266331856721221 \
        --seeds 2 \
        --attraction-distance 10 \
        --min-move-distance 1 \
        --stubbornness 10 \
        --particle-spacing 0.1 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 20 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/organic.svg
```

![organic tree](/examples/diffusion-limited-aggregation/organic.svg)

# Usage

## How to build

DLA is a Rust project, so it's really quite simple.
Note that you really want to use the optimized release build, as the default debug build is _significantly_ slower.

```shell
cargo build --release
```

## How to run

```shell
cargo run --release -- <args here>
```

Because of the CWD constraints on the `cargo run` invocation, it's helpful to specify the path to the binary when combining this tool with others.

```shell
cd <path/to/repository/root>
./tools/dla/target/release/dla \
    --seeds=20 \
    --min-move-distance=1 \
    --attraction-distance=20 \
    --particle-spacing=0.1 \
    --particles 50000 |
./tools/geom2graph/build/src/geom2graph --graph2geom |
./tools/render.py
```

# Options

There are many tunable parameters that you can pass to the DLA tool in the form of commandline arguments.

For best results, you'll want to combine different tunable parameters.

```
$ cargo run --release -- --help
dla 0.1.0
Off-lattice diffusion limited aggregation

USAGE:
    dla [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Silence all logging
    -V, --version    Prints version information
    -v, --verbose    Increase logging verbosity

OPTIONS:
    -a, --attraction-distance <attraction-distance>
            Distance threshold for joining together two particles [default: 3]

    -d, --dimensions <dimensions>
            Dimensionality of the particles [default: 2]
    -f, --format <format>
            Output format. Either "tgf" graph format or "points" point cloud [default: tgf]

    -m, --min-move-distance <min-move-distance>        Minimum move distance for random walk [default: 1]
    -o, --output <output>                              Output file to write result to. Defaults to stdout
        --particle-spacing <particle-spacing>          Spacing between joined together particles [default: 1]
    -p, --particles <particles>                        Number of particles to add [default: 10000]
        --seed <seed>                                  The random seed to use, for reproducibility [default: -1]
        --seeds <seeds>
            Number of seed particles. If one seed particle is used, it will be placed at the origin. Otherwise, the seed
            particles will be uniformly spread around the origin [default: 1]
        --stickiness <stickiness>
            Defines the probability that another particle will allow a particle to stick to another. Applies after
            stubbornness [default: 1]
        --stubbornness <stubbornness>
            Defines how many interactions are necessary for a particle to stick to another. The number of join attempts
            is tracked per-particle [default: 0]
```

## Output format

There are two output formats.
One outputs the graph representing the connections between particles.
This is to allow modeling the network flow (making it look like an actual root or arterial network).

The other just outputs the particle coordinates in a WKT point cloud.

### TGF graph

This is the default output format.
It can be explicitly set by passing `--format tgf`

Note that the seed particle(s) will always be listed first, and not counted towards the number of `--particles`.

```shell
$ cargo run --release -- --format tgf --particles 4
0	POINT(0 0)
1	POINT(0.41126267623640556 0.9115168737521371)
2	POINT(0.2730681337390918 -0.9619946955863371)
3	POINT(-0.5849774368385797 0.9981519538314024)
4	POINT(1.0616508501787 0.15191485743305966)
#
1	 0
2	 0
3	 1
4	 1
```

Note that each node in the graph gets an integer ID, and a WKT label.
The edges are unlabeled, but could conceivably be weighted to model network flow.

This format can be consumed by the [geom2graph](../geom2graph) tool, which can go back and forth between geometries and their intersection graphs.
Note that the `geom2graph` tool is written in C++, and must be compiled before use.
It also has an [open bug](https://github.com/Notgnoshi/generative/issues/92) limiting its use to 2D geometries.

```shell
$ pushd ../geom2graph
$ git submodule update --init --recursive
$ mkdir build && cd build
$ cmake ..
$ make -j$(nproc)
$ popd
$ cargo run --release -- --format tgf --particles 4 | ../geom2graph/build/src/geom2graph --graph2geom
LINESTRING (0.7181469855612641 -0.6958914478058121, 0.9453811632260513 -1.669731593056052)
LINESTRING (0 0, 0.7181469855612641 -0.6958914478058121)
LINESTRING (0 0, 0.4041585052860049 0.9146889649520117)
LINESTRING (0 0, -0.9686994923890934 0.2482363660810256)
```

This LINESTRING output can be visualized by either the [render.py](../render.py) or [wkt2svg.py](wkt2svg) tools.

```shell
$ cargo run --release -- --format tgf --particles 100 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../render.py
```

![render.png](/examples/diffusion-limited-aggregation/render.png)

Note that when converting the WKT geometries to SVG, we scale the geometries 10x with the [project.py](../project.py) tool because the SVG generation uses a constant line-width of 1.

```shell
$ cargo run --release -- --format tgf --particles 100 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/wkt2svg.svg
```

![wkt2svg.svg](/examples/diffusion-limited-aggregation/wkt2svg.svg)

### WKT point cloud

We can also retrieve just the particle coordinates by passing `--output points` to the DLA tool.
Again, note that the seed particle(s) will always be listed first, and not counted towards the number of `--particles`.

```shell
$ cargo run --release -- --format points --particles 4
POINT (0 0)
POINT (0.7762362075762119 -0.6304421861262934)
POINT (1.774836054384656 -0.6833415807745498)
POINT (-0.08119975483432285 -1.1450328684798705)
POINT (1.9019358086650664 -1.6752315204663781)
```

This output can be directly consumed by [render.py](../render.py) or [wkt2svg.py](wkt2svg.py)

```shell
$ cargo run --release -- --format points --particles 4 | ../render.py --axis
```

![render-points.png](/examples/diffusion-limited-aggregation/render-points.png)

Again, we scale the geometries up before converting to an SVG.

```shell
$ cargo run --release -- --format points --particles 4 |
    ../project.py --kind I --scale 20 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/wkt2svg-points.svg
```

![wkt2svg-points.svg](/examples/diffusion-limited-aggregation/wkt2svg-points.svg)

```shell
$ cargo run --release -- \
        --seeds 2 \
        --attraction-distance 10 \
        --min-move-distance 1 \
        --stubbornness 10 \
        --particle-spacing 0.1 \
        --format points |
    ../render.py
```

![render-points-tuned.png](/examples/diffusion-limited-aggregation/render-points-tuned.png)

## Number of particles

You can adjust the number of particles added to the DLA model by setting the `--particles` option.

```shell
$ cargo run --release -- --particles 10000 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../render.py
```

![10k-particles](/examples/diffusion-limited-aggregation/10k-particles.png)

You can also adjust the number of seed particles used to initialize the DLA model by passing the `--seeds` option.

```shell
$ cargo run --release -- --seeds 100 --particles 10000 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../render.py
```

![10k-particles 100-seeds](/examples/diffusion-limited-aggregation/10k-particles-100-seeds.png)

## Reproducibility

This is a highly stochastic algorithm, so it's pretty important to allow specifying the random seed.
This can be done with the `--seed` commandline argument.
If the seed is negative (it defaults to `-1`) a random seed will be chosen.
The seed will be logged to `stderr` at the default INFO log level.

```shell
$ cargo run --release -- >/dev/null
INFO - Intializing rng with seed 6117905109029161177
$ cargo run --release -- >/dev/null
INFO - Intializing rng with seed 13409134547490921682
$ cargo run --release -- >/dev/null
INFO - Intializing rng with seed 13754397807511405978
```

If using a random seed generates a model you like, you can reproduce the results by passing `--seed` to the DLA tool.

## Attraction distance

There are two kinds of particles

1. Fixed particles that cannot move after fixing their position
2. Dynamic particles that take a random walk until they run into a fixed particle

The attraction distance configures how close the dynamic particles need to be to a fixed particle before they attempt to join to it.

```shell
$ cargo run --release -- --seed 18249510142899035534 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 20 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/attraction-default.svg
```

![attraction-default.svg](/examples/diffusion-limited-aggregation/attraction-default.svg)

```shell
$ cargo run --release -- --seed 18249510142899035534 --attraction-distance 10 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 20 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/attraction-10.svg
```

![attraction-10.svg](/examples/diffusion-limited-aggregation/attraction-10.svg)

## Min move distance

The minimum move distance is a parameter used in two cases:
1. When performing a random walk, we move the maximum of the distance to the closest fixed particle and the min move distance
2. When joining a dynamic particle to a fixed particle, we nudge the dynamic particle by the min move distance if it fails to join

```shell
$ cargo run --release -- --seed 15180982073580564743 --min-move-distance 1 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/min-move-1.svg
```

![min-move-1](/examples/diffusion-limited-aggregation/min-move-1.svg)

```shell
$ cargo run --release -- --seed 15180982073580564743 --min-move-distance 0.1 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/min-move-0.1.svg
```

![min-move-0.1](/examples/diffusion-limited-aggregation/min-move-0.1.svg)

```shell
$ cargo run --release -- --seed 15180982073580564743 --min-move-distance 5 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/min-move-5.svg
```

![min-move-5](/examples/diffusion-limited-aggregation/min-move-5.svg)

## Particle spacing

The particle spacing is the distance between fixed particles.

```shell
$ cargo run --release -- --seed 11831587872380458120 --particle-spacing 1 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/spacing-1.svg
```

![spacing-1](/examples/diffusion-limited-aggregation/spacing-1.svg)

```shell
$ cargo run --release -- --seed 11831587872380458120 --particle-spacing 5 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/spacing-5.svg
```

![spacing-5](/examples/diffusion-limited-aggregation/spacing-5.svg)

```shell
$ cargo run --release -- --seed 11831587872380458120 --particle-spacing 0.1 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 50 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/spacing-0.1.svg
```

![spacing-0.1](/examples/diffusion-limited-aggregation/spacing-0.1.svg)

## Stubbornness

Each fixed particle keeps track of how many attempts have been made to join it.
Stubbornness is the number of attempts that must be made before any attempt works.

It's a way to bias joining particles where particles have already been joined.

```shell
$ cargo run --release -- --seed 8110181389453817884 --stubbornness 0 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py  --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/stubbornness-0.svg
```

![stubbornness-0](/examples/diffusion-limited-aggregation/stubbornness-0.svg)

```shell
$ cargo run --release -- --seed 8110181389453817884 --stubbornness 5 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py  --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/stubbornness-5.svg
```

![stubbornness-5](/examples/diffusion-limited-aggregation/stubbornness-5.svg)

```shell
$ cargo run --release -- --seed 8110181389453817884 --stubbornness 10 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py  --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/stubbornness-10.svg
```

![stubbornness-10](/examples/diffusion-limited-aggregation/stubbornness-10.svg)

The higher the stubbornness, the more likely the DLA model is to form arteries.

## Stickiness

Stickiness is similar to stubbornness.
It's the probability that any attempt succeeds, after the stubbornness has been accounted for.

```shell
$ cargo run --release -- --seed 17626503329693161072 --stickiness 1.0 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/stickiness-1.svg
```

![stickiness-1](/examples/diffusion-limited-aggregation/stickiness-1.svg)

```shell
$ cargo run --release -- --seed 17626503329693161072 --stickiness 0.5 |
    ../geom2graph/build/src/geom2graph --graph2geom |
    ../project.py --kind I --scale 10 |
    ../wkt2svg.py --output ../../examples/diffusion-limited-aggregation/stickiness-0.5.svg
```

![stickiness-0.5](/examples/diffusion-limited-aggregation/stickiness-0.5.svg)
