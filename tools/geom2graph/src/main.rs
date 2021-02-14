//use log::{debug, error, info, trace, warn};
use std::io::Write;

mod cmdline;
mod wkio;

fn main() {
    let args = cmdline::Options::from_args();
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose)
        .init()
        .unwrap();

    let mut writer = args.get_output_writer();
    let deserializer = wkio::WktDeserializer::new(args.get_input_reader());

    writeln!(&mut writer, "sample output").expect("Couldn't write?!");
    writer.flush().unwrap();

    for _geom in deserializer {
        // TODO: Do something useful with the geometries.
    }
}
