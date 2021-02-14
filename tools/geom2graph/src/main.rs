use log::{debug, error, info, trace, warn};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use wkt::Wkt;

mod cmdline;

/// Get a BufWriter for the given path or stdout.
fn get_writer(output: Option<PathBuf>) -> BufWriter<Box<dyn Write>> {
    match output {
        Some(path) => match File::create(&path) {
            Err(why) => panic!("couldn't create {} because: {}", path.display(), why),
            Ok(file) => {
                trace!("Writing to {}", path.display());
                BufWriter::new(Box::new(file))
            }
        },
        None => {
            trace!("Writing to stdout");
            BufWriter::new(Box::new(std::io::stdout()))
        }
    }
}

/// Get a BufReader for the given path or stdin.
fn get_reader(input: Option<PathBuf>) -> BufReader<Box<dyn Read>> {
    match input {
        Some(path) => match File::open(&path) {
            Err(why) => panic!("couldn't read {} because: {}", path.display(), why),
            Ok(file) => {
                trace!("Reading from {}", path.display());
                BufReader::new(Box::new(file))
            }
        },
        None => {
            trace!("Reading from stdin");
            BufReader::new(Box::new(std::io::stdin()))
        }
    }
}

#[derive(Debug)]
struct WktDeserializer<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> WktDeserializer<R> {
    fn new(reader: BufReader<R>) -> WktDeserializer<R> {
        WktDeserializer { reader }
    }
}

impl<R: Read> Iterator for WktDeserializer<R> {
    type Item = Wkt<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        // We were able to read a line.
        if let Ok(_bytes) = self.reader.read_line(&mut buf) {
            trace!("Read: {}", buf);

            if let Ok(geom) = Wkt::<f64>::from_str(&buf) {
                return Some(geom);
            } else {
                warn!("Failed to parse '{}' as WKT", buf);
                // Don't return an error, because we want to keep reading geometries.
                return None;
            }
        } else {
            return None;
        }
    }
}

fn main() {
    let args = cmdline::Options::from_args();
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose)
        .init()
        .unwrap();

    let mut writer = get_writer(args.output);
    let reader = get_reader(args.input);
    let deserializer = WktDeserializer::new(reader);

    writeln!(&mut writer, "sample output").expect("Couldn't write?!");
    writer.flush().unwrap();

    for geom in deserializer {
        info!("deserialized: {:?}", geom);
    }
}
