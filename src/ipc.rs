use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Stream};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::thread;
use tao::event_loop::EventLoopProxy;

const SOCKET_NAME: &str = "MarkdownReader-SingleInstance-V1";

pub enum UserEvent {
    OpenFile(PathBuf),
    UpdateTitle(String),
    OpenWelcome,
    ActivateWindow,
}

pub fn try_send_to_existing_instance(file_path: Option<&Path>) -> bool {
    let name = match SOCKET_NAME.to_ns_name::<GenericNamespaced>() {
        Ok(n) => n,
        Err(_) => return false,
    };

    let stream = match Stream::connect(name) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut writer = stream;
    let payload = if let Some(path) = file_path {
        path.to_string_lossy().to_string()
    } else {
        String::new()
    };

    if writeln!(writer, "{}", payload).is_ok() {
        let _ = writer.flush();
        return true;
    }

    false
}

pub fn start_ipc_server(proxy: EventLoopProxy<UserEvent>) {
    let name = match SOCKET_NAME.to_ns_name::<GenericNamespaced>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to create IPC socket name: {}", e);
            return;
        }
    };

    let listener = match ListenerOptions::new().name(name).create_sync() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to create IPC listener: {}", e);
            return;
        }
    };

    thread::spawn(move || {
        for conn in listener.incoming() {
            if let Ok(stream) = conn {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        let path = PathBuf::from(trimmed);
                        let _ = proxy.send_event(UserEvent::OpenFile(path));
                    } else {
                        let _ = proxy.send_event(UserEvent::ActivateWindow);
                    }
                }
            }
        }
    });
}
