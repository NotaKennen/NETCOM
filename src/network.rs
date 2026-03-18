use std::{
    fs::read_to_string, io::{Read, Write}, net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream}, str::FromStr, sync::mpsc, thread::sleep
};

use rand::random;
use crate::{commands, utils};
use crate::settings::*;

/*
Networking module for NETCOM

Just like the other modules, the client should work 
even if this module was replaced by something else, 
basically it should just expose the base API for the client

All communication between this module and main should be in NetCommands.
NetCommands should be able to represent any command listed in SPEC §5.1.2
*/

// TODO: Rework module description
// We use a networking thread now, so it has changed
// Note for any readers, yes the description (above) is outdated

// TODO: Rework networking module
// The thread stuff messed everything up

/// Keeps reading a stream until end, returns bytes as `Vec<u8>`
fn dynamic_read(stream: &mut impl Read) -> Vec<u8> {
    // FIXME: Rework dynamic_read()
    // Might break?
    let mut buf = [0u8; 512];
    let mut out = Vec::new();
    loop {
        let n = match stream.read(&mut buf) {
            Ok(0) => break, // connection closed
            Ok(n) => n,
            Err(_) => break,
        };
        out.extend_from_slice(&buf[..n]);
        if n < buf.len() {
            break;
        }
    }
    return out
}

/// The main networking thread
/// 
/// Includes setup and command returns (via provided channel)
/// 
/// Function also handles 
pub fn net_thread(in_channel: mpsc::Receiver<NetCommand>, out_channel: mpsc::Sender<NetCommand>) {
    
    // TODO: Add an extra channel for logs

    // Data lookup
    let listener_addr = read_to_string(format!("{}{}", DATA_DIRECTORY, LISTENER_PATH)).unwrap();
    let initial_host = read_to_string(format!("{}{}", DATA_DIRECTORY, INITIALHOST_PATH)).unwrap();
    let username = read_to_string(format!("{}{}", DATA_DIRECTORY, USER_PATH)).unwrap();
    let privkey: [u8; 32] = std::fs::read_to_string(format!("{}{}", DATA_DIRECTORY, KEY_PATH)).unwrap().as_bytes().try_into().unwrap();
    let pubkey: [u8; 32] = crate::crypt::get_public(&privkey);

    // Set up module
    let mut netm = NetworkMan::new();
    let connected = netm.connect(&initial_host, true).is_ok();
    let bound = netm.bind(&listener_addr).is_ok();
    if !bound && !connected  {panic!("{CYAN}[NET] Couldn't connect, couldn't bind{RESET}")}
    if bound && !connected {println!("{CYAN}[NET] Couldn't connect, running server mode{RESET}")}
    if !bound && connected {println!("{CYAN}[NET] Couldn't bind, running connect-only mode{RESET}")}

    // Send a JOIN
    let random = random::<u8>(); // We just use a small random as the extra salt for join
    let join = commands::join(&username, pubkey, privkey, &format!("join{}", random));
    netm.send_command(join);

    // Main network loop 
    loop {
        // FIXME: Ensure network health
        // Do this by checking stream amounts
        // Minimal implementation below, make this better
        if netm.get_stream_amount() == 0 && !bound {
            panic!("[NET] Not connected, Not bound!")
        }

        // Get outgoing messages
        let out_cmd = in_channel.try_recv();
        if out_cmd.is_ok() {
            // Receive messages
            let cmd = out_cmd.unwrap();
            let dead = netm.send_command(cmd.clone());
            for item in dead {
                if !ENABLE_TUI {println!("{CYAN}[NET] Stream index {} died{RESET}", item)}
            }

            // Kill thread on LEAVE
            match cmd {
                NetCommand::Leave {..} => {return}
                _ => {}
            }
        }

        // Accept incoming
        let _ = netm.accept_incoming(None);

        // Read for commands
        let commands = netm.get_commands();
        for command in commands {
            let _ = out_channel.send(command);
        }

        // Sleep for a bit to not use 10 petabytes of network
        sleep(READ_TIMEOUT); // Read timeout is a good sleep amount usually
    }    
}

