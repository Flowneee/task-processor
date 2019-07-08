use std::ffi::{CStr, CString};

use libc::c_char;

pub use libc;

/// Convert JSON, serialized into C string to type, which can be deserialized from JSON.
pub fn c_str_to_json<T>(data: *const c_char) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let s = c_str_to_str(data)?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

/// Convert C string to Rust string.
pub fn c_str_to_str(s: *const c_char) -> Result<String, String> {
    let c_str = unsafe { CStr::from_ptr(s) };
    c_str
        .to_str()
        .map(|x| x.to_owned())
        .map_err(|e| e.to_string())
}

/// Convert Rust string to Result, serialized in C string.
pub fn str_to_c_str(s: &str) -> *const c_char {
    let checked_vec = s
        .as_bytes()
        .iter()
        .filter(|x| **x != 0)
        .cloned()
        .collect::<Vec<u8>>();
    // This call of `from_vec_unchecked` should be safe since input was filtered above.
    unsafe { CString::from_vec_unchecked(checked_vec) }.into_raw()
}

/// Serialize data into JSON, which is stored inside C string.
pub fn json_to_c_str<T>(data: &T) -> *const c_char
where
    T: serde::Serialize,
{
    serde_json::to_string(data)
        .map(|x| str_to_c_str(&x))
        .unwrap_or_else(|e| str_to_c_str(&e.to_string()))
}

/// Generate entry point to plugin.
#[macro_export]
macro_rules! plugin {
    ( name: $n:expr; main: $i:ident ) => {
        #[no_mangle]
        pub extern "C" fn plugin_main(
            data: *const $crate::libc::c_char,
        ) -> *const $crate::libc::c_char {
            $crate::c_str_to_json(data)
                .map(|x| $crate::json_to_c_str(&$i(x)))
                .unwrap_or_else(|e| $crate::str_to_c_str(&e))
        }

        #[no_mangle]
        pub extern "C" fn plugin_name() -> *const $crate::libc::c_char {
            $crate::str_to_c_str($n)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_c_str() {
        let data = "{ \"key\": \"value\" }";
        let parsed = c_str_to_json::<serde_json::Value>(str_to_c_str(data)).unwrap();
        assert_eq!(parsed["key"], "value")
    }

    #[test]
    fn test_str_with_null_to_c_str() {
        let data = "{ \"k\0\0ey\": \0\"value\" }";
        let parsed = c_str_to_json::<serde_json::Value>(str_to_c_str(data)).unwrap();
        assert_eq!(parsed["key"], "value")
    }
}
