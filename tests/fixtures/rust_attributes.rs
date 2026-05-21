#[derive(Debug, Clone)]
#[pyclass]
pub struct TupleReader {
    #[pyo3(get)]
    items: Vec<u8>,
}

#[pymethods]
impl TupleReader {
    #[new]
    pub fn new(items: Vec<u8>) -> Self {
        Self { items }
    }

    #[getter]
    pub fn remaining(&self) -> usize {
        self.items.len()
    }
}

pub enum WireValue {
    #[serde(rename = "cell")]
    Cell,
}

#[outer]
// keep attr attached
pub fn documented_attr() {}

pub struct TupleAttrs(
    #[first]
    pub u8,
    // separator
    #[second]
    pub(crate) String,
);

#[cfg(test)]
mod attr_mod {
    #[test]
    fn inner() {}
}
