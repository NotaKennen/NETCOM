
/*
A whole bunch of useful constants.
Basically setting for the entire program.
*/

// Protocol 
pub const ACCEPTABLE_EPOCH: u64 = 120;
pub const SALT_LIFETIME: std::time::Duration = std::time::Duration::from_secs(120);

// Graphics
pub const ENABLE_TUI: bool = true;

// Data lookup 
// All paths listed are "DATA_DIRECTORY/path"
pub const DATA_DIRECTORY: &str = "data/";
pub const KEY_PATH: &str = "key";
pub const USER_PATH: &str = "username";
pub const LISTENER_PATH: &str = "listener";
pub const INITIALHOST_PATH: &str = "connection";

// waow cool colors
pub const RESET: &str = "\x1b[0m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";

// Networking
pub const CONN_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(750);
pub const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);
pub const REDIR_LIMIT: i64 = 10;