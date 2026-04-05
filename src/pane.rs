use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::process::{Command, Stdio};

#[gen_stub_pyclass]
#[pyclass(get_all)]
pub struct Pane {
    pub session_name: String,
    pub window_index: u32,
    pub pane_index: u32,
    pub pane_id: String,
    pub title: String,
    pub socket: Option<String>,
}

#[gen_stub_pymethods]
#[pymethods]
impl Pane {
    #[new]
    pub fn new(
        session_name: String,
        window_index: u32,
        pane_index: u32,
        pane_id: String,
        title: String,
        socket: Option<String>,
    ) -> Self {
        Self {
            session_name,
            window_index,
            pane_index,
            pane_id,
            title,
            socket,
        }
    }

    #[getter]
    pub fn capture(&self) -> Option<String> {
        let output = self
            .cmd()
            .args(["capture-pane", "-p", "-t", self.target()])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            None
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Pane(session_name={:?}, window_index={}, pane_index={}, pane_id={:?}, title={:?}, socket={:?})",
            self.session_name,
            self.window_index,
            self.pane_index,
            self.pane_id,
            self.title,
            self.socket
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl Pane {
    fn cmd(&self) -> Command {
        let mut cmd = Command::new("tmux");
        if let Some(ref socket) = self.socket {
            cmd.args(["-S", socket]);
        }
        cmd
    }

    fn target(&self) -> &str {
        &self.pane_id
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pane>()?;
    Ok(())
}
