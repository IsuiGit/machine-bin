use pyo3::prelude::*;
use pyo3::exceptions::{PyOSError, PyRuntimeError};
use pyo3::types::PyCFunction;
use std::net::UdpSocket;
use std::sync::{Arc, OnceLock};

const BIND_ADDR: &str = "0.0.0.0:0";

// Create global singleton instances
static HOST: OnceLock<String> = OnceLock::new();
static PORT: OnceLock<u16> = OnceLock::new();
static NAME: OnceLock<String> = OnceLock::new();
static SENDER: OnceLock<Arc<UdpSocket>> = OnceLock::new();

#[pyfunction]
fn init(host: String, port: u16, name: String) {
    HOST.set(host).unwrap();
    PORT.set(port).unwrap();
    NAME.set(name).unwrap();
}

#[pyfunction]
fn create_udp_sender() -> PyResult<()> {
    let socket = Arc::new(UdpSocket::bind(BIND_ADDR).map_err(|e| PyOSError::new_err(format!("Failed to bind: {}", e)))?);
    SENDER.set(socket).map_err(|_| PyRuntimeError::new_err("Sender already initialized"))?;
    Ok(())
}

#[pyfunction]
fn send_udp_message(message: &str) -> PyResult<()> {
    let sender = SENDER.get().ok_or_else(|| PyRuntimeError::new_err("Sender not initialized. Call create_udp_sender() first."))?;
    let addr = (
        HOST.get().ok_or_else(|| PyRuntimeError::new_err("Host not initialized. Call init() first."))?.clone(),
        PORT.get().ok_or_else(|| PyRuntimeError::new_err("Port not initialized. Call init() first."))?.clone()
    );
    sender.send_to(message.as_bytes(), &addr).map_err(|e| PyOSError::new_err(format!("Send error: {}", e)))?;
    Ok(())
}

#[pyfunction]
fn trace_callback(
    py: Python<'_>,
    frame: &Bound<'_, PyAny>,
    event: &Bound<'_, PyAny>,
    _arg: &Bound<'_, PyAny>,
) -> PyResult<Py<PyCFunction>> {
    // Set callback anyway
    let callback = wrap_pyfunction!(trace_callback, py)?;
    // Check if event exist
    let Ok(event_str) = event.extract::<String>() else { return Ok(callback.into()); };
    if event_str == "call" {
        // Get frame code object or return callback
        let Ok(f_code) = frame.getattr("f_code") else { return Ok(callback.into()) };
        // Get co_name (function name) or return callback
        let Ok(co_name) = f_code.getattr("co_name") else { return Ok(callback.into()) };
        // Get function name as string or return callback
        let Ok(func_name) = co_name.extract::<String>() else { return Ok(callback.into()) };
        // Get function line or return callback
        let Ok(line_no) = frame.getattr("f_lineno").and_then(|l| l.extract::<i32>()) else { return Ok(callback.into()); };
        // Get filename or return callback
        let Ok(filename) = f_code.getattr("co_filename").and_then(|f| f.extract::<String>()) else {return Ok(callback.into()); };
        if filename != NAME.get().ok_or_else(|| PyRuntimeError::new_err("Name not initialized. Call init() first."))?.clone() { return Ok(callback.into()); };
        // Form message
        let message = format!("{} | {} | {}", filename, line_no, func_name);
        let _ = send_udp_message(&message);
    }
    // Return callback
    Ok(callback.into())
}

#[pymodule]
fn machine_tracer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(create_udp_sender, m)?)?;
    m.add_function(wrap_pyfunction!(send_udp_message, m)?)?;
    m.add_function(wrap_pyfunction!(trace_callback, m)?)?;
    Ok(())
}
