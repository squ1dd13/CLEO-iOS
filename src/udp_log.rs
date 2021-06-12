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
    file: Mutex<Option<File>>,
}

pub(crate) static mut GLOBAL_LOGGER: Option<Logger> = None;

impl Logger {
    pub fn new(name: &str) -> &'static mut Logger {
        unsafe {
            if GLOBAL_LOGGER.is_some() {
                panic!("Logger already created!");
            }

            GLOBAL_LOGGER = Some(Logger {
                name: String::from(name),
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

    pub fn commit<S: AsRef<str>>(&self, level: Level, value: S) {
        let msg_type = match level {
            Level::Error => MessageType::Error,
            Level::Warn => MessageType::Warning,
            Level::Info => MessageType::Important,
            Level::Debug => MessageType::Normal,
            Level::Trace => MessageType::Normal,
        };

        let message = Message {
            group: self.name.clone(),
            msg_type,
            string: String::from(value.as_ref()),
            process: get_proc_name(),
            time: Local::now().format("%H:%M:%S").to_string(),
        };

        if level < Level::Debug {
            if let Some(mut file) = self.file.lock().ok() {
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
            self.commit(record.level(), format!("{}", record.args()));
        }
    }

    fn flush(&self) {}
}
