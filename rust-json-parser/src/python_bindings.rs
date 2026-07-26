use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::exceptions::PyValueError;

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
            JsonError::InvalidEscape { char, position } => PyValueError::new_err(
                format!("Invalid escape at position {position}: char {char}")
            ),
            JsonError::InvalidNumber { value, position } => PyValueError::new_err(
                format!("Invalid number found at position {position}: {value}")
            ),
            JsonError::InvalidUnicode { sequence, position } => PyValueError::new_err(
                format!("Invalid unicode at position {position}: sequence {sequence")
            ),
            JsonError::UnexpectedEndOfInput { expected, position } => PyValueError::new_err(
                format!("Unexpected end of input at position {position}: expected {expected}")
            ),
            JsonError::UnexpectedToken { expected, found, position } => PyValueError::new_err(
                format!("Unexpected token at position {position}: expected {expected}, found {found}")
            ),
        }
    }
}

// Python callable functions
#[pyfunction]
fn parse_json(py: Python<'_>, input: &str) -> PyResult<Bound<'_, PyAny>> {
    let mut parser = JsonParser::new(input)?;
    let result = parser.parse()?;
    Ok(result.into_pyobject(py)?)
}

// #[pyfunction]
// fn parse_json_file(py: Python<'_>, path: &str) -> PyResult<Bound<'_, PyAny>> {
//     todo!();
// }
// 
// #[pyfunction]
// #[pyo3(signature = (obj, indent=None))]
// fn dumps(obj: &Bound<PyAny>, indent: Option<usize>) -> PyResult<String> {
//     todo!();
// }

// this is not exposed to python (note the lack of #[pyfunction] rust property)
// fn py_to_json_value(obj: &Bound<PyAny>) -> PyResult<JsonValue> {
//     todo!();
// }

// module registration - This is essential for Python to be able to use the 
// functions annotated with #[pyfunction]
// #[pymodule]
// fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(parse_json, m)?)?;
//     m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
//     m.add_function(wrap_pyfunction!(dumps, m)?)?;
//     Ok(())
// }