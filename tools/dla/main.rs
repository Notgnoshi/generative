use clap::Parser;
use generative::dla::format_tgf;
use generative::dla::format_wkt;
use generative::dla::Model;
use log::trace;

mod cmdline;

fn main() {
    let args = cmdline::CmdlineOptions::parse();

    stderrlog::new()
        .quiet(args.quiet)
        .verbosity((args.verbose + 2) as usize) // Default to INFO level
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
        cmdline::OutputFormat::Tgf => {
            format_tgf(&mut writer, model.particle_graph);
        }
        cmdline::OutputFormat::Wkt => {
            format_wkt(&mut writer, model.particle_graph);
        }
    };
}
