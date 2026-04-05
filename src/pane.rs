use pyo3::{exceptions::PyRuntimeError, prelude::*};
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

    #[pyo3(signature = (keys, enter = false, clear_first=false))]
    pub fn send_keys(&self, keys: String, enter: bool, clear_first: bool) -> PyResult<bool> {
        let mut cmd = self.cmd();
        let target = self.target();
        if clear_first {
            let output = self
                .cmd()
                .args(["send-keys", "-t", target, "C-l"])
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

            if !output.status.success() {
                return Err(PyRuntimeError::new_err(format!(
                    "tmux send-keys clear failed: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                )));
            }
        }
        cmd.args(["send-keys", "-t", target, &keys]);
        if enter {
            cmd.arg("Enter");
        }

        let output = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux send-keys failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Ok(true)
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
