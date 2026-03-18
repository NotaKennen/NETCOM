use mansikka::{
    AnsiController, char::*, clear_screen, libc::{
        disable_raw_mode, 
        enable_raw_mode, 
        terminal_size
    }, move_cursor, print_loc, string::ImprovedString, wait_for_in
};

/* 
UI Module for NETCOM

Contains basic UI things, mainly message displays.
Also handles message from user -> main (msg in)

Unlike Networking, this is NOT designed to be replaceable.
If reworked, main will also need reworks.
*/

// TODO: Toggle for network activity
// Probably ctrl+d or something
// Might also wanna include sys activity
// Basically just dump the log 

pub struct UiMan {
    // Setup stuff
    org_terminal: Option<libc::termios>,// Copy of the original terminal for restoring
    controller: AnsiController,         // Controller for Mansikka
    chbuf: [u8; 8],                     // char buffer for Mansikka

    // Actual chat stuff
    out_buf: Vec<(String, String)>,     // Outgoing messages buffer
    input_selector: i64,                // Which one is selected
    msg_input: String,                  // Message being input
    tag_input: String,                  // Tags being input

    // Display stuff
    message_buffer: Vec<String>,        // Messages to display
}
impl UiMan {
    /// Creates a new UI Manager, also sets up the ANSI display
    pub fn new(enable_raw: bool) -> Self {
        // Setup
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let controller = AnsiController::new(stdin, stdout);
        let org = if enable_raw {Some(enable_raw_mode())} else {None};

        // Ret
        UiMan {
            org_terminal: org,
            controller,
            chbuf: [0; 8],

            out_buf: vec![],
            input_selector: 0,
            msg_input: String::new(), // Selector 0
            tag_input: String::new(), // Selector 1

            message_buffer: vec![],

        }
    }

    /// Insert a message to the buffer to be displayed
    pub fn new_message(&mut self, text: String) {
        self.message_buffer.insert(0, text);
    }

    /// Refreshes the screen, renders content
    /// 
    /// Returns `True/False` based on whether or not the UI has 
    /// received a close signal (reversed)
    pub fn refresh(&mut self) -> bool {
        
        // Take input and handle special commands
        wait_for_in(&mut self.controller, &mut self.chbuf);
        if self.chbuf == K_CTRL_C {self.restore(); return false}
        else if self.chbuf == K_NONE {} // bwomp
        else if self.chbuf == K_ARROW_DOWN {
            self.input_selector = (self.input_selector + 1).clamp(0, 1)
        }
        else if self.chbuf == K_ARROW_UP {
            self.input_selector = (self.input_selector - 1).clamp(0, 1)
        }
        else if self.chbuf == K_ENTER {
            if self.msg_input == "" {}
            else {
                self.out_buf.push((
                    self.msg_input.trim().replace("\0", "").to_string().clone(),
                    self.tag_input.trim().replace("\0", "").to_string().clone(),
                ));
                self.msg_input = String::new();
            }
        }
        else if self.chbuf == [127, 0, 0, 0, 0, 0, 0, 0] {
            match self.input_selector {
                0 => {self.msg_input.pop();}
                1 => {self.tag_input.pop();}
                _ => {}
            }
        }
        
        // Char writing
        else {
            // Make the char buffer into a char if it is that
            // and dump into our buffer
            let c = self.chbuf[0] as char;
            if !c.is_control() {
                match self.input_selector {
                    0 => self.msg_input.push(c),
                    1 => self.tag_input.push(c),
                    _ => {}
                }
            }
        }

        // Screen render setup
        let screen_size = terminal_size().expect("libc blew up");
        let bottom_location = (screen_size.0.into(), 0);
        
        // Clear before rendering
        clear_screen(&mut self.controller);

        // Display tag input
        let tag_prefix = if self.input_selector == 1 {">"} else {" "};
        let display_tags = &self.tag_input[self.tag_input.len().saturating_sub((screen_size.1 - 3).into())..];
        print_loc(
            format!("{} {}", tag_prefix, display_tags).into(), 
            bottom_location, 
            &mut self.controller
        );

        // Display message input
        let msg_prefix = if self.input_selector == 0 {">"} else {" "}; 
        let display_message = &self.msg_input[self.msg_input.len().saturating_sub((screen_size.1 - 3).into())..];
        print_loc( // ^ Truncate (displayed) message to screen length 
            format!("{} {}", msg_prefix, display_message).into(), 
            (bottom_location.0 - 1, 0), 
            &mut self.controller
        );

        // Divider
        // TODO: Status display in divider
        print_loc(
            "-".repeat(screen_size.1 as usize).into(),
            (bottom_location.0 - 2, 0),
            &mut self.controller, 
        );

        // Display messages
        let mut offset: i64 = 3; // 3 because messages start at bottom-3
        for (idx, message) in self.message_buffer.iter().enumerate() {
            let lines = (message.len() as f64 / screen_size.1 as f64).floor() as i64;
            offset += lines;
            let c_location: i64 = bottom_location.0 as i64 - idx as i64 - offset;
            if c_location < 0 {continue}
            print_loc(
                ImprovedString::from(message.as_ref()),
                (c_location as u32, 0),
                &mut self.controller,
            )
        }

        // Set cursor to its proper location
        let location: (u32, u32) = match self.input_selector {
            0 => {(bottom_location.0-1, (self.msg_input.len()+3) as u32 )}
            1 => {(bottom_location.0-0, (self.tag_input.len()+3) as u32 )}
            _ => {(0, 0)}
        };
        move_cursor(location.0, location.1, &mut self.controller);

        return true
    }

    /// Fetches all outgoing messages
    /// 
    /// Returns Vec of (message, tags)
    pub fn get_outgoing(&mut self) -> Vec<(String, String)> {
        let out = self.out_buf.clone();
        self.out_buf = vec![];          // ^ FIXME: Clone :(
        return out;                     // Might not even need fixing though
    }

    /// Restores the terminal to it's original state
    pub fn restore(&mut self) {
        clear_screen(&mut self.controller);
        move_cursor(0, 0, &mut self.controller);
        if self.org_terminal.is_some() {
            disable_raw_mode(self.org_terminal.unwrap());
        }
    }
}