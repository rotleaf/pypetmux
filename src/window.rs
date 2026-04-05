use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::process::{Command, ExitStatus, Stdio};

use crate::pane::Pane;

#[gen_stub_pyclass]
#[pyclass(get_all)]
pub struct Window {
    pub session_name: String,
    pub index: u32,
    pub socket: Option<String>,
    pub name: String,
}

#[gen_stub_pyclass]
#[pyclass(get_all)]
pub struct WindowMetadata {
    pub index: u32,
    pub name: String,
    pub active: bool,
    pub layout: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub flags: String,
}

#[gen_stub_pymethods]
#[pymethods]
impl WindowMetadata {
    fn __repr__(&self) -> String {
        format!(
            "WindowMetadata(index={}, name={:?}, active={}, layout={:?}, width={:?}, height={:?}, flags={:?})",
            self.index, self.name, self.active, self.layout, self.width, self.height, self.flags
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl Window {
    fn target(&self) -> String {
        format!("{}:{}", self.session_name, self.index)
    }

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
impl Window {
    #[new]
    pub fn new(session_name: String, index: u32, socket: Option<String>, name: String) -> Self {
        Self {
            session_name,
            index,
            socket,
            name,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Window(session_name={:?}, index={}, name={:?}, socket={:?})",
            self.session_name, self.index, self.name, self.socket
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// kill window
    pub fn kill(&self) -> bool {
        let target = self.target();
        self.cmd()
            .args(["kill-window", "-t", &target])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s: ExitStatus| s.success())
            .unwrap_or(false)
    }

    /// select a window if not selected
    #[getter]
    pub fn select(&self) -> bool {
        let target = self.target();
        self.cmd()
            .args(["select-window", "-t", &target])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s: ExitStatus| s.success())
            .unwrap_or(false)
    }

    #[setter]
    pub fn set_name(&mut self, new_name: String) -> PyResult<()> {
        let target = self.target();
        let ok = self
            .cmd()
            .args(["rename-window", "-t", &target, &new_name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s: ExitStatus| s.success())
            .unwrap_or(false);

        if ok {
            self.name = new_name;
            Ok(())
        } else {
            Err(PyValueError::new_err(format!(
                "failed to rename window to '{}'",
                new_name
            )))
        }
    }

    /// list panes in a window
    #[getter]
    pub fn panes(&self) -> Vec<Pane> {
        let target = self.target();

        let output = self
            .cmd()
            .args([
                "list-panes",
                "-t",
                &target,
                "-F",
                "#{pane_index}|#{pane_id}|#{pane_title}",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        match output {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.trim().split('|').collect();
                    if parts.len() != 3 {
                        return None;
                    }

                    let pane_index = parts[0].parse::<u32>().ok()?;
                    let pane_id = parts[1].to_string();
                    let title = parts[2].to_string();

                    Some(Pane {
                        session_name: self.session_name.clone(),
                        window_index: self.index,
                        pane_index,
                        pane_id,
                        title,
                        socket: self.socket.clone(),
                    })
                })
                .collect(),
            _ => vec![],
        }
    }

    /// get window metadata
    #[getter]
    pub fn metadata(&self) -> PyResult<WindowMetadata> {
        let target = self.target();

        let fmt = "#{window_index}|#{window_name}|#{window_active}|#{window_layout}|#{window_width}|#{window_height}|#{window_flags}";

        let output = self
            .cmd()
            .args(["display-message", "-p", "-t", &target, fmt])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        let line = raw.trim();
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() != 7 {
            return Err(PyRuntimeError::new_err(format!(
                "unexpected tmux window metadata output: {:?}",
                line
            )));
        }

        let index = parts[0].parse::<u32>().map_err(|e| {
            PyRuntimeError::new_err(format!("bad window_index {:?}: {}", parts[0], e))
        })?;

        let width = if parts[4].trim().is_empty() {
            None
        } else {
            Some(parts[4].trim().parse::<u32>().map_err(|e| {
                PyRuntimeError::new_err(format!("bad window_width {:?}: {}", parts[4], e))
            })?)
        };

        let height = if parts[5].trim().is_empty() {
            None
        } else {
            Some(parts[5].trim().parse::<u32>().map_err(|e| {
                PyRuntimeError::new_err(format!("bad window_height {:?}: {}", parts[5], e))
            })?)
        };

        Ok(WindowMetadata {
            index,
            name: parts[1].to_string(),
            active: parts[2] == "1",
            layout: parts[3].to_string(),
            width,
            height,
            flags: parts[6].to_string(),
        })
    }

    /// Move to the next tmux window in this session and return it.
    ///
    /// Returns:
    ///     The newly selected Window, or None if the operation fails.
    #[getter]
    pub fn next(&self) -> Option<Window> {
        let session_target = self.session_name.as_str();

        let output = self
            .cmd()
            .args(["next-window", "-t", session_target])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let output = self
            .cmd()
            .args([
                "display-message",
                "-p",
                "-t",
                session_target,
                "#{window_index}|#{window_name}",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = raw.trim().split('|').collect();

        if parts.len() != 2 {
            return None;
        }

        let index = parts[0].parse::<u32>().ok()?;
        let name = parts[1].to_string();

        Some(Window {
            session_name: self.session_name.clone(),
            index,
            name,
            socket: self.socket.clone(),
        })
    }

    #[getter]
    pub fn previous(&self) -> Option<Window> {
        let session_target = self.session_name.as_str();

        let output = self
            .cmd()
            .args(["previous-window", "-t", session_target])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let output = self
            .cmd()
            .args([
                "display-message",
                "-p",
                "-t",
                session_target,
                "#{window_index}|#{window_name}",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = raw.trim().split('|').collect();

        if parts.len() != 2 {
            return None;
        }

        let index = parts[0].parse::<u32>().ok()?;
        let name = parts[1].to_string();

        Some(Window {
            session_name: self.session_name.clone(),
            index,
            name,
            socket: self.socket.clone(),
        })
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Window>()?;
    // pane::register(m)?;
    Ok(())
}
