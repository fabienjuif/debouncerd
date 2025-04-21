# debouncerd

> A simple daemon to debounce command executions — useful when you want to delay and group repeated command triggers (e.g., in response to frequent file changes or events).

## Features

- Daemon + client architecture
- Debounce logic based on customizable timeout
- Optional grouping via IDs
- Custom working directory support

## Installation

```bash
git clone git@github.com:fabienjuif/debouncerd.git
cargo install --path ./debouncerd
```

## Usage

### Daemon

```bash
debouncerd
```

### Client

```
A debounce wrapper that runs a command with a timeout and optional settings

Usage: debouncerctl [OPTIONS] <TIMEOUT> <CMD>

Arguments:
  <TIMEOUT>  Debounce timeout in milliseconds
  <CMD>      Command to execute

Options:
      --id <ID>    Optional identifier for the debounce group (useful for distinguishing runs)
      --pwd <PWD>  Optional present working directory to run the command from [default: /home/fabien/perso/debouncerd]
  -h, --help       Print help
  -V, --version    Print version
```

#### Example

```bash
debouncerctl 500 "echo 'Hello, world!'"
```

This ensures that if the same command is triggered multiple times within 500ms, it will only be executed once.
