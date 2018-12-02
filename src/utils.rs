use std::env;

/// Checks for debug envorinment variable `ENIGMAP_DEBUG` and returns debug state
pub fn check_debug() -> bool {
    // get env variable as a Option<OsString>
    let var = env::var_os("ENIGMAP_DEBUG");
    match var {
        // get value and parse it
        Some(val) => {
            let val = val.to_string_lossy().trim().parse();
            // check for error while parsing
            let val = match val {
                Ok(some) => some,
                Err(_) => 0,
            };
            match val {
                1 => true,
                _ => false,
            }
        }
        // if non existent, set false
        None => false,
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! debug_println {
    ( $( $x:expr ),* ) => {
        {
            use crate::utils::check_debug;
            if check_debug() {
                println!(
                    $(
                        $x,
                    )*
                );
            }
        }
    };
}
