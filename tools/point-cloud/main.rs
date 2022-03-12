mod cmdline;

use clap::Parser;
use rand::distributions::{Distribution, Uniform};
use std::io::Write;

use cmdline::RandomDomain;
use generative::stdio::get_output_writer;

// TODO: Use a real computational geometry library that knows about WKT
struct Double2 {
    x: f64,
    y: f64,
}

fn generate(points: usize, domain: RandomDomain) -> Vec<Double2> {
    match domain {
        RandomDomain::UnitSquare => generate_square(points),
        RandomDomain::UnitCircle => generate_circle(points),
    }
}

fn generate_square(points: usize) -> Vec<Double2> {
    let mut v = Vec::with_capacity(points);

    let mut rng = rand::thread_rng();
    let dist = Uniform::from(0.0..1.0);

    for _ in 0..points {
        let point = Double2 {
            x: dist.sample(&mut rng),
            y: dist.sample(&mut rng),
        };
        v.push(point);
    }

    v
}

fn generate_circle(points: usize) -> Vec<Double2> {
    let mut v = Vec::with_capacity(points);

    let mut rng = rand::thread_rng();
    let r_dist = Uniform::from(0.0..1.0);
    let theta_dist = Uniform::from(0.0..2.0 * std::f64::consts::PI);

    for _ in 0..points {
        let r = r_dist.sample(&mut rng);
        let theta = theta_dist.sample(&mut rng);

        let point = Double2 {
            x: r * theta.cos(),
            y: r * theta.sin(),
        };
        v.push(point);
    }

    v
}

fn main() {
    let args = cmdline::CmdlineOptions::parse();

    let points = generate(args.num_points, args.domain);
    let mut writer = get_output_writer(args.output).unwrap();
    for point in points {
        writeln!(
            writer,
            "POINT ({} {})",
            point.x * args.scale,
            point.y * args.scale
        )
        .expect("Failed to write random point");
    }
}
