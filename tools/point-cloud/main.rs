mod cmdline;

use clap::Parser;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::Binomial;
use std::io::Write;

use cmdline::RandomDomain;
use generative::stdio::get_output_writer;

struct Double2 {
    x: f64,
    y: f64,
}

fn generate(points: usize, domain: RandomDomain, rng: &mut StdRng) -> Vec<Double2> {
    match domain {
        RandomDomain::UnitSquare => generate_square(points, rng),
        RandomDomain::UnitCircle => generate_circle(points, rng),
    }
}

fn generate_square(points: usize, rng: &mut StdRng) -> Vec<Double2> {
    let mut v = Vec::with_capacity(points);
    let dist = Uniform::from(0.0..1.0);

    for _ in 0..points {
        let point = Double2 {
            x: dist.sample(rng),
            y: dist.sample(rng),
        };
        v.push(point);
    }

    v
}

fn generate_circle(points: usize, rng: &mut StdRng) -> Vec<Double2> {
    let mut v = Vec::with_capacity(points);

    let r_dist = Uniform::from(0.0..1.0);
    let theta_dist = Uniform::from(0.0..2.0 * std::f64::consts::PI);

    for _ in 0..points {
        let r = r_dist.sample(rng);
        let theta = theta_dist.sample(rng);

        let point = Double2 {
            x: r * theta.cos(),
            y: r * theta.sin(),
        };
        v.push(point);
    }

    v
}

fn generate_random_seed_if_not_specified(seed: u64) -> u64 {
    if seed == 0 {
        let mut rng = rand::thread_rng();
        rng.gen()
    } else {
        seed
    }
}

fn main() {
    let args = cmdline::CmdlineOptions::parse();
    let seed = generate_random_seed_if_not_specified(args.seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let num_points = if args.random_number {
        // close enough to normal with integers
        let n = args.points * 2; // changes mean
        let p = 0.5; // changes skew
        let dist = Binomial::new(n, p).unwrap();
        dist.sample(&mut rng)
    } else {
        args.points
    };

    eprintln!("Generating {} points with seed {}", num_points, seed);

    let points = generate(num_points as usize, args.domain, &mut rng);
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
