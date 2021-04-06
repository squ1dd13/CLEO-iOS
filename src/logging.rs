use cached::proc_macro::cached;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::net;

#[derive(Clone, Copy, Serialize, Deserialize)]
enum MessageType {
    Normal,
    Error,
    Warning,
    Important,
}

#[derive(Serialize, Deserialize)]
struct Message {
    group: String,
    msg_type: MessageType,
    string: String,
    process: String,
    time: String,
}

#[cached]
fn get_proc_name() -> String {
    let cur_exec = std::env::current_exe();

    // Get the name of the current process, or "???" if we can't get the name.
    if let Ok(path) = cur_exec {
        String::from(
            path.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("???"))
                .to_str()
                .unwrap_or("???"),
        )
    } else {
        String::from("???")
    }
}

impl Message {
    fn pack(&self) -> Option<Vec<u8>> {
        let serialized = bincode::serialize::<Message>(self).ok();

        if serialized.is_none() {
            return serialized;
        }

        let serialized = serialized.unwrap();

        let mut len_bytes = Vec::from(u32::to_le_bytes((serialized.len() as u32) + 4));
        len_bytes.extend(serialized.iter());

        Some(len_bytes)
    }

    fn write_to_file(&self, file: &mut File) {
        let prefix = match self.msg_type {
            MessageType::Normal => "",
            MessageType::Error => "<!!!> ",
            MessageType::Warning => "<!> ",
            MessageType::Important => "<***> ",
        };

        // Format: "[process.subsystem][time] <message type> message"
        let _ = file.write_fmt(format_args!(
            "[{}.{}][{}] {}{}\n",
            self.process, self.group, self.time, prefix, self.string
        ));
    }
}

pub struct Logger {
    name: String,
    socket: Option<net::UdpSocket>,
    address: String,
    file: Option<File>,
}

impl Logger {
    pub fn new(name: &str) -> Logger {
        Logger {
            name: String::from(name),
            socket: None,
            address: String::new(),
            file: None,
        }
    }

    pub fn connect_udp(&mut self, address: &str) {
        self.socket = net::UdpSocket::bind("0.0.0.0:0").ok();
        self.address = String::from(address);
    }

    pub fn connect_file(&mut self, path: &str) {
        self.file = File::create(path).ok();
    }

    fn commit<S: AsRef<str>>(&mut self, msg_type: MessageType, value: S) {
        if self.socket.is_none() {
            return;
        }

        let message = Message {
            group: self.name.clone(),
            msg_type,
            string: String::from(value.as_ref()),
            process: get_proc_name(),
            time: Local::now().format("%H:%M:%S").to_string(),
        };

        if let Some(file) = &mut self.file {
            message.write_to_file(file);
        }

        let packed = message.pack();

        if packed.is_none() {
            return;
        }

        // Silence 'unused Result' warning.
        let _ = self
            .socket
            .as_ref()
            .unwrap()
            .send_to(&packed.unwrap(), self.address.as_str());
    }

    pub fn normal<S: AsRef<str>>(&mut self, contents: S) {
        self.commit(MessageType::Normal, contents);
    }

    pub fn warning<S: AsRef<str>>(&mut self, contents: S) {
        self.commit(MessageType::Warning, contents);
    }

    pub fn error<S: AsRef<str>>(&mut self, contents: S) {
        self.commit(MessageType::Error, contents);
    }

    pub fn important<S: AsRef<str>>(&mut self, contents: S) {
        self.commit(MessageType::Important, contents);
    }
}
