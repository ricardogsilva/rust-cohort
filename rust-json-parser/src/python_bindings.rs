use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::fs::read_to_string;

use crate::{JsonError, JsonParser, JsonValue};

// type conversion: Rust -> Python
impl<'py> IntoPyObject<'py> for JsonValue {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            JsonValue::Null => Ok(py.None().into_bound(py)),
            JsonValue::Boolean(v) => Ok(v.into_pyobject(py)?.to_owned().into_any()),
            JsonValue::Number(v) => Ok(v.into_pyobject(py)?.to_owned().into_any()),
            JsonValue::String(v) => Ok(v.into_pyobject(py)?.to_owned().into_any()),
            JsonValue::Array(v) => {
                let py_list = PyList::empty(py);
                for item in v {
                    py_list.append(item.into_pyobject(py)?)?;
                }
                Ok(py_list.into_any())
            }
            JsonValue::Object(v) => {
                let py_dict = PyDict::new(py);
                for (key, value) in v {
                    py_dict.set_item(key, value.into_pyobject(py)?)?;
                }
                Ok(py_dict.into_any())
            }
        }
    }
}

// type conversion: Rust error -> Python exception
impl From<JsonError> for PyErr {
    fn from(err: JsonError) -> PyErr {
        match err {
            JsonError::InvalidEscape { char, position } => PyValueError::new_err(format!(
                "Invalid escape at position {position}: char {char}"
            )),
            JsonError::InvalidNumber { value, position } => PyValueError::new_err(format!(
                "Invalid number found at position {position}: {value}"
            )),
            JsonError::InvalidUnicode { sequence, position } => PyValueError::new_err(format!(
                "Invalid unicode at position {position}: sequence {sequence}"
            )),
            JsonError::UnexpectedEndOfInput { expected, position } => PyValueError::new_err(
                format!("Unexpected end of input at position {position}: expected {expected}"),
            ),
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => PyValueError::new_err(format!(
                "Unexpected token at position {position}: expected {expected}, found {found}"
            )),
        }
    }
}

// Python callable functions
#[pyfunction]
fn parse_json<'py>(py: Python<'py>, input: &str) -> PyResult<Bound<'py, PyAny>> {
    let mut parser = JsonParser::new(input)?;
    let result = parser.parse()?;
    // this `py` variable holds a token which proves we have the Python GIL
    // and are therefore safe to manipulate Python objects
    result.into_pyobject(py)
}

#[pyfunction]
fn parse_json_file<'py>(py: Python<'py>, path: &str) -> PyResult<Bound<'py, PyAny>> {
    let raw = read_to_string(path)?;
    let mut parser = JsonParser::new(&raw)?;
    let result = parser.parse()?;
    result.into_pyobject(py)
}

#[pyfunction]
#[pyo3(signature = (obj, indent=None))]
fn dumps(obj: &Bound<PyAny>, indent: Option<usize>) -> PyResult<String> {
    let converted = py_to_json_value(obj)?;
    match indent {
        Some(_) => Ok(JsonValue::pretty_print(&converted, indent)),
        None => Ok(converted.to_string()),
    }
}

// this is not exposed to python (note the lack of #[pyfunction] rust property)
fn py_to_json_value(obj: &Bound<PyAny>) -> PyResult<JsonValue> {
    if obj.is_none() {
        return Ok(JsonValue::Null);
    }

    // check for bool BEFORE checking for number
    // - the course docs mention that we need to check for bool before
    // checking for number because Python implements bool as a number
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(JsonValue::Boolean(b));
    }

    if let Ok(n) = obj.extract::<f64>() {
        return Ok(JsonValue::Number(n));
    }

    if let Ok(s) = obj.extract::<String>() {
        return Ok(JsonValue::String(s));
    }

    if let Ok(list) = obj.cast::<PyList>() {
        let mut v: Vec<JsonValue> = Vec::new();
        for i in list.iter() {
            v.push(py_to_json_value(&i)?);
        }
        return Ok(JsonValue::Array(v));
    }

    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut hmap: HashMap<String, JsonValue> = HashMap::new();
        for (k, v) in dict.iter() {
            hmap.insert(k.extract::<String>()?, py_to_json_value(&v)?);
        }
        return Ok(JsonValue::Object(hmap));
    }

    Err(PyValueError::new_err(
        "Unsupported type for JSON conversion",
    ))
}

// module registration - This is essential for Python to be able to use the
// functions annotated with #[pyfunction]
#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;

    // it would be nice to retrieve this dynamically from cargo.toml somehow
    // even better would be if cargo would retrieve it from git and never write
    // it in cargo.toml in the first place - this serves as a good example of how
    // to add additional metadata to a Python module
    m.add("__version__", "0.1.0")?;
    Ok(())
}
