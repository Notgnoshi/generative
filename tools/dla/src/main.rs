use std::io::Write;
mod cmdline;

fn main() {
    let args = cmdline::Options::from_args();

    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose + 2) // Default to INFO level
        .init()
        .unwrap();

    let mut writer = args.get_output_writer();
    writeln!(&mut writer, "placeholder output").expect("Failed to write output");
}
