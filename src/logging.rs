//! Logging backend which logs over UDP and to a file.

use cached::proc_macro::cached;
use chrono::Local;
use log::{Level, Metadata, Record};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, net, path::Path, sync::Mutex};

#[derive(Clone, Copy, Serialize, Deserialize)]
enum MessageType {
    Normal,
    Error,
    Warning,
    Debug,
}

#[derive(Serialize, Deserialize)]
struct Message {
    module: String,
    msg_type: MessageType,
    string: String,
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
        let level_name = match self.msg_type {
            MessageType::Normal => "info",
            MessageType::Error => "error",
            MessageType::Warning => "warning",
            MessageType::Debug => "debug",
        };

        // This is a direct copy of the format used by VSCode (adapted for Rust).
        //      [date time] [module] [level] Text
        let _ = file.write_fmt(format_args!(
            "[{}] [{}] [{}] {}\n",
            self.time, self.module, level_name, self.string
        ));
    }
}

pub struct Logger {
    socket: Option<net::UdpSocket>,
    address: String,
    file: Mutex<Option<File>>,
}

pub(crate) static mut GLOBAL_LOGGER: Option<Logger> = None;

impl Logger {
    pub fn new() -> &'static mut Logger {
        unsafe {
            if GLOBAL_LOGGER.is_some() {
                panic!("Logger already created!");
            }

            GLOBAL_LOGGER = Some(Logger {
                socket: None,
                address: String::new(),
                file: Mutex::new(None),
            });

            GLOBAL_LOGGER.as_mut().unwrap()
        }
    }

    pub fn connect_udp(&mut self, address: &str) {
        self.socket = net::UdpSocket::bind("0.0.0.0:0").ok();
        self.address = String::from(address);
    }

    pub fn connect_file<P: AsRef<Path>>(&mut self, path: P) {
        self.file = Mutex::new(File::create(path).ok());
    }

    pub fn commit(&self, record: &log::Record) {
        let msg_type = match record.level() {
            Level::Error => MessageType::Error,
            Level::Warn => MessageType::Warning,
            Level::Info => MessageType::Normal,
            Level::Debug | Level::Trace => MessageType::Debug,
        };

        let message = Message {
            module: record
                .module_path()
                .unwrap_or("unknown")
                .split("::")
                .last()
                .unwrap()
                .to_string(),
            msg_type,
            string: format!("{}", record.args()),
            time: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        };

        if record.level() < Level::Debug {
            if let Ok(mut file) = self.file.lock() {
                message.write_to_file(file.as_mut().unwrap());
            }
        }

        if self.socket.is_none() {
            return;
        }

        let packed = message.pack();

        if packed.is_none() {
            return;
        }

        let _ = self
            .socket
            .as_ref()
            .unwrap()
            .send_to(&packed.unwrap(), self.address.as_str());
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.commit(record);
            // self.commit(record.level(), format!("{}", record.args()));
        }
    }

    fn flush(&self) {}
}
