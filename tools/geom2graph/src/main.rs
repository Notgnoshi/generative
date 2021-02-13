use structopt::StructOpt;

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "geom2graph",
    about = "A CLI application to convert geometries into a graph data structure."
)]
struct CmdLineOptions {
    /// Increase output verbosity.
    #[structopt(short, long)]
    verbose: bool,

    /// Input file to read from. Defaults to stdin.
    #[structopt(short, long, parse(from_os_str))]
    input: Option<PathBuf>,

    /// Output file to write to. Defaults to stdout.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}

/// Get a BufWriter for the given path or stdout.
fn get_writer(output: Option<PathBuf>) -> BufWriter<Box<dyn Write>> {
    match output {
        Some(path) => match File::create(&path) {
            Err(why) => panic!("couldn't create {} because: {}", path.display(), why),
            Ok(file) => BufWriter::new(Box::new(file)),
        },
        None => BufWriter::new(Box::new(std::io::stdout())),
    }
}

/// Get a BufReader for the given path or stdin.
fn get_reader(input: Option<PathBuf>) -> BufReader<Box<dyn Read>> {
    match input {
        Some(path) => match File::open(&path) {
            Err(why) => panic!("couldn't read {} because: {}", path.display(), why),
            Ok(file) => BufReader::new(Box::new(file)),
        },
        None => BufReader::new(Box::new(std::io::stdin())),
    }
}

fn main() {
    let args = CmdLineOptions::from_args();
    if args.verbose {
        println!("{:#?}", args);
    }
    let mut writer = get_writer(args.output);
    let reader = get_reader(args.input);

    writeln!(&mut writer, "sample output").expect("Couldn't write?!");
    writer.flush().unwrap();

    for line in reader.lines() {
        if let Ok(geom) = line {
            println!("{}", geom);
        }
    }
}
