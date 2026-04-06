# pypetmux

`pypetmux` is a Python wrapper around **tmux**, allowing you to programmatically manage sessions, windows, and panes from Python.

It provides a simple object-oriented API on top of tmux commands.

---

## Features

- Start and manage tmux servers
- Create and list sessions
- Navigate windows and panes
- Query metadata (size, layout, etc.)
- Thin wrapper over tmux (no magic, predictable behavior)

---

## Installation

### From PyPI

```bash
pip install pypetmux
```

### Requirements

- `tmux` must be installed and available in your `PATH`

---

## Quick Example

```python
from pypetmux import Server

server = Server()

# ensure tmux is running
if not server.is_running:
    server.start()

# create a session if it doesn't exist
if not server.has_session("work"):
    session = server.new_session("work")

# list sessions
for session in server.sessions:
    print(session.name)
```

---

## How It Works

`pypetmux` is a thin wrapper over the `tmux` CLI.

Each object corresponds to a tmux concept:

| Python        | tmux equivalent |
|---------------|-----------------|
| `Server`      | tmux server     |
| `Session`     | tmux session    |
| `Window`      | tmux window     |
| `Pane`        | tmux pane       |

The library internally runs tmux commands and parses the output.

---

## Usage

### Server

```python
from pypetmux import Server

server = Server()

print(server.is_running)     # check if tmux is running
server.start()               # start server
server.kill()                # kill server
```

---

### Sessions

```python
session = server.new_session("demo")

print(session.name)

# rename session
session.name = "new-name"

# list windows
for window in session.windows:
    print(window.name)

# metadata
meta = session.metadata()
print(meta.width, meta.height)
```

---

### Windows

```python
window = session.windows[0]

# select window
window.select

# rename
window.name = "editor"

# navigate
next_window = window.next
prev_window = window.previous

# panes
for pane in window.panes:
    print(pane.title)

# metadata
print(window.metadata())
```

---

## Notes

- This library **does not replace tmux** — it wraps it
- Errors from tmux are surfaced as Python exceptions
- Behavior depends on your local tmux configuration

---

## Development
- _clone the repository, cd into it_
```bash
pip install maturin
maturin develop
```

---
