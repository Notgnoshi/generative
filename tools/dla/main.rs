use generative::dla::format_tgf;
use generative::dla::format_wkt;
use generative::dla::Model;
use log::trace;

mod cmdline;

fn main() {
    let args = cmdline::Options::from_args();

    stderrlog::new()
        .quiet(args.quiet)
        .verbosity(args.verbose + 2) // Default to INFO level
        .init()
        .unwrap();

    let mut model = Model::new(
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
    match args.format {
        cmdline::OutputFormat::GraphTGF => {
            format_tgf(&mut writer, model.particle_graph);
        }
        cmdline::OutputFormat::PointCloudWKT => {
            format_wkt(&mut writer, model.particle_graph);
        }
    };
}
