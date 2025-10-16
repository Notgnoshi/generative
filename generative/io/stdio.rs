use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

pub fn get_output_writer(output: &Option<PathBuf>) -> eyre::Result<BufWriter<Box<dyn Write>>> {
    match output {
        Some(path) => {
            let file = File::create(path)?;
            Ok(BufWriter::new(Box::new(file)))
        }
        None => Ok(BufWriter::new(Box::new(std::io::stdout()))),
    }
}

pub fn get_input_reader(input: &Option<PathBuf>) -> eyre::Result<BufReader<Box<dyn Read>>> {
    match input {
        Some(path) => {
            let file = File::open(path)?;
            Ok(BufReader::new(Box::new(file)))
        }
        None => Ok(BufReader::new(Box::new(std::io::stdin()))),
    }
}
