use pyo3::prelude::*;
use pyo3::exceptions::{PyOSError, PyRuntimeError};
use std::net::UdpSocket;
use std::sync::{Arc, OnceLock};

const BIND_ADDR: &str = "0.0.0.0:0";

// Create global singleton instances
static HOST: OnceLock<String> = OnceLock::new();
static PORT: OnceLock<u16> = OnceLock::new();
static SENDER: OnceLock<Arc<UdpSocket>> = OnceLock::new();

#[pyfunction]
fn init(host: String, port: u16) {
    HOST.set(host).unwrap();
    PORT.set(port).unwrap();
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

#[pymodule]
fn machine_tracer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(create_udp_sender, m)?)?;
    m.add_function(wrap_pyfunction!(send_udp_message, m)?)?;
    Ok(())
}