pub struct NetworkMan {
    connections: Vec<Connection>,
    listener: Option<TcpListener>
}
impl NetworkMan {
    /// Makes a new network manager. 
    /// 
    /// To get started, you will need to bind a TCP Listener using `netman.bind()`,
    /// as well as connect to a host using `netman.connect()`.
    pub fn new() -> Self {
        return NetworkMan {connections: vec![], listener: None};
    }
    
    /// Connects to a host, automatically handles REDIR and READY,
    /// unless otherwise specified
    pub fn connect(&mut self, addr: &str, allow_redir: bool) -> Result<(), ()> {
        // waow cool entrypoint or something
        // could rework the redir amount to be reverse and use a limit
        // idk I'm lazy
        // won't even do it
        self.be_connect(addr, allow_redir, 0)
    }

    /// "Backend" connect, same as `connect()` but includes redir block
    fn be_connect(&mut self, addr: &str, allow_redir: bool, redir_amount: i64) -> Result<(), ()> {
        if redir_amount > REDIR_LIMIT {return Err(())} // Redir spam block
        let e_res = self.connect_once(addr);
        if e_res.is_err() {return Err(())}
        let res = e_res.unwrap();
        match res {
            HTCCommand::Ready(stream) => {
                self.connections.push(stream.into());
                return Ok(())
            }
            HTCCommand::Redir(newaddr) => {
                if !allow_redir {return Err(())}
                self.be_connect(&newaddr, allow_redir, redir_amount+1)
            }
            HTCCommand::Close => {
                return Err(())
            }
        }
    }

    /// Used for connecting without redirects,
    /// 
    /// returns HTCCommand to read for redirs or other commands
    fn connect_once(&self, addr: &str) -> Result<HTCCommand, ()> {

        // Format the IP
        let tmp: Vec<&str> = addr.split(":").collect();
        if tmp.len() < 2 {return Err(())}
        let (naddr, f_port) = (tmp[0], tmp[1]);
        let port = {
            let p = u16::from_str(f_port);
            if p.is_err() {return Err(())}
            else {p.unwrap()}
        };
        let sockaddr = {
            let s = Ipv4Addr::from_str(naddr);
            if s.is_err() {return Err(())}
            else {s.unwrap()}
        };
        
        // Connect stream
        let socketaddr = SocketAddr::new(std::net::IpAddr::V4(sockaddr), port);
        let e_stream = TcpStream::connect_timeout(&socketaddr, CONN_TIMEOUT);
        if e_stream.is_err() {return Err(())}
        let mut stream = e_stream.unwrap();
        let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
        
        // Get response
        let mut readbuf: [u8; 256] = [0; 256];
        let e_amount = stream.read(&mut readbuf);
        let amount = if e_amount.is_err() {return Err(())} else {e_amount.unwrap()};
        if amount == 0 {return Err(())}

        // Read buffer for response
        let resp = {
            let s = String::from_utf8(readbuf[..amount].to_vec());
            if s.is_err() {return Err(())} else {s.unwrap()}
        };
        match resp {
            rsp if rsp.starts_with("REDIR") => {
                let newaddr = rsp.strip_prefix("REDIR\0").unwrap();
                return Ok(HTCCommand::Redir(newaddr.to_string()));
            }
            rsp if rsp.starts_with("READY") => {
                return Ok(HTCCommand::Ready(stream));
            }
            rsp if rsp.starts_with("CLOSE") => {
                return Ok(HTCCommand::Close)
            }
            _ => {return Err(())}
        }
        
        
    }

