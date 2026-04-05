use pyo3::prelude::*;

pub mod pane;
pub mod server;
pub mod session;
pub mod window;

pyo3_stub_gen::define_stub_info_gatherer!(stub_info);

#[pymodule]
pub fn pypetmux(m: &Bound<'_, PyModule>) -> PyResult<()> {
    server::register(m)?;
    session::register(m)?;
    window::register(m)?;
    pane::register(m)?;
    Ok(())
}
