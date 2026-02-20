use pyo3::prelude::*;
use std::{
    env::current_dir,
    net::UdpSocket,
};


#[pyfunction]
fn self_check(host: String, port: String) -> PyResult<()> {
    let addr = format!("{}:{}", host, port);
    let message = format!("machine_tracer loaded at {}", current_dir()?.display());
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to bind ({})", e)))?;
    socket.send_to(message.as_bytes(), &addr).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Send error ({})", e)))?;
    Ok(())
}

#[pymodule]
fn machine_tracer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(self_check, m)?)?;
    Ok(())
}
