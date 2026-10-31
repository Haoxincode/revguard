use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use revguard_core::{inject_dir, strip_dir, verify_dir, Config};
use std::path::PathBuf;

fn cfg_from_opt(config_json: Option<String>) -> PyResult<Config> {
    match config_json {
        Some(s) => Config::from_json(&s).map_err(|e| PyRuntimeError::new_err(e.to_string())),
        None => Ok(Config::default()),
    }
}

#[pyfunction]
#[pyo3(signature = (dir, config_json=None))]
fn inject(dir: String, config_json: Option<String>) -> PyResult<String> {
    let cfg = cfg_from_opt(config_json)?;
    let m = inject_dir(PathBuf::from(dir).as_path(), &cfg)
        .map_err(PyRuntimeError::new_err)?;
    Ok(m.to_json_pretty())
}

#[pyfunction]
fn strip(dir: String) -> PyResult<String> {
    let r = strip_dir(PathBuf::from(dir).as_path()).map_err(PyRuntimeError::new_err)?;
    Ok(format!(
        "{{\"stripped\":{},\"hash_ok\":{},\"mismatches\":{}}}",
        r.stripped,
        r.hash_ok,
        r.hash_mismatch.len()
    ))
}

#[pyfunction]
fn verify(dir: String) -> PyResult<String> {
    let r = verify_dir(PathBuf::from(dir).as_path()).map_err(PyRuntimeError::new_err)?;
    Ok(format!(
        "{{\"checked\":{},\"ok\":{},\"missing\":{},\"mismatches\":{},\"no_manifest\":{}}}",
        r.checked,
        r.ok,
        r.missing_injection.len(),
        r.hash_mismatch.len(),
        r.no_manifest
    ))
}

#[pymodule]
fn revguard(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(inject, m)?)?;
    m.add_function(wrap_pyfunction!(strip, m)?)?;
    m.add_function(wrap_pyfunction!(verify, m)?)?;
    m.add("__version__", "0.1.0")?;
    Ok(())
}
