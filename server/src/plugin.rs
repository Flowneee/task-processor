use std::{collections::BTreeMap, ffi::OsStr, fs::read_dir, path::Path};

use libc::c_char;
use libloading::{Library, Result as LibResult};

use interface::*;

type PluginMainFn = unsafe extern "C" fn(*const c_char) -> *const c_char;
type PluginNameFn = unsafe extern "C" fn() -> *const c_char;

pub(crate) struct Plugin {
    lib: Library,
}

impl Plugin {
    pub(crate) fn new<P: AsRef<OsStr>>(path: P) -> LibResult<Self> {
        let lib = Library::new(path)?;
        // Try to load plugin functions
        unsafe { lib.get::<PluginMainFn>(b"plugin_main")? };
        unsafe { lib.get::<PluginNameFn>(b"plugin_name")? };
        Ok(Self { lib })
    }

    pub(crate) fn call(&self, data: &str) -> Result<serde_json::Value, String> {
        // Load plugin_main. Call of `.unwrap()` is safe since we already loaded
        // function in constructor.
        let plugin_main = unsafe { self.lib.get::<PluginMainFn>(b"plugin_main").unwrap() };

        // Convert data to Rust string and then to C string.
        let input_data = str_to_c_str(&data);
        let output = unsafe { plugin_main(input_data) };
        c_str_to_json(output)
    }

    fn name(&self) -> Result<String, String> {
        // Load plugin_name. Call of `.unwrap()` is safe since we already loaded
        // function in constructor.
        let plugin_name = unsafe { self.lib.get::<PluginNameFn>(b"plugin_name").unwrap() };
        c_str_to_str(unsafe { plugin_name() })
    }
}

pub(crate) fn load_plugins<P: AsRef<OsStr>>(dir: P) -> std::io::Result<BTreeMap<String, Plugin>> {
    Ok(read_dir(Path::new(&dir))?
        .filter_map(|x| x.ok())
        .filter_map(|x| {
            let plugin = Plugin::new(x.path()).ok()?;
            let name = plugin.name().ok()?;
            Some((name, plugin))
        })
        .collect())
}
