use std::time::SystemTime;
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
            if salt_cache.check_salt(&utils::key_to_string(public_key), salt) {return false}
            salt_cache.enter_salt(&utils::key_to_string(public_key), salt);
            
            // Verify evidence
            let exp_evidence = format!("JOIN\0{}\0{}\0{}\0{}\0", username, utils::key_to_string(public_key), timestamp, salt);
            if crypt::verify(&public_key, &evidence, &exp_evidence) {
                return false
            }

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

/// A basic cache for storing and making salts
pub struct SaltCache {
    salt_counter: u32, // mmm very random indeed
    stored_salts: Vec<String>,
}   // TODO: Remove salts > 2 minutes old
impl SaltCache {
    pub fn new() -> Self {
        SaltCache { 
            salt_counter: random::<u16>() as u32,
            stored_salts: vec![]
        }
    }

    /// Gives you an unique salt
    /// 
    /// Uses an internal counter to give you a number,
    /// since it doesn't need to be random.
    /// It is still unique though.
    pub fn get_salt(&mut self ) -> String {
        self.salt_counter += 1;
        if self.salt_counter > 99999999 {self.salt_counter = 0}
        self.salt_counter.to_string()
    }

    /// Enters a salt into the cache
    pub fn enter_salt(&mut self, key: &str, salt: &str) {
        let f_salt = format!("{}\0{}", key, salt);
        self.stored_salts.push(f_salt)
    }

    /// Checks whether or not a salt has been entered
    /// 
    /// Returns True if a salt IS in the cache.
    /// False if salt IS NOT in the cache.
    pub fn check_salt(&mut self, key: &str, salt: &str) -> bool {
        let f_salt = format!("{}\0{}", key, salt);
        let stat = self.stored_salts.contains(&f_salt);
        return stat // FIXME: mmm yes very efficient
    }               // (it's not) ((make it))
}