    /// Gets all currently queued up commands from streams
    /// and returns them to you.
    /// 
    /// The function does not check for command validity.
    /// Use the `commands` module instead. 
    pub fn get_commands(&mut self) -> Vec<NetCommand> {
        let mut rettable: Vec<NetCommand> = vec![];
        for connection in &mut self.connections {
            // Read buffer
            let _ = connection.stream.set_read_timeout(Some(READ_TIMEOUT));
            let buf = dynamic_read(&mut connection.stream);
            let content = {
                let s = String::from_utf8(buf); 
                if s.is_err() {continue} else {s.unwrap()}
            };
            let mut f_split_content: Vec<String> = {
                let mut t = vec![];
                let vecc: Vec<&str> = content.split("\0").collect();
                for stri in vecc {
                    t.push(stri.to_string())
                }
                t
            };

            // Take in the buffer from the connection
            let mut totbuf = connection.buffer.clone();
            totbuf.append(&mut f_split_content);
            let mut split_content = totbuf;

            // Get the commands
            loop {
                if split_content.len() == 0 {break}
                let command_name = &split_content[0];
                if command_name == "" {
                    split_content.drain(..1);
                    continue;
                }
                match command_name.as_ref() {
                    "JOIN" => {
                        if split_content.len() < 6 {connection.buffer.append(&mut split_content); break}
                        
                        // Error prone formats
                        // If errors, we simply ignore the (whole) command
                        let f_public_key = {
                            let s = utils::string_to_key(&split_content[2]);
                            if s.is_err() {split_content.remove(0); continue} else {s.unwrap()}
                        };
                        let f_timestamp = {
                            let u = u64::from_str(&split_content[3]);
                            if u.is_err() {split_content.remove(0); continue} else {u.unwrap()}
                        }; 
                        
                        // Structure command
                        let command = NetCommand::Join { 
                            username: split_content[1].to_string(),
                            public_key: f_public_key,
                            timestamp: f_timestamp,
                            salt: split_content[4].to_string(),
                            evidence: split_content[5].to_string(),
                        };

                        // Dump it
                        rettable.push(command);
                        split_content.drain(..6);
                    }

                    "LEAVE" => {
                        if split_content.len() < 6 {connection.buffer.append(&mut split_content); break}
                        
                        // Error prone formats
                        let f_public_key = {
                            let s = utils::string_to_key(&split_content[2]);
                            if s.is_err() {split_content.remove(0); continue} else {s.unwrap()}
                        };
                        let f_timestamp = {
                            let u = u64::from_str(&split_content[3]);
                            if u.is_err() {split_content.remove(0); continue} else {u.unwrap()}
                        }; 
                        
                        // Command making
                        let command = NetCommand::Leave { 
                            username: split_content[1].to_string(),
                            public_key: f_public_key,
                            timestamp: f_timestamp,
                            salt: split_content[4].to_string(),
                            evidence: split_content[5].to_string(),
                        };

                        // Dump
                        rettable.push(command);
                        split_content.drain(..6);
                    }

                    "MSG" => {
                        if split_content.len() < 8 {connection.buffer.append(&mut split_content); break}

                        // Error prone formats
                        let f_public_key = {
                            let s = utils::string_to_key(&split_content[2]);
                            if s.is_err() {split_content.remove(0); continue} else {s.unwrap()}
                        };
                        let f_timestamp = {
                            let u = u64::from_str(&split_content[3]);
                            if u.is_err() {split_content.remove(0); continue} else {u.unwrap()}
                        }; 

                        // Command structuring
                        let command = NetCommand::Message { 
                            username: split_content[1].to_string(), 
                            public_key: f_public_key,
                            timestamp: f_timestamp,
                            content: split_content[4].to_string(),
                            tags: {
                                let mut tags: Vec<String> = vec![]; 
                                for tag in split_content[5].to_string().split(" ") {
                                    tags.push(tag.to_string())
                                } tags
                            },
                            salt: split_content[6].to_string(),
                            evidence: split_content[7].to_string(),
                        };

                        // dump
                        rettable.push(command);
                        split_content.drain(..8);
                    }
                    _ => {split_content.remove(0); continue}
                }
            }
        }
        return rettable
    }

