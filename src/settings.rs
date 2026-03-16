
/*
A whole bunch of useful constants.
Basically setting for the entire program.
*/

//
// Configurable
//

// Protocol 
pub const ACCEPTABLE_EPOCH: u64 = 120;

// Graphics
pub const ENABLE_TUI: bool = true;

// (Configurable) Networking
pub const INITIAL_HOST: &str = "127.0.0.1:6500";
pub const LISTENER_ADDR: &str = "0.0.0.0:6500";

// TODO: Move the networking config back to files
// Solve this alongside the username issue

//
// Non-Configurable
// (Unless you want to change something anyway)
//

// Data lookup 
// All paths listed are "DATA_DIRECTORY/path"
pub const DATA_DIRECTORY: &str = "data/";
pub const KEY_PATH: &str = "key";
pub const USER_PATH: &str = "username";

// waow cool colors
pub const RESET: &str = "\x1b[0m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";

// Networking
pub const CONN_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(2500);
pub const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);
pub const REDIR_LIMIT: i64 = 10;