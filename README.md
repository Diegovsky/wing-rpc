<img src="https://raw.githubusercontent.com/Diegovsky/wing-rpc/master/resources/logo.svg?sanitize=true" alt="Wing RPC Logo" width="250" align="right">

# Wing RPC
This repository contains a bunch of subprojects that all colectively implement `wing`, a lightweight RPC project/format to facilitate the communication
between two or more processes.

To use Wing, you need two things:
  - The wing compiler, `wingc`
    - It generates the relevant glue classes/enums/structs and keeps them in sync between languages for you.
  - The wing library for you programming language
    - Implements the low-level bits to send data over the wire

Wing is intended to be used by native/desktop applications with the following objectives:
  - Backend/frontend architecture.
  - Not being a bloated mess like Electron and other webview based tech.
  - It uses a pattern of sending/receiving messages.
  - The backend is written in Rust.

Note that it works with whatever compiler backends are implemented right now, which is:
  - Python 3
  - Rust

So, you *can* write a python-python app no problem.

Disclaimer: This doc is a work in-progress, but I plan to elaborate further later on.

## Directories
  - `examples/`: Contains some examples/showcases of the project
  - `pywing-rpc`: Python library that implements the necessary glue to "speak" wing-rpc
  - `wing-rpc`: Rust crate that does the same thing
  - `wingc`: Rust crate that implements all of the compiler's internals, including language plugins
  - `wingc-cli`: Rust crate that thinly provides a CLI to the `wingc` crate. **You'll need to use this to generate language glue code**
