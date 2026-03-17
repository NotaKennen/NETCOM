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
    // Get args if passed, and apply
    let nogui = std::env::args().any(|arg| arg == "--nogui");
    let tui_status = !nogui && ENABLE_TUI;

    // Collect stuff
    let mut salt_cache = commands::SaltCache::new();
    let mut uima = UiMan::new(tui_status);
    let username = read_to_string(format!("{}{}", DATA_DIRECTORY, USER_PATH)).unwrap();
    let private_key: [u8; 32] = std::fs::read_to_string(format!("{}{}", DATA_DIRECTORY, KEY_PATH)).unwrap().as_bytes().try_into().unwrap();
    let public_key: [u8; 32] = crypt::get_public(&private_key); // ^ holy unwrap abomination
    
    // Start up networking thread
    let (nthread_send, net_recv) = mpsc::channel();
    let (net_send, nthread_recv) = mpsc::channel();
    thread::spawn(move || net_thread(nthread_recv, nthread_send));
    println!("{BLUE}[?] Network thread set up");

    // Extra prints
    if !tui_status {println!("{BLUE}[?] TUI Disabled, using log prints{RESET}")}
    println!("{BLUE}[?] Ready{RESET}");

    // Main loop
    loop {
        
        // Handle incoming commands
        let e_cmd = net_recv.try_recv();
        if e_cmd.is_ok() {
            let cmd = e_cmd.unwrap();
            if !tui_status {dbg!(&cmd);}
            if commands::verify(&cmd, &mut salt_cache) {
                match cmd {
                    NetCommand::Message { ref username, ref content, ..} => {
                        let strcmd = format!("{}: {}", username, content);
                        uima.new_message(strcmd);
                    }
                    NetCommand::Join { ref username, ..} => {
                        let strcmd = format!("JOIN : {}", username);
                        uima.new_message(strcmd);
                    }
                    NetCommand::Leave { ref username, ..} => {
                        let strcmd = format!("LEAVE : {}", username);
                        uima.new_message(strcmd);
                    }
                }
                let _ = net_send.send(cmd); // Relay command
            }
        }

        // Graphics and UI stuff
        if tui_status {
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
        // TODO: add a LEAVE command for nogui
    }

    // Send leave
    let leave = commands::leave(&username, public_key, private_key, &salt_cache.get_salt());
    let _ = net_send.send(leave);
    sleep(Duration::from_secs(2)); // Wait so that networkthread can send it
}
