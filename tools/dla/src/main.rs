use std::io::Write;
use log::trace;
mod cmdline;
mod dla;

fn main() {
    let args = cmdline::Options::from_args();

    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose + 2) // Default to INFO level
        .init()
        .unwrap();

    let mut model = dla::Model::new(
        args.dimensions,
        // TODO: Seed type.
        args.seeds,
        args.seed,
        args.particle_spacing,
        args.attraction_distance,
        args.min_move_distance,
        args.stubbornness,
        args.stickiness,
    );

    model.run(args.particles);

    trace!("Model {:?}", model);

    let mut writer = args.get_output_writer();
    writeln!(&mut writer, "placeholder output").expect("Failed to write output");

    // let formatter = match args.format {
    //     cmdline::OutputFormat::GraphTGF => TGFFormatter::new(),
    //     cmdline::OutputFormat::PointCloudWKT => PointCloudFormatter::new(),
    // };
    // formatter.format(&mut writer, &model.particle_graph);
}