    /// Sends a command to all connected streams. Also clears out dead streams.
    /// 
    /// Returns the index(es) of cleared streams.
    pub fn send_command(&mut self, command: NetCommand) -> Vec<usize> {

        // Send out commands
        let mut index = 0;
        let mut removable: Vec<usize> = vec![];
        for connection in &mut self.connections {
            let stat = connection.stream.write_all(&command.to_buf());
            if stat.is_err() {
                removable.push(index);
            }
            index += 1;
        }

        // Remove dead streams
        removable.reverse();
        for item in &removable {
            if self.connections.len() <= *item {continue}
            self.connections.remove(*item);
        }
        return removable;
    }

    /// Takes care of any incoming connections
    /// 
    /// Redirects the incoming connection to specified address,
    /// if not specified, accepts them to this address
    pub fn accept_incoming(&mut self, redirect_address: Option<&str>) -> Result<usize, ()> {
        if self.listener.is_none() {return Err(())}
        let listener = self.listener.as_ref().unwrap();

        // Take in new stream
        let mut new_streams = 0;
        listener.set_nonblocking(true).expect("TCP Listener is mean :(");
        for e_stream in listener.incoming() {
            if e_stream.is_err() {break}
            let mut stream = e_stream.unwrap();
            let _ = stream.set_write_timeout(Some(READ_TIMEOUT));

            // Redirect if needed
            if redirect_address.is_some() {
                // Unwrap checked :)
                let _ = stream.write(format!("REDIR\0{}\0", redirect_address.unwrap()).as_bytes());
                continue;
            }

            // Dump stream into memory
            let _ = stream.write(b"READY\0");
            self.connections.push(stream.into());
            new_streams += 1;

        }

        return Ok(new_streams);
    }

    /// Tries to bind a listener so that you may be connected to
    pub fn bind(&mut self, address: &str) -> Result<(), ()> {
        let listener = {
            let l = TcpListener::bind(address);
            if l.is_err() {
                None
            } else {Some(l.unwrap())}
        };
        let status = listener.is_some();
        self.listener = listener;
        if status {return Ok(())} else {Err(())}
    }

    /// Gets the amount of streams currently active
    pub fn get_stream_amount(&self) -> usize {
        return self.connections.len()
    }
}

/// A struct representing the Commands mentioned in `SPEC §5.1.2`
#[derive(Clone)]
#[derive(Debug)]
pub enum NetCommand {
    Join {
        username: String,
        public_key: [u8; 32],
        timestamp: u64,
        salt: String,
        evidence: String,
    },
    Leave {
        username: String,
        public_key: [u8; 32],
        timestamp: u64,
        salt: String,
        evidence: String,
    },
    Message {
        username: String,
        public_key: [u8; 32],
        timestamp: u64,
        content: String,
        tags: Vec<String>,
        salt: String,
        evidence: String,
    },
}
impl NetCommand {
    pub fn to_buf(&self) -> Vec<u8> {
        match self {
            NetCommand::Join { username, public_key, timestamp, salt, evidence} => {
                let cmd = format!("JOIN\0{}\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(public_key), timestamp, salt, evidence);
                return cmd.as_bytes().to_vec();
            }
            NetCommand::Leave { username, public_key, timestamp, salt, evidence} => {
                let cmd = format!("LEAVE\0{}\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(public_key), timestamp, salt, evidence);
                return cmd.as_bytes().to_vec();
            }
            NetCommand::Message { username, public_key, timestamp, content, tags, salt, evidence} => {
                let cmd = format!("MSG\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(public_key), timestamp, content, utils::tag_to_string(tags.to_vec()), salt, evidence);
                return cmd.as_bytes().to_vec();
            }
        }
    }
}

/// Host to Client commands (`SPEC §5.1.3`)
/// 
/// Separated from NetCommands so you don't
/// have to handle everything there for this
pub enum HTCCommand {
    Redir(String),
    Ready(TcpStream),
    Close,
}

/// A struct representing a TCP connection, 
/// which includes some nice extra commands
pub struct Connection {
    stream: TcpStream,
    buffer: Vec<String>,
} 
impl From<TcpStream> for Connection {
    fn from(value: TcpStream) -> Self {
        Connection { 
            stream: value, 
            buffer: vec![] 
        }
    }
}