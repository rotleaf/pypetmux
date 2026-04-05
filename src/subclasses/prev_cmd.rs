// to enable run or read functionality for a pane previous command,
// current usage: pane.run_previous_command()
// this file changes that to pane.previous_command.run/show
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use std::process::{Command, Stdio};

#[gen_stub_pyclass]
#[pyclass]
pub struct LastCommand {
    #[pyo3(get)]
    pub pane_id: String,

    #[pyo3(get)]
    pub socket: Option<String>,
}

impl LastCommand {
    pub fn new(pane_id: String, socket: Option<String>) -> Self {
        Self { pane_id, socket }
    }

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

    fn tmux_ok(&self, args: &[&str]) -> PyResult<()> {
        let output = self
            .cmd()
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(PyRuntimeError::new_err(format!(
                "tmux command failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )))
        }
    }

    fn tmux_capture(&self, args: &[&str]) -> PyResult<String> {
        let output = self
            .cmd()
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(PyRuntimeError::new_err(format!(
                "tmux command failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )))
        }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl LastCommand {
    fn __repr__(&self) -> String {
        format!(
            "LastCommand(pane_id={:?}, socket={:?})",
            self.pane_id, self.socket
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// Read the last bash command from this pane.
    ///
    /// Returns:
    ///     The last command as a string.
    ///
    /// WARNING: using this function might mess up your pane history
    pub fn read(&self) -> PyResult<String> {
        let target = self.target();
        let probe = r#"printf '__PYPETMUX__%s\n' "$(fc -ln -1)""#;

        self.tmux_ok(&["send-keys", "-t", target, probe, "Enter"])?;

        let text = self.tmux_capture(&["capture-pane", "-p", "-t", target])?;

        let line = text
            .lines()
            .rev()
            .find(|l| l.trim_start().starts_with("__PYPETMUX__"))
            .ok_or_else(|| PyRuntimeError::new_err("could not read last command"))?;

        Ok(line
            .trim_start()
            .trim_start_matches("__PYPETMUX__")
            .trim()
            .to_string())
    }

    /// Run the last bash command from this pane.
    ///
    /// Returns:
    ///     True on success.
    pub fn run(&self) -> PyResult<bool> {
        self.tmux_ok(&["send-keys", "-t", self.target(), "fc -s", "Enter"])?;
        Ok(true)
    }
}
