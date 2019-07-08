# Task processor

## Overview

This application provide server for "cloud" processing. All processing is done via "plugins", which should be connected to server.

### API

 - `GET /tasks`: get list of registered tasks;
 - `POST /tasks`: register new task. Format (JSON): `{ "module": "<plugin_name>", "data": <task_data> }`;
 - `GET /plugins`: get list of loaded plugins;
 - `POST /plugins/reload`: reload plugins.
 
### Start

After building project start `server` binary. It supports CLI parameters, take a look at them with `$ ./server --help`.

If you want to start server after `cargo build` on default address `127.0.0.1:8000`, just do this:

``` bash
cd target/release
RUST_LOG=info ./server
```

### Configuration file

Configuration file (which is loaded, if path is passed via CLI) have format (JSON):

``` json
{
    "plugins_path": "./plugins",
    "bind_address": "0.0.0.0:80"
}
```

### Project structure

 - `server`: server itself;
 - `interface`: provide types and functions, common for both plugins and server;
 - `plugins/`: contain plugins (each plugin is a crate).

## Plugins

Each plugin is a dynamic library, which is export 2 functions:

``` c++
const char * plugin_main(const char * data);
const char * plugin_name();
```

or in Rust:

``` rust
#[no_mangle]
pub extern "C" fn plugin_main(data: *const libc::c_char) -> *const libc::c_char {...}
#[no_mangle]
pub extern "C" fn plugin_name() -> *const libc::c_char {...}
```

where `plugin_name` is return plugin name, and `plugin_main` is actual entry point to plugin.

Both input and output strings of `plugin_main` is a JSON documents.

Functions, described above, should not be implemented manually, instead you should call `plugin!` macro from `interface` crate (take a look at any provided plugin for example).

### Default plugins

#### sum

Adds all arguments. Input data should be and array of positive integers (`u64`): `[1,2,3,4]`. Request example: `{ "module": "sum", "data": [1,2,3,4,5] }`.

#### mul

Multiplies all arguments. Input data should be and array of positive integers (`u64`): `[1,2,3,4]`. Request example: `{ "module": "mul", "data": [1,2,3,4,5] }`.

#### fibonacci

Calculate Fibonacci number. Input data should be positive integer (`u64`). Request example: `{ "module": "fibonacci", "data": 1000 }`.
