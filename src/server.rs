use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use std::{
    env,
    path::PathBuf,
    process::Stdio,
    process::{Command, ExitStatus},
};

use crate::session::Session;

#[gen_stub_pyclass]
#[pyclass]
pub struct Server {
    pub socket: Option<String>,
}

impl Server {
    fn cmd(&self) -> Command {
        let mut cmd = Command::new("tmux");
        if let Some(ref socket) = self.socket {
            cmd.args(["-S", socket]);
        }
        cmd
    }

    fn default_socket_path() -> Option<String> {
        let uid = current_uid()?;

        let base = env::var_os("TMUX_TMPDIR")
            .or_else(|| env::var_os("TMPDIR"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));

        Some(
            base.join(format!("tmux-{uid}"))
                .join("default")
                .to_string_lossy()
                .into_owned(),
        )
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl Server {
    #[new]
    #[pyo3(signature = (socket = None))]
    pub fn new(socket: Option<String>) -> Self {
        Self { socket }
    }

    /// create a new session
    /// returns a Session Object
    // TODO: make name optional
    pub fn new_session(&self, name: String) -> PyResult<Session> {
        let output = self
            .cmd()
            .args(["new-session", "-d", "-s", &name])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "new session failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(Session {
            name,
            socket: self.socket.clone(),
        })
    }

    /// list sessions in this server
    #[getter]
    pub fn sessions(&self) -> Vec<Session> {
        let output = self
            .cmd()
            .args(["list-sessions", "-F", "#{session_name}"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        match output {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(|l| Session {
                    name: l.to_string(),
                    socket: self.socket.clone(),
                })
                .collect(),
            _ => vec![],
        }
    }

    /// check if a tmux server is running
    #[getter]
    pub fn is_running(&self) -> bool {
        self.cmd()
            .arg("list-sessions")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s: ExitStatus| s.success())
            .unwrap_or(false)
    }

    /// kill a tmux server
    pub fn kill(&self) -> bool {
        self.cmd()
            .arg("kill-server")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s: ExitStatus| s.success())
            .unwrap_or(false)
    }

    /// check if the server contains a session
    pub fn has_session(&self, name: String) -> bool {
        self.cmd()
            .args(["has-session", "-t", &name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s: ExitStatus| s.success())
            .unwrap_or(false)
    }

    /// start a tmux server
    pub fn start(&self) -> bool {
        self.cmd()
            .arg("start-server")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s: ExitStatus| s.success())
            .unwrap_or(false)
    }

    /// get the tmux socket path for this server
    ///
    /// if a socket was explicitly provided, that is returned.
    /// otherwise, the default tmux socket path is returned.
    #[getter]
    pub fn current_socket(&self) -> Option<String> {
        if let Some(socket) = &self.socket {
            return Some(socket.clone());
        }

        let output = self
            .cmd()
            .args(["display-message", "-p", "#{socket_path}"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok();

        if let Some(output) = output
            && output.status.success()
        {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }

        Self::default_socket_path()
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Server>()?;
    // session::register(m)?;
    Ok(())
}

#[cfg(unix)]
fn current_uid() -> Option<u32> {
    use std::process::Command;

    let output = Command::new("id")
        .arg("-u")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u32>()
        .ok()
}

#[cfg(not(unix))]
fn current_uid() -> Option<u32> {
    None
}
