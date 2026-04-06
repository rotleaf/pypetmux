# pypetmux

Python bindings for interacting with tmux.

## Installation

```bash
pip install pypetmux
```

### example
```python
from pypetmux import Server

server = Server()
print(server.is_running)

if not server.has_session("demo"):
    session = server.new_session("demo")

for s in server.sessions:
    print(s.name)
```