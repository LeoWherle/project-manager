# Project Manager CLI

Project Manager CLI (`pm`) is a command-line tool to manage your projects efficiently. It allows you to add, remove, list, and open projects directly from the terminal.

## Features

- **Open Projects**: Quickly open your projects in your favorite editor.
- **Add Projects**: Add new projects to your configuration.
- **Remove Projects**: Remove projects from your configuration.
- **List Projects**: List all your projects with optional verbose output.
- **Edit Configuration**: Edit the projects configuration file directly.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [pkg-config](https://www.freedesktop.org/wiki/Software/pkg-config/)
- [libssl-dev](https://www.openssl.org/)

**Ubuntu**:
```sh
sudo apt install pkg-config libssl-dev
```

### Installation
To install the `pm` binary, run the following command:

```sh
cargo build --release
```
It will prompt you with sudo password to install the binary in `/usr/local/bin` directory.

### Bash Completion
```sh
sudo cp pm_completion.sh /etc/bash_completion.d/pm
```

## Usage

```sh
pm [COMMAND]
```

### Commands

- `open <project_name>`: Open a project by its name.
- `add <directory>`: Add a new project from the specified directory.
- `remove <directory>`: Remove a project from the specified directory.
- `add-source`: Add a new source for a project.
- `list [--verbose]`: List all projects. Use `--verbose` for detailed output.
- `edit`: Edit the projects configuration file.

### Examples

- **Open a project**:
  ```sh
  pm open my_project
  ```

- **Add a project**:
  ```sh
  pm add /path/to/my_project
  ```

## Configuration

The configuration file is located at [`~/.config/project-manager/projects.json`]("~/.config/project-manager/projects.json"). It stores information about your projects, including their names, paths, descriptions, and sources.
