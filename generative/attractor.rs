use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use geo::{BoundingRect, Coord, LineString};
use wkt::ToWkt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Write out each visited point as a WKT POINT
    Points,
    /// Write out each visited point in a WKT LINESTRING
    Line,
    /// Write out the visited points as a PNG image
    Image,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // important: Should match clap::ValueEnum format
            OutputFormat::Points => write!(f, "points"),
            OutputFormat::Line => write!(f, "line"),
            OutputFormat::Image => write!(f, "image"),
        }
    }
}

pub struct AttractorFormatter {
    format: OutputFormat,
    writer: BufWriter<Box<dyn Write>>,
    output: Option<PathBuf>,

    accumulated: Vec<Coord>,

    width: Option<u32>,
    height: Option<u32>,
}

// public
impl AttractorFormatter {
    pub fn new(
        format: OutputFormat,
        output: Option<PathBuf>,
        expected_coords: usize,
        width: Option<u32>,
        height: Option<u32>,
    ) -> eyre::Result<Self> {
        let writer: Box<dyn Write> = match &output {
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
        let buffer_capacity = match format {
            OutputFormat::Points | OutputFormat::Line => 1024,
            OutputFormat::Image => expected_coords,
        };
        let accumulated = Vec::with_capacity(buffer_capacity);

        Ok(Self {
            format,
            writer,
            output,
            accumulated,
            width,
            height,
        })
    }

    pub fn handle_point(&mut self, x: f64, y: f64) -> eyre::Result<()> {
        self.accumulate_coord(Coord { x, y })
    }

    pub fn flush(&mut self) -> eyre::Result<()> {
        self.write_accumulated()?;
        self.writer.flush()?;
        Ok(())
    }
}

// private
impl AttractorFormatter {
    fn accumulate_coord(&mut self, coord: Coord) -> eyre::Result<()> {
        self.accumulated.push(coord);
        // Writing an image requires saving all of the points in memory so we can map them to pixel
        // coordinates later. For other formats, we can flush periodically to save memory.
        if self.format != OutputFormat::Image
            && self.accumulated.len() == self.accumulated.capacity()
        {
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
            OutputFormat::Image => self.write_image(accumulated)?,
        }

        Ok(())
    }

    fn write_image(&mut self, accumulated: Vec<Coord<f64>>) -> eyre::Result<()> {
        let accumulated = LineString::from(accumulated);
        let bbox = accumulated.bounding_rect().ok_or_else(|| {
            eyre::eyre!("Cannot determine bounding box of accumulated coordinates")
        })?;
        let (width, height) = self.determine_image_size(&bbox);

        // Padding is to avoid off-by-one errors due to rounding floats -> int
        let mut image = image::GrayImage::new(width + 1, height + 1);
        for pixel in image.pixels_mut() {
            pixel.0[0] = 255; // white
            // I struggled using GrayAlphaImage and setting the alpha values correctly. Maybe I'll
            // revisit that later. For now, just darken the pixels on each visit.
        }
        for coord in accumulated {
            let (x, y) = Self::map_coordinate_to_pixel(&coord, &bbox, width, height);
            let pixel = image.get_pixel_mut(x, y);
            pixel.0[0] = pixel.0[0].saturating_sub(64); // darken the pixel, but don't wrap around!
        }

        image.save_with_format(self.output.as_ref().unwrap(), image::ImageFormat::Png)?;

        Ok(())
    }

    fn determine_image_size(&self, bbox: &geo::Rect) -> (u32, u32) {
        let coord_width = bbox.max().x - bbox.min().x;
        let coord_height = bbox.max().y - bbox.min().y;
        let aspect_ratio = coord_width / coord_height;
        tracing::debug!("extents: {bbox:?}");
        tracing::debug!(
            "dimensions: {coord_width:.4} x {coord_height:.4}, aspect: {aspect_ratio:.4}"
        );

        let (width, height) = match (self.width, self.height) {
            (Some(width), Some(height)) => (width, height),
            (None, None) => {
                let default_width = 800;
                let height = (default_width as f64 / aspect_ratio) as u32;
                (default_width, height)
            }
            (Some(width), None) => {
                let height = (width as f64 / aspect_ratio) as u32;
                (width, height)
            }
            (None, Some(height)) => {
                let width = (height as f64 * aspect_ratio) as u32;
                (width, height)
            }
        };
        tracing::debug!("Image size: {width}x{height}");

        (width, height)
    }

    fn map_coordinate_to_pixel(
        coord: &Coord<f64>,
        bbox: &geo::Rect,
        image_width: u32,
        image_height: u32,
    ) -> (u32, u32) {
        let image_width = image_width as f64;
        let image_height = image_height as f64;
        // TODO: If this is expensive, we can precompute the scale factor
        let px_x = (coord.x - bbox.min().x) * image_width / (bbox.max().x - bbox.min().x);
        let px_y = (coord.y - bbox.min().y) * image_height / (bbox.max().y - bbox.min().y);

        // TODO: Round? Truncate?
        (px_x as u32, px_y as u32)
    }
}
