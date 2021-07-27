pub const DEBUG_ECHO_DEVICE_NAME_LABEL: &str = "DEBUG_ECHO_DESCRIPTION";

use std::env;

fn main() {
    let device_name = match env::var(DEBUG_ECHO_DEVICE_NAME_LABEL) {
        Ok(n) => n,
        Err(_e) => "debugEcho".to_string(),
    };
    println!("Pod is running and using debugEcho device named: {}", device_name);
    loop {

    }
}
