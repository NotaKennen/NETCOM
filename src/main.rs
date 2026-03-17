mod settings;
mod commands;
mod network;
mod crypt;
mod utils;
mod ui;

use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;
use std::fs::read_to_string;
use crate::network::{NetCommand, net_thread};
use crate::ui::UiMan;
use crate::settings::*;

fn main() {
    // Nonrelevant stuff
    let mut salt_cache = commands::SaltCache::new();

    // Data lookup (and listener)
    let username = read_to_string(format!("{}{}", DATA_DIRECTORY, USER_PATH)).unwrap();

    // Get keys and make a keyset
    let private_key: [u8; 32] = std::fs::read_to_string(format!("{}{}", DATA_DIRECTORY, KEY_PATH)).unwrap().as_bytes().try_into().unwrap();
    let public_key: [u8; 32] = crypt::get_public(&private_key); // ^ holy unwrap abomination

    // Base graphics setup
    // TODO: Convert UI to thread-based
    let mut uima = UiMan::new(ENABLE_TUI);

    // Start up networking thread
    let (nthread_send, net_recv) = mpsc::channel();
    let (net_send, nthread_recv) = mpsc::channel();
    thread::spawn(move || net_thread(nthread_recv, nthread_send));
    println!("{BLUE}[?] Network thread set up");

    // Extra prints
    if !ENABLE_TUI {println!("{BLUE}[?] TUI Disabled, using log prints{RESET}")}
    println!("{BLUE}[?] Ready{RESET}");

    // Main loop
    loop {
        
        // Handle incoming commands
        let e_cmd = net_recv.try_recv();
        if e_cmd.is_ok() {
            let cmd = e_cmd.unwrap();
            if commands::verify(&cmd, &mut salt_cache) {
                if !ENABLE_TUI {dbg!(&cmd);}
                match cmd {
                    NetCommand::Message { ref username, public_key: _, timestamp: _, ref content, ..} => {
                        let strcmd = format!("{}: {}", username, content);
                        uima.new_message(strcmd);
                    }
                    _ => {} // TODO: Display other commands than messages
                }
                let _ = net_send.send(cmd); // Relay command
            }
        }

        // Graphics and UI stuff
        if ENABLE_TUI {
            for out_msg in uima.get_outgoing() {
                let tags: Vec<&str> = out_msg.1.split(" ").collect();
                let command = commands::message(&username, public_key, private_key, &out_msg.0, utils::upgrade_vec(tags), &salt_cache.get_salt());
                let _ = net_send.send(command);
            }
            let stat = uima.refresh();
            if !stat {break}
        }
        
        // Or sleep in case it blows and no UI is set
        else {sleep(Duration::from_millis(500))};
    }

    // Send leave
    let leave = commands::leave(&username, public_key, private_key, &salt_cache.get_salt());
    let _ = net_send.send(leave);

}
