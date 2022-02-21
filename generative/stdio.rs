use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

pub fn get_output_writer(output: Option<PathBuf>) -> Result<BufWriter<Box<dyn Write>>, String> {
    match output {
        Some(path) => match File::create(&path) {
            Err(why) => Err(format!(
                "Couldn't create: '{}' because: '{}'",
                path.display(),
                why
            )),
            Ok(file) => Ok(BufWriter::new(Box::new(file))),
        },
        None => Ok(BufWriter::new(Box::new(std::io::stdout()))),
    }
}

pub fn get_input_reader(input: Option<PathBuf>) -> Result<BufReader<Box<dyn Read>>, String> {
    match input {
        Some(path) => match File::open(&path) {
            Err(why) => Err(format!(
                "Couldn't open: '{}' because: '{}'",
                path.display(),
                why
            )),
            Ok(file) => Ok(BufReader::new(Box::new(file))),
        },
        None => Ok(BufReader::new(Box::new(std::io::stdin()))),
    }
}
