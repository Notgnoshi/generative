use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashSet};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;

use bitvec::vec::BitVec;
use clap::Parser;
use generative::io::{get_output_writer, write_geometries, GeometryFormat};
use geo::{Coord, Geometry, LineString};
use stderrlog::ColorChoice;

/// Generate fogleworms
///
/// Generate distinct tilings of N "worms" of length N in an NxN grid.
#[derive(Debug, Parser)]
#[clap(name = "fogleworms", verbatim_doc_comment)]
struct CmdlineOptions {
    /// The log level
    #[clap(short, long, default_value_t = log::Level::Info)]
    pub log_level: log::Level,

    /// Output file to write result to. Defaults to stdout.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Output geometry format.
    #[clap(short = 'O', long, default_value_t = GeometryFormat::Wkt)]
    pub output_format: GeometryFormat,

    /// Grid size
    #[clap(default_value_t = 4)]
    pub grid_size: u32,
}

fn div_rem<T: std::ops::Div<Output = T> + std::ops::Rem<Output = T> + Copy>(x: T, y: T) -> (T, T) {
    let quot = x / y;
    let rem = x % y;
    (quot, rem)
}

struct Descendants<'legacy> {
    pub ancestor: &'legacy Worm,
}

struct Worm {
    pub indices: Vec<usize>,
}

impl Worm {
    fn new(size: usize) -> Self {
        Self {
            indices: Vec::with_capacity(size),
        }
    }

    fn descendants(&self) -> Descendants<'_> {
        Descendants { ancestor: self }
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.indices.hash(&mut hasher);
        hasher.finish()
    }
}

struct Grid {
    pub size: usize,
    pub filled: BitVec,
    pub worms: Vec<Worm>,
    pub history: BTreeSet<u64>,
}

impl Grid {
    fn new(size: usize) -> Self {
        Self {
            size,
            filled: BitVec::repeat(false, size * size),
            worms: Vec::with_capacity(size),
            history: BTreeSet::new(),
        }
    }

    fn can_place(&self, worm: &Worm) -> bool {
        for index in worm.indices.iter() {
            if let Some(v) = self.filled.get(*index) {
                if *v {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn try_place(&mut self, worm: Worm) -> bool {
        if self.can_place(&worm) {
            self.history.insert(worm.hash());
            for index in worm.indices.iter() {
                // unsafe {
                //     self.filled.set_unchecked(*index, true);
                // }
                self.filled.set(*index, true);
            }
            self.worms.push(worm);
            true
        } else {
            false
        }
    }

    /// Use flood fill to generate the next available worm on the grid
    fn next_candidate(&mut self) -> Option<Worm> {
        if self.filled.count_ones() == self.size {
            return None;
        }
        if let Some(mut current) = self.filled.first_zero() {
            let mut temp = self.filled.clone();
            let mut worm = Worm::new(self.size);

            temp.set(current, true);
            worm.indices.push(current);

            for _ in 0..self.size - 1 {
                // Checking for adjacent unfilled cells in North, West, East, South order is
                // equivalent to biasing for smallest unfilled cell.

                let (north, north_overflow) = current.overflowing_sub(self.size);
                let west = current - 1;
                let east = current + 1;
                let south = current + self.size;

                if !north_overflow && !temp.get(north).unwrap() {
                    worm.indices.push(north);
                    temp.set(north, true);
                    current = north;
                    continue;
                }
                if current % self.size != 0 && !temp.get(west).unwrap() {
                    worm.indices.push(west);
                    temp.set(west, true);
                    current = west;
                    continue;
                }
                if east % self.size != 0 && !temp.get(east).unwrap() {
                    worm.indices.push(east);
                    temp.set(east, true);
                    current = east;
                    continue;
                }
                if south < self.size * self.size && !temp.get(south).unwrap() {
                    worm.indices.push(south);
                    temp.set(south, true);
                    current = south;
                    continue;
                }
                // No adjacent cell
                return None;
            }
            return Some(worm);
        }
        None
    }

    fn write<W>(&self, writer: W, format: &GeometryFormat)
    where
        W: Write,
    {
        let geometries = self.worms.iter().map(|worm| {
            let coords = worm.indices.iter().map(|index| {
                let (row, col) = div_rem(*index, self.size);
                Coord {
                    x: col as f64,
                    y: row as f64,
                }
            });
            Geometry::LineString(LineString::from_iter(coords))
        });
        write_geometries(writer, geometries, format);
    }
}

/// Problem taken from <https://twitter.com/FogleBird/status/1349563985268531200>. Generate
/// distinct tilings of N "worms" of length N in an NxN grid.
fn main() {
    let args = CmdlineOptions::parse();

    stderrlog::new()
        .verbosity(args.log_level)
        .color(ColorChoice::Auto)
        .init()
        .expect("Failed to initialize stderrlog");

    let solutions = HashSet::<Grid>::new();

    let mut writer = get_output_writer(&args.output).unwrap();
    for solution in solutions {
        solution.write(&mut writer, &args.output_format);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_candidate() {
        let mut grid = Grid::new(4);
        for i in [0, 1, 2, 3, 6, 7, 10, 11] {
            grid.filled.set(i, true);
        }

        let candidate = grid.next_candidate().unwrap();
        assert_eq!(candidate.indices, [4, 5, 9, 8]);
    }
}
