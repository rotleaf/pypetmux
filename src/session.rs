use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::process::{Command, Stdio};

use crate::window::Window;

#[gen_stub_pyclass]
#[pyclass(get_all)]
pub struct SessionMetadata {
    pub id: String,
    pub name: String,
    pub created: u64,
    pub attached: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub windows: u32,
}

#[gen_stub_pymethods]
#[pymethods]
impl SessionMetadata {
    fn __repr__(&self) -> String {
        format!(
            "SessionMetadata(id={:?}, name={:?}, created={}, attached={}, width={:?}, height={:?}, windows={})",
            self.id, self.name, self.created, self.attached, self.width, self.height, self.windows
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

#[gen_stub_pyclass]
#[pyclass(get_all)]
pub struct Session {
    pub name: String,
    pub socket: Option<String>,
}

impl Session {
    fn cmd(&self) -> Command {
        let mut cmd = Command::new("tmux");

        if let Some(ref socket) = self.socket {
            cmd.args(["-S", socket]);
        }

        cmd
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Session {
    #[new]
    #[pyo3(signature = (name, socket = None))]
    pub fn new(name: String, socket: Option<String>) -> Self {
        Self { name, socket }
    }

    fn __repr__(&self) -> String {
        format!("Session(name={:?}, socket={:?})", self.name, self.socket)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
    /// get windows in a session
    pub fn windows(&self) -> Vec<Window> {
        let target = self.name.clone();
        let output = self
            .cmd()
            .args([
                "list-windows",
                "-t",
                &target,
                "-F",
                "#{window_index}|#{window_name}",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        match output {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|l| {
                    let parts: Vec<&str> = l.trim().split('|').collect();
                    if parts.len() != 2 {
                        return None;
                    }
                    let index = parts[0].parse::<u32>().ok()?;
                    let name = parts[1].to_string();
                    Some(Window {
                        session_name: self.name.clone(),
                        index,
                        name,
                        socket: self.socket.clone(),
                    })
                })
                .collect(),
            _ => vec![],
        }
    }

    /// rename a session
    pub fn rename(&mut self, new_name: String) -> PyResult<bool> {
        let output = self
            .cmd()
            .args(["rename-session", "-t", &self.name, &new_name])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err("rename session failed: {e}"));
        }

        self.name = new_name;
        Ok(true)
    }

    /// get a sessions metadata
    pub fn metadata(&self) -> PyResult<SessionMetadata> {
        let fmt = "#{session_id}|#{session_name}|#{session_created}|#{session_attached}|#{session_width}|#{session_height}|#{session_windows}";

        let output = self
            .cmd()
            .args(["list-sessions", "-F", fmt])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout);

        for line in raw.lines() {
            let parts: Vec<&str> = line.trim().split('|').collect();
            if parts.len() != 7 {
                continue;
            }

            if parts[1] == self.name {
                return Ok(SessionMetadata {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    created: parts[2].parse().map_err(|e| {
                        PyRuntimeError::new_err(format!(
                            "bad session_created {:?}: {}",
                            parts[2], e
                        ))
                    })?,
                    attached: parts[3] != "0",
                    width: parts[4].trim().parse::<u32>().ok(),
                    height: parts[5].trim().parse::<u32>().ok(),
                    windows: parts[6].parse().map_err(|e| {
                        PyRuntimeError::new_err(format!(
                            "bad session_windows {:?}: {}",
                            parts[6], e
                        ))
                    })?,
                });
            }
        }

        Err(PyRuntimeError::new_err(format!(
            "session {:?} not found",
            self.name
        )))
    }

    #[getter]
    pub fn exists(&self) -> bool {
        self.cmd()
            .args(["has-session", "-t", &self.name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    pub fn kill(&self) -> bool {
        self.cmd()
            .args(["kill-session", "-t", &self.name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Session>()?;
    m.add_class::<SessionMetadata>()?;
    Ok(())
}
