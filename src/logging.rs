//! Logging backend which logs over UDP and to a file.

use chrono::Local;
use log::{Level, Metadata, Record};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, net, sync::Mutex};

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

impl Message {
    fn pack(&self) -> Option<Vec<u8>> {
        let serialized = bincode::serialize::<Message>(self).ok();

        if serialized.is_none() {
            return serialized;
        }

        let serialized = serialized.unwrap();

        let mut len_bytes = Vec::from(u32::to_le_bytes((serialized.len() as u32) + 4));
        len_bytes.extend(&serialized);

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

pub struct Logger;

impl Logger {
    pub fn commit(&self, record: &log::Record) {
        let msg_type = match record.level() {
            Level::Error => MessageType::Error,
            Level::Warn => MessageType::Warning,
            Level::Info => MessageType::Normal,
            Level::Debug | Level::Trace => MessageType::Debug,
        };

        let module_path = match record.module_path() {
            Some(path) if path.contains("mio::") || path.contains("reqwest") => return,
            Some(path) => path,
            None => return,
        };

        let message = Message {
            module: module_path
                .split("::")
                .last()
                .unwrap_or("unknown")
                .to_string(),
            msg_type,
            string: format!("{}", record.args()),
            time: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        };

        if let Some(Err(err)) = MSG_SENDER.get().map(|s| s.lock().map(|s| s.send(message))) {
            log::error!("error in log sender chain: {}", err);
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.commit(record);
        }
    }

    fn flush(&self) {}
}

static MSG_SENDER: OnceCell<Mutex<std::sync::mpsc::Sender<Message>>> = OnceCell::new();

fn panic_hook(info: &std::panic::PanicInfo) {
    let message = info
        .message()
        .map(ToString::to_string)
        .or_else(|| info.payload().downcast_ref::<&str>().map(|s| s.to_string()));

    let message = match message.as_ref() {
        Some(m) => m,
        None => "no message, sorry :/",
    };

    let aslr_slide = crate::hook::get_game_aslr_offset();
    let time = chrono::Local::now();
    let backtrace = std::backtrace::Backtrace::force_capture();

    let info_dump = format!(
        "Uh oh! Looks like something went very wrong.

Please don't ignore this! There are a few different ways you can help out. You could...
  - Send this on Discord by DM (squ1dd13#1643) or in the server (discord.gg/cXwkTUasJU), or
  - Create a new issue on GitHub: https://github.com/squ1dd13/CLEO-iOS/issues/new

Below is some information that might help explain the problem.

ASLR slide (game): {aslr_slide:#x}
Message: {message}
Time: {time}
Backtrace: see below

{backtrace}"
    );

    log::error!("{info_dump}");

    let _ = std::fs::write(
        crate::meta::resources::get_documents_path("PANIC.txt"),
        info_dump,
    );

    std::process::abort();
}

fn install_panic_hook() {
    // Install the panic hook so we can print useful stuff rather than just exiting on a panic.
    std::panic::set_hook(Box::new(panic_hook));
}

pub fn init() {
    install_panic_hook();

    log::set_logger(unsafe {
        static mut DUMMY: Logger = Logger {};
        &mut DUMMY
    })
    .map(|_| log::set_max_level(log::LevelFilter::max()))
    .unwrap();

    let (sender, receiver) = std::sync::mpsc::channel();

    // fixme: MSG_SENDER may be being set too late for some launches.
    MSG_SENDER.set(Mutex::new(sender)).unwrap();

    // Only attempt to connect over UDP if we're in debug mode.
    let socket = if cfg!(feature = "debug") {
        net::UdpSocket::bind("0.0.0.0:0").ok()
    } else {
        None
    };

    let mut file = File::create(crate::meta::resources::get_log_path()).unwrap();

    // Start receiving log messages on a background thread. This eliminates the massive performance
    //  impact of writing to files/sockets in normal game code.
    std::thread::spawn(move || loop {
        let msg = receiver.recv().unwrap();
        msg.write_to_file(&mut file);

        if let Some(socket) = &socket {
            if let Some(bin) = msg.pack() {
                let _ = socket.send_to(&bin, "172.16.0.48:4568");
            }
        }
    });
}
