# pypetmux

`pypetmux` provides Python bindings for working with `tmux`.

It currently exposes the `Server` API.

## Quick example

```python
from pypetmux import Server

server = Server()

if not server.is_running:
    server.start()

if not server.has_session("work"):
    server.new_session("work")

print(server.sessions)
```