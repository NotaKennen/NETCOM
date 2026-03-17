use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use rand::random;

use crate::{crypt, network::NetCommand, utils};
use crate::settings::ACCEPTABLE_EPOCH;

/*
Extension module to network.rs

Should include a base "API" for making NetCommands.
Basically, each Command has a function here,
and you can use the function to create the command.
Epoch and evidence should be generated automatically.
(And any other possible fields)

Additionally should be able to do base verification of commands.
*/

/// Generates a JOIN command based on given arguments
/// 
/// Sets the epoch as the generation time, so don't hold onto it.
pub fn join(username: &str, public_key: [u8; 32], private_key: [u8; 32], salt: &str) -> NetCommand {
    
    // Get rest of the fields
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time is mean :(").as_secs();
    let str_evidence = &format!("JOIN\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(&public_key), timestamp, salt);
    let evidence = crypt::sign(&private_key, str_evidence);

    NetCommand::Join { 
        username: username.to_string(), 
        public_key, 
        timestamp: timestamp, 
        salt: salt.to_string(), 
        evidence: evidence,
    }
}

/// Generates a LEAVE command based on given arguments
/// 
/// Sets the epoch as the generation time, so don't hold onto it.
pub fn leave(username: &str, public_key: [u8; 32], private_key: [u8; 32], salt: &str) -> NetCommand {

    // Get rest of the fields
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time is mean :(").as_secs();
    let str_evidence = &format!("LEAVE\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(&public_key), timestamp, salt);
    let evidence = crypt::sign(&private_key, str_evidence);

    NetCommand::Leave { 
        username: username.to_string(), 
        public_key, 
        timestamp: timestamp, 
        salt: salt.to_string(), 
        evidence: evidence,
    }
}

/// Generates a MSG command based on given arguments
/// 
/// Sets the epoch as the generation time, so don't hold onto it.
pub fn message(username: &str, public_key: [u8; 32], private_key: [u8; 32], content: &str, tags: Vec<String>, salt: &str) -> NetCommand {

    // Get rest of the fields
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time is mean :(").as_secs();
    let str_evidence = &format!("MSG\0{}\0{}\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(&public_key), timestamp, content, utils::tag_to_string(tags.to_vec()), salt);
    let evidence = crypt::sign(&private_key, str_evidence);

    NetCommand::Message { 
        username: username.to_string(), 
        public_key, 
        timestamp: timestamp, 
        content: content.to_string(), 
        tags: tags, 
        salt: salt.to_string(), 
        evidence: evidence, 
    }
}


/// Verifies that a command is valid
/// 
/// Returns false on invalid, true on valid
pub fn verify(command: &NetCommand, salt_cache: &mut SaltCache) -> bool {
    match command {
        NetCommand::Join { username, public_key, timestamp, salt, evidence } => {
            // Verify epoch
            let cur_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time is mean :(").as_secs();
            if &(cur_time - ACCEPTABLE_EPOCH) > timestamp {
                return false
            }

            // Verify salt
            if salt_cache.check_salt(&utils::key_to_string(public_key), salt) {
                return false
            }
            
            // Verify evidence
            let exp_evidence = format!("JOIN\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(public_key), timestamp, salt);
            if crypt::verify(&public_key, &evidence, &exp_evidence) {
                return false
            }
            
            // Cache salt
            salt_cache.enter_salt(&utils::key_to_string(public_key), salt);
            // Salt is cached here so that it has to pass the evidence check first
            // So that no one falsifies salts

            return true
        }
        NetCommand::Leave { username, public_key, timestamp, salt, evidence } => {
            // Verify epoch
            let cur_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time is mean :(").as_secs();
            if &(cur_time - ACCEPTABLE_EPOCH) > timestamp {
                return false
            }

            // Verify salt
            if salt_cache.check_salt(&utils::key_to_string(public_key), salt) {return false}
            salt_cache.enter_salt(&utils::key_to_string(public_key), salt);
            
            // Verify evidence
            let exp_evidence = format!("LEAVE\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(public_key), timestamp, salt);
            let verified = crypt::verify(&public_key, &evidence, &exp_evidence);
            if !verified {return false}

            return true
        }
        NetCommand::Message { username, public_key, timestamp, content, tags, salt, evidence } => {
            // Verify epoch
            let cur_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time is mean :(").as_secs();
            if &(cur_time - ACCEPTABLE_EPOCH) > timestamp {
                return false
            }

            // Verify salt
            if salt_cache.check_salt(&utils::key_to_string(public_key), salt) {return false}
            salt_cache.enter_salt(&utils::key_to_string(public_key), salt);

            // Verify evidence
            let exp_evidence = format!("MSG\0{}\0{}\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(public_key), timestamp, content, utils::tag_to_string(tags.to_vec()), salt);
            let verified = crypt::verify(&public_key, &evidence, &exp_evidence);
            if !verified {return false}

            return true
        }
    }
}

const SALT_LIFETIME: Duration = Duration::from_secs(120);

/// A base structure for creating and managing salts
pub struct SaltCache {
    salt_counter: u32,
    stored_salts: HashMap<(String, String), Instant>,
}
impl SaltCache {
    pub fn new() -> Self {
        SaltCache {
            salt_counter: random::<u16>() as u32,
            stored_salts: HashMap::new(),
        }
    }

    /// Gives you an unique (but non-random) salt
    pub fn get_salt(&mut self) -> String {
        self.salt_counter = (self.salt_counter + 1) % 100_000_000;
        self.salt_counter.to_string()
    }

    /// cleans timed out salts from the cache
    fn cleanup(&mut self) {
        let now = Instant::now();
        self.stored_salts.retain(|_, t| now.duration_since(*t) < SALT_LIFETIME);
    }

    /// Enters a salt into the cache
    pub fn enter_salt(&mut self, key: &str, salt: &str) {
        self.cleanup();
        self.stored_salts.insert(
            (key.to_string(), salt.to_string()),
            Instant::now(),
        );
    }

    /// Checks whether or not a salt is in the cache
    pub fn check_salt(&mut self, key: &str, salt: &str) -> bool {
        self.cleanup();
        self.stored_salts.contains_key(&(key.to_string(), salt.to_string()))
    }
}