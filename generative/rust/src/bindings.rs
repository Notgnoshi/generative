use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::convert::TryInto;

/// TODO: Move to dla module
#[derive(Debug)]
struct Parameters {
    seeds: usize,
    seed: u64,
    particle_spacing: f64,
    attraction_distance: f64,
    min_move_distance: f64,
    stubbornness: usize,
    stickiness: f64,
}

/// TODO: Implement the Send trait to allow multithreaded access?
/// TODO: DLA stuff shouldn't go in the top level python library!
#[pyclass(unsendable, name = "Parameters")]
#[derive(Debug)]
#[allow(non_camel_case_types)]
struct Py_Parameters {
    #[pyo3(get, set)]
    seeds: usize,
    #[pyo3(get, set)]
    seed: u64,
    #[pyo3(get, set)]
    particle_spacing: f64,
    #[pyo3(get, set)]
    attraction_distance: f64,
    #[pyo3(get, set)]
    min_move_distance: f64,
    #[pyo3(get, set)]
    stubbornness: usize,
    #[pyo3(get, set)]
    stickiness: f64,
}

#[pymethods]
impl Py_Parameters {
    #[new]
    fn new(
        seeds: usize,
        seed: u64,
        particle_spacing: f64,
        attraction_distance: f64,
        min_move_distance: f64,
        stubbornness: usize,
        stickiness: f64,
    ) -> Self {
        Py_Parameters {
            seeds,
            seed,
            particle_spacing,
            attraction_distance,
            min_move_distance,
            stubbornness,
            stickiness,
        }
    }
}

impl Py_Parameters {
    fn to_dla_params(&self) -> Parameters {
        Parameters {
            seeds: self.seeds,
            seed: self.seed,
            particle_spacing: self.particle_spacing,
            attraction_distance: self.attraction_distance,
            min_move_distance: self.min_move_distance,
            stubbornness: self.stubbornness,
            stickiness: self.stickiness,
        }
    }
}

#[pyclass(unsendable, name = "Particle")]
#[derive(Debug)]
#[allow(non_camel_case_types)]
struct Py_Particle {
    particle: crate::dla::model::Particle,
}

#[pymethods]
impl Py_Particle {
    #[new]
    fn new(x: Option<f64>, y: Option<f64>, join_attempts: Option<usize>) -> Self {
        let mut coordinates: [f64; 2] = [0.0, 0.0];
        if let Some(x) = x {
            coordinates[0] = x;
        }
        if let Some(y) = y {
            coordinates[1] = y;
        }

        Py_Particle {
            particle: crate::dla::model::Particle {
                coordinates,
                join_attempts: match join_attempts {
                    Some(j) => j,
                    None => 0,
                },
            },
        }
    }

    #[getter]
    fn get_x(&self) -> PyResult<f64> {
        Ok(self.particle.coordinates[0])
    }
    #[setter]
    fn set_x(&mut self, value: f64) -> PyResult<()> {
        self.particle.coordinates[0] = value;
        Ok(())
    }

    #[getter]
    fn get_y(&self) -> PyResult<f64> {
        Ok(self.particle.coordinates[1])
    }
    #[setter]
    fn set_y(&mut self, value: f64) -> PyResult<()> {
        self.particle.coordinates[1] = value;
        Ok(())
    }

    #[getter]
    fn get_coordinates(&self) -> PyResult<Vec<f64>> {
        Ok(self.particle.coordinates.to_vec())
    }

    #[setter]
    fn set_coordinates(&mut self, value: Vec<f64>) -> PyResult<()> {
        let array: [f64; 2] = match value.try_into() {
            Ok(a) => a,
            Err(o) => {
                return Err(PyValueError::new_err(format!(
                    "Length must be 2. Got list with length {}",
                    o.len()
                )));
            }
        };
        self.particle.coordinates = array;
        Ok(())
    }

    #[getter]
    fn get_join_attempts(&self) -> PyResult<usize> {
        Ok(self.particle.join_attempts)
    }
    #[setter]
    fn set_join_attempts(&mut self, value: usize) -> PyResult<()> {
        self.particle.join_attempts = value;
        Ok(())
    }
}

impl Py_Particle {
    fn from_dla_particle(particle: crate::dla::model::Particle) -> Self {
        Py_Particle { particle }
    }
}

/// TODO: What API do I _need_ to expose?
/// TODO: What API _should_ I expose?
/// TODO: Why does this have to implement Clone?
#[pyclass(unsendable, name = "Graph")]
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
struct Py_Graph {
    graph: crate::dla::model::GraphType,
}

#[pymethods]
impl Py_Graph {}

/// TODO: How should the graph be wrapped? I think perhaps it should be a mutable reference?
#[pyclass(unsendable, name = "Model")]
#[derive(Debug)]
#[allow(non_camel_case_types)]
struct Py_Model {
    model: crate::dla::model::Model,
    #[pyo3(get, set)]
    particle_graph: Py_Graph,
}

#[pymethods]
impl Py_Model {}

#[pymodule]
fn rust(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Py_Parameters>()?;
    m.add_class::<Py_Particle>()?;
    m.add_class::<Py_Graph>()?;
    m.add_class::<Py_Model>()?;
    Ok(())
}
