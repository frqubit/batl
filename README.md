# Battalion

Battalion is a CLI tool for managing codebase relationships. It uses a simple heirarchy of **repositories** and **workspaces** to link codebases together when needed, and keep them separate when not.

## Installation

```bash
cargo install --git https://github.com/frqubit/batl batl
batl setup

# (optional) Install batlas
batl fetch battalion.batlas
batl battalion.batlas:build
batl battalion.batlas:install
```

## Usage

```bash
# Create a new repository
batl init prototypes.awesome-project

# Create a library
batl init prototypes.awesome-library

# cd into the workspace
cd $(batl which prototypes.awesome-project)

# ...or if you use batlas with VSCode...
batlas prototypes.awesome-project code %!

# create a link while in directory of workspace
mkdir lib
batl link prototypes.awesome-library lib/awesome

# Start building!
```
