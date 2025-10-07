use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use geo::{Coord, LineString};
use wkt::ToWkt;

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    /// Write out each visited point as a WKT POINT
    Points,
    /// Write out each visited point in a WKT LINESTRING
    Line,
    // TODO: Image
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            OutputFormat::Points => write!(f, "points"),
            OutputFormat::Line => write!(f, "line"),
        }
    }
}

// TODO: A performant output writer that can handle parallelism and different output formats
//
// * Image: would need to share a thread-safe image buffer, and then write to the actual image
//   file all at once at the end. Each thread would have to keep its own local copy of the 2D
//   histogram, and then merge them at the end. It's fine to keep everything in-memory for the
//   image writer, because you have to build a 2D histogram of the hit pixels anyway, and
//   that's gotta be in-memory regardless. So what if we have to spin up a few copies of it
//   per-thread and then merge? It's not gonna be gigabytes... I hope
//
//   But then how do you map the (x, y) coordinates to pixel coordinates without loading all of
//   them into memory first (to find the min/max extents)?

pub struct AttractorFormatter {
    format: OutputFormat,
    writer: BufWriter<Box<dyn Write>>,

    accumulated: Vec<Coord>,
}

// public
impl AttractorFormatter {
    pub fn new(
        format: OutputFormat,
        output: Option<PathBuf>,
        expected_coords: usize,
    ) -> eyre::Result<Self> {
        let writer: Box<dyn Write> = match output {
            Some(path) => {
                if path == Path::new("-") {
                    Box::new(std::io::stdout())
                } else {
                    let file = std::fs::File::create(path)?;
                    Box::new(file)
                }
            }
            None => Box::new(std::io::stdout()),
        };
        let writer = BufWriter::new(writer);
        let accumulated = Vec::with_capacity(expected_coords);

        Ok(Self {
            format,
            writer,
            accumulated,
        })
    }

    pub fn handle_point(&mut self, x: f64, y: f64) -> eyre::Result<()> {
        match self.format {
            OutputFormat::Points | OutputFormat::Line => self.accumulate_coord(Coord { x, y }),
        }
    }

    pub fn flush(&mut self) -> eyre::Result<()> {
        self.write_accumulated()?;
        self.writer.flush()?;
        Ok(())
    }

    /// In the case parallelism is used, merge another AttractorFormatter from another thread into
    /// this one
    pub fn merge(&mut self, _other: AttractorFormatter) {
        // TODO: This is mostly only useful for the image formatter
    }
}

// private
impl AttractorFormatter {
    fn accumulate_coord(&mut self, coord: Coord) -> eyre::Result<()> {
        self.accumulated.push(coord);
        // TODO: Don't do this for images
        if self.accumulated.len() > 1_000 {
            self.write_accumulated()?;
        }
        Ok(())
    }

    fn write_accumulated(&mut self) -> eyre::Result<()> {
        let mut accumulated = Vec::new();
        std::mem::swap(&mut self.accumulated, &mut accumulated);
        match self.format {
            OutputFormat::Points => {
                for coord in &accumulated {
                    writeln!(self.writer, "POINT({} {})", coord.x, coord.y)?;
                }
            }
            OutputFormat::Line => {
                if !accumulated.is_empty() {
                    let linestring = LineString::from(accumulated);
                    writeln!(self.writer, "{}", linestring.to_wkt())?;
                }
            }
        }

        Ok(())
    }
}
