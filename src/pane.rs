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
                    "tmux clear failed: {}",
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
                "send keys failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Ok(true)
    }

    /// select this pane
    #[getter]
    pub fn select(&self) -> PyResult<bool> {
        let output = self
            .cmd()
            .args(["select-pane", "-t", self.target()])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "select pane failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Ok(true)
    }

    /// clear a panes history
    #[getter]
    pub fn clear(&self) -> PyResult<bool> {
        let output = self
            .cmd()
            .args(["clear-history", "-t", self.target()])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux clear-history failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Ok(true)
    }

    /// capture a panes content
    /// Args: trim - remove trailing whitespaces from the capture output
    #[pyo3(signature=(trim=false))]
    pub fn capture(&self, trim: bool) -> Option<String> {
        let output = self
            .cmd()
            .args(["capture-pane", "-p", "-t", self.target()])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if output.status.success() {
            let mut text = String::from_utf8_lossy(&output.stdout).to_string();

            if trim {
                let lines: Vec<&str> = text.lines().collect();
                let start = lines
                    .iter()
                    .position(|l| !l.trim().is_empty())
                    .unwrap_or(lines.len());
                let end = lines
                    .iter()
                    .rposition(|l| !l.trim().is_empty())
                    .map(|i| i + 1)
                    .unwrap_or(start);

                text = lines[start..end].join("\n");
            }

            Some(text)
        } else {
            None
        }
    }

    /// Kill this pane.
    pub fn kill(&self) -> PyResult<bool> {
        let output = self
            .cmd()
            .args(["kill-pane", "-t", self.target()])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux kill-pane failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Ok(true)
    }

    /// Resize the pane.
    ///
    /// Args:
    ///     direction: One of "left", "right", "up", or "down".
    ///     amount: Number of cells to resize by.
    ///
    #[pyo3(signature = (direction, amount))]
    pub fn resize(&self, direction: String, amount: u32) -> PyResult<bool> {
        let flag = match direction.as_str() {
            "left" => "-L",
            "right" => "-R",
            "up" => "-U",
            "down" => "-D",
            _ => {
                return Err(PyRuntimeError::new_err(
                    "direction must be one of: left, right, up, down",
                ));
            }
        };

        let output = self
            .cmd()
            .args([
                "resize-pane",
                flag,
                "-t",
                self.target(),
                &amount.to_string(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux resize-pane failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Ok(true)
    }

    /// Split this pane and return the new pane.
    ///
    /// Args:
    ///     horizontal: Split horizontally if True, vertically otherwise.
    ///     command: Optional command to run in the new pane.
    ///
    #[pyo3(signature = (horizontal = false, command = None, keep=true))]
    pub fn split(&self, horizontal: bool, command: Option<String>, keep: bool) -> PyResult<Pane> {
        let flag = if horizontal { "-h" } else { "-v" };

        let mut cmd = self.cmd();
        cmd.args([
            "split-window",
            flag,
            "-t",
            self.target(),
            "-P",
            "-F",
            "#{pane_index}|#{pane_id}|#{pane_title}",
        ]);
        let wrapped_command;
        if let Some(ref c) = command {
            if keep {
                wrapped_command = format!("sh -lc '{}; exec \"$SHELL\"'", c.replace('\'', r"'\''"));
                cmd.arg(&wrapped_command);
            } else {
                cmd.arg(c);
            }
        }

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux split-window failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = raw.trim().split('|').collect();

        if parts.len() != 3 {
            return Err(PyRuntimeError::new_err(format!(
                "unexpected split-window output: {:?}",
                raw.trim()
            )));
        }

        let pane_index = parts[0].parse::<u32>().map_err(|e| {
            PyRuntimeError::new_err(format!("bad pane_index {:?}: {}", parts[0], e))
        })?;

        Ok(Pane {
            session_name: self.session_name.clone(),
            window_index: self.window_index,
            pane_index,
            pane_id: parts[1].to_string(),
            title: parts[2].to_string(),
            socket: self.socket.clone(),
        })
    }

    /// Rename the pane title.
    ///
    /// Args:
    ///     title: New pane title.
    pub fn rename_title(&mut self, title: String) -> PyResult<bool> {
        let output = self
            .cmd()
            .args(["select-pane", "-T", &title, "-t", self.target()])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux select-pane -T failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        self.title = title;
        Ok(true)
    }

    /// Respawn the pane.
    ///
    /// Args:
    ///     command: Optional command to run after respawning.
    ///     kill: Kill the existing pane process first.
    #[pyo3(signature = (command = None, kill = false))]
    pub fn respawn(&self, command: Option<String>, kill: bool) -> PyResult<bool> {
        let mut cmd = self.cmd();
        cmd.args(["respawn-pane", "-t", self.target()]);

        if kill {
            cmd.arg("-k");
        }

        if let Some(ref c) = command {
            cmd.arg(c);
        }

        let output = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux respawn-pane failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Ok(true)
    }

    /// Whether the pane is alive.
    #[getter]
    pub fn is_alive(&self) -> PyResult<bool> {
        let output = self
            .cmd()
            .args(["display-message", "-p", "-t", self.target(), "#{pane_dead}"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux display-message failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim() == "0")
    }

    /// get this shells current command/program
    /// will be the shell name if no program is active
    #[getter]
    pub fn current_command(&self) -> Option<String> {
        let output = self
            .cmd()
            .args([
                "display-message",
                "-p",
                "-t",
                self.target(),
                "#{pane_current_command}",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() { None } else { Some(value) }
        } else {
            None
        }
    }

    /// get this panes current running command with its arguments
    /// will be the shell name if no program is active
    #[getter]
    pub fn current_commandline(&self) -> Option<String> {
        let pid_output = self
            .cmd()
            .args(["display-message", "-p", "-t", self.target(), "#{pane_pid}"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if !pid_output.status.success() {
            return None;
        }

        let pid = String::from_utf8_lossy(&pid_output.stdout)
            .trim()
            .to_string();
        if pid.is_empty() {
            return None;
        }

        let output = Command::new("ps")
            .args(["-p", &pid, "-o", "args="])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;

        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() { None } else { Some(value) }
        } else {
            None
        }
    }

    /// Change this pane to bash.
    ///
    /// Args:
    ///     respawn: If True, kill the current pane process and respawn it as bash.
    ///              If False, run `exec bash` inside the pane.
    ///
    #[pyo3(signature = (respawn = false))]
    pub fn bash_shell(&self, respawn: bool) -> PyResult<bool> {
        use pyo3::exceptions::PyRuntimeError;
        use std::process::Stdio;

        let target = self.target();

        let output = if respawn {
            self.cmd()
                .args(["respawn-pane", "-k", "-t", target, "bash"])
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output()
        } else {
            self.cmd()
                .args(["send-keys", "-t", target, "exec bash", "Enter"])
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output()
        }
        .map_err(|e| PyRuntimeError::new_err(format!("failed to run tmux: {e}")))?;

        if !output.status.success() {
            return Err(PyRuntimeError::new_err(format!(
                "tmux change shell failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Ok(true)
    }

    /// Repeat the previous command in this pane's bash shell.
    ///
    /// Returns:
    ///     True on success.
    ///
    /// Raises:
    ///     RuntimeError: If the pane is not currently running bash.
    pub fn run_previous_command(&self) -> PyResult<bool> {
        match self.current_command() {
            Some(cmd) if cmd == "bash" => self.send_keys("fc -s".to_string(), true, false),
            Some(cmd) => Err(PyRuntimeError::new_err(format!(
                "previous_command requires bash, current command is {:?}",
                cmd
            ))),
            None => Err(PyRuntimeError::new_err(
                "could not determine current command",
            )),
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
