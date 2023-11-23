use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, mpsc::{Sender, Receiver, self}}, thread::JoinHandle, net::{TcpStream, Shutdown}, time::Duration, io::{BufWriter, Write}};
use std::thread;

use common::{ipc::{radar_to_ui, ui_to_radar}, aircraft_data::AircraftUpdate};

const ATTEMPT_UI_CONNECT_DELAY: Duration = Duration::from_millis(500);



pub struct IpcManager {
    thread_should_terminate: Arc<AtomicBool>,
    rtu_tx: Sender<radar_to_ui::PacketType>,
    msg_rx: Receiver<ImplMessage>,
    tcp_stream_to_drop: Option<TcpStream>,
    thread: Option<JoinHandle<()>>,
}

impl IpcManager {
    pub fn new(port: u16) -> Self {
        let (rtu_tx, rtu_rx) = mpsc::channel::<radar_to_ui::PacketType>();
        let (msg_tx, msg_rx) = mpsc::channel::<ImplMessage>();
        let thread_should_terminate = Arc::new(AtomicBool::new(false));
        
        let thread = ipc_worker(Arc::clone(&thread_should_terminate), msg_tx, rtu_rx, port);


        IpcManager { thread_should_terminate, rtu_tx, msg_rx, tcp_stream_to_drop: None, thread: Some(thread) }
    }
    pub fn poll(&mut self, max: usize) -> Vec<Message> {
        let mut count = 0;
        let mut vec = Vec::with_capacity(max);
        while let Ok(impl_message) = self.msg_rx.try_recv() {
            if count == max { break; }
            match impl_message {
                ImplMessage::Message(message) => {
                    vec.push(message);
                    count += 1;
                },
                ImplMessage::NewTcpStream(tcp_stream) => self.tcp_stream_to_drop = Some(tcp_stream),
            }
        }
        vec
    }
}

impl Drop for IpcManager {
    fn drop(&mut self) {
        self.thread_should_terminate.store(true, Ordering::Relaxed);
        if let Some(tcp_stream) = self.tcp_stream_to_drop.take() {
            tcp_stream.shutdown(Shutdown::Both).ok();
        }
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}


fn ipc_worker(thread_should_terminate: Arc<AtomicBool>, msg_tx: Sender<ImplMessage>, rtu_rx: Receiver<radar_to_ui::PacketType>, port: u16) -> JoinHandle<()> {
    
    thread::spawn(move || {
        let hostname = format!("localhost:{port}");
        'outer: loop {
            std::thread::sleep(ATTEMPT_UI_CONNECT_DELAY);
            match TcpStream::connect(&hostname) {
                Err(_) => {
                    if thread_should_terminate.load(Ordering::Relaxed) {
                        break 'outer;
                    }
                },
                Ok(tcp_stream) => {
                    msg_tx.send(ImplMessage::NewTcpStream(tcp_stream.try_clone().unwrap()));
                    let mut tcp_sender = BufWriter::new(tcp_stream.try_clone().unwrap());
                    loop {
                        // Forward message if there are any to send
                        if let Ok(packet) = rtu_rx.try_recv() {
                            let (size, bytes) = serialise_radar_to_ui_message(&packet);
                            std::io::copy(&mut size.as_slice(), &mut tcp_sender).ok();
                            std::io::copy(&mut bytes.as_slice(), &mut tcp_sender).ok();
                            tcp_sender.flush().ok();
                        }

                        if thread_should_terminate.load(Ordering::Relaxed) {
                            break 'outer;
                        }

                        // Wait for the next inbound message
                        if let Ok(packet) = bincode::deserialize_from::<_, ui_to_radar::PacketType>(&tcp_stream) {
                            match packet {
                                ui_to_radar::PacketType::AircraftDataUpdate(aircraft_update) => {
                                    msg_tx.send(ImplMessage::Message(Message::AircraftDataUpdate(aircraft_update))).ok();
                                },
                                ui_to_radar::PacketType::LogMessage(log_message) => {
                                    msg_tx.send(ImplMessage::Message(Message::LogMessage(log_message))).ok();
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
            
        }

    })
}


#[derive(Debug, Clone)]
pub enum Message {
    AircraftDataUpdate(Vec<AircraftUpdate>),
    LogMessage(String),
}

#[derive(Debug)]
enum ImplMessage {
    Message(Message),
    NewTcpStream(TcpStream),
}


fn serialise_radar_to_ui_message(message: &radar_to_ui::PacketType) -> ([u8; 4], Vec<u8>) {
    let size = bincode::serialized_size(&message).unwrap() as u32;
    let bytes = bincode::serialize(&message).unwrap();
    (u32::to_be_bytes(size), bytes)
}