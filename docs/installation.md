# Installation

## Requirements

- Python
- `tmux` installed and available in `PATH`

## Install from PyPI

```bash
pip install pypetmux
```

### Build from source
#### Requirements

- _rust installed to path_ install from [rustup](https://rustup.rs)
- _maturin_: `cargo install maturin` or `pip install maturin`

```bash
git clone https://github.com/rotleaf/pypetmux
cd pypetmux
maturin build --release
```