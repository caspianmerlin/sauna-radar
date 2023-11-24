use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, mpsc::{Sender, Receiver, self}}, thread::JoinHandle, net::{TcpStream, Shutdown}, time::Duration, io::{BufWriter, Write}, fmt::format};
use std::thread;
use common::{aircraft_data::{Autopilot, fms_graphics::{FmsGraphic, FmsLine, FmsArc, FmsArcState}}, position::Position, ipc::radar_to_ui::PacketType, api_requests::ApiRequestType};
use serde::{Deserialize, Serialize};
use common::{ipc::{radar_to_ui, ui_to_radar}, aircraft_data::AircraftUpdate};
use log::{info, error};

const API_POLL_INTERVAL: Duration = Duration::from_millis(1000);
const API_REQUEST_TIMEOUT: Duration = Duration::from_millis(1000);
const AIRCRAFT_DATA_ENDPOINT: &str = "/api/aircraft/getAllWithFms";
const LOG_BUFFER_ENDPOINT: &str = "/api/commands/commandBuffer";
const TEXT_COMMAND_ENDPOINT: &str = "/api/commands/send/textCommand";


pub struct ApiLink {
    thread_should_terminate: Arc<AtomicBool>,
    rta_tx: Sender<radar_to_ui::PacketType>,
    msg_rx: Receiver<ImplMessage>,
    thread: Option<JoinHandle<()>>,
}

impl ApiLink {
    pub fn new(hostname: String, port: u16) -> Self {
        let (rta_tx, rta_rx) = mpsc::channel::<radar_to_ui::PacketType>();
        let (msg_tx, msg_rx) = mpsc::channel::<ImplMessage>();
        let thread_should_terminate = Arc::new(AtomicBool::new(false));
        
        let hostname = format_hostname(&hostname, port);
        let thread = api_worker(Arc::clone(&thread_should_terminate), msg_tx, rta_rx, hostname);


        ApiLink { thread_should_terminate, rta_tx, msg_rx, thread: Some(thread) }
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
            }
        }
        vec
    }
    pub fn send(&self, packet: radar_to_ui::PacketType) {
        self.rta_tx.send(packet).ok();
    }
}

impl Drop for ApiLink {
    fn drop(&mut self) {
        self.thread_should_terminate.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}


fn api_worker(thread_should_terminate: Arc<AtomicBool>, msg_tx: Sender<ImplMessage>, rta_rx: Receiver<radar_to_ui::PacketType>, hostname: String) -> JoinHandle<()> {
    
    thread::spawn(move || {

        let hostname = hostname;
        let aircraft_data_endpoint = format!("{hostname}{AIRCRAFT_DATA_ENDPOINT}");
        let log_buffer_endpoint = format!("{hostname}{LOG_BUFFER_ENDPOINT}");
        let text_command_endpoint = format!("{hostname}{TEXT_COMMAND_ENDPOINT}");
        let client = ureq::AgentBuilder::new().timeout(API_REQUEST_TIMEOUT).build();
        loop {
            if thread_should_terminate.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(API_POLL_INTERVAL);

            // Get aircraft data
            if let Some(data) =  client.get(&aircraft_data_endpoint).call().ok().and_then(|response| response.into_json::<Vec<SimAircraft>>().ok()).map(|vec| vec.into_iter().map(AircraftUpdate::from).collect::<Vec<_>>()) {
                msg_tx.send(ImplMessage::Message(Message::AircraftDataUpdate(data))).ok();
            }

            // Get log buffer
            if let Some(data) = client.get(&log_buffer_endpoint).call().ok().and_then(|response| response.into_json::<Vec<String>>().ok()) {
                for log_msg in data {
                    msg_tx.send(ImplMessage::Message(Message::LogMessage(log_msg))).ok();
                }
            }

            // Send any requests if there are any to send
            if let Ok(PacketType::ApiRequest(ApiRequestType::TextCommand(text_command_request))) = rta_rx.try_recv() {
                client.post(&text_command_endpoint).send_json(&text_command_request).ok();
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
}

fn format_hostname(hostname: &str, port: u16) -> String {
    let hostname = hostname.trim_start_matches("http://");
    format!("http://{hostname}:{port}")
}

































#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimAircraft {
    callsign: String,
    delay_ms: i32,
    sim_state: SimState,
    fms: Fms,
    position: SimAircraftPosition,
    autopilot: Autopilot,
    connection_status: ConnectionStatus,
}

impl From<SimAircraft> for common::aircraft_data::AircraftUpdate {
    fn from(value: SimAircraft) -> Self {
        let callsign = value.callsign;
        let position = common::position::Position::new_with_alt(value.position.latitude, value.position.longitude, value.position.indicated_altitude);
        let heading_mag = value.position.magnetic_heading;
        let heading_true = value.position.true_heading;
        let track_mag = value.position.track_mag;
        let track_true = value.position.track_true;
        let pitch = value.position.pitch;
        let bank = value.position.bank;
        let indicated_airspeed = value.position.indicated_air_speed;
        let mach_number = value.position.mach_number;
        let ground_speed = value.position.ground_speed;
        let vertical_speed = value.position.vertical_speed;
        let wind_direction = value.position.wind_direction;
        let wind_speed = value.position.wind_speed;
        let on_ground = value.position.on_ground;
        let altimeter_setting_hpa = value.position.altimeter_setting;
        let autopilot = value.autopilot;
        let fms_string = value.fms.as_string;
        let fms_graphics = value.fms.fms_lines.into_iter().map(FmsGraphic::from).collect::<Vec<_>>();
        let sim_rate = value.sim_state.sim_rate;
        let is_paused = value.sim_state.paused;
        let connection_status = match value.connection_status {
            ConnectionStatus::Connected => common::aircraft_data::ConnectionStatus::Connected,
            ConnectionStatus::Disconnected => common::aircraft_data::ConnectionStatus::Disconnected,
            ConnectionStatus::Connecting => common::aircraft_data::ConnectionStatus::Connecting,
            ConnectionStatus::Waiting => common::aircraft_data::ConnectionStatus::Waiting(value.delay_ms),
        };

        common::aircraft_data::AircraftUpdate {
            callsign,
            data: common::aircraft_data::AircraftData {
                position,
                heading_mag,
                heading_true,
                track_mag,
                track_true,
                pitch,
                bank,
                indicated_airspeed,
                mach_number,
                ground_speed,
                vertical_speed,
                wind_direction,
                wind_speed,
                on_ground,
                altimeter_setting_hpa,
                autopilot,
                fms_string,
                fms_graphics,
                sim_rate,
                is_paused,
                connection_status
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimState {
    sim_rate: f32,
    paused: bool,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimAircraftPosition {
    latitude: f32,
    longitude: f32,
    #[serde(rename = "heading_Mag")]
    magnetic_heading: f32,
    #[serde(rename = "heading_True")]
    true_heading: f32,
    #[serde(rename = "track_Mag")]
    track_mag: f32,
    #[serde(rename = "track_True")]
    track_true: f32,
    bank: f32,
    pitch: f32,
    mach_number: f32,
    vertical_speed: f32,
    on_ground: bool,
    indicated_altitude: f32,
    indicated_air_speed: f32,
    ground_speed: f32,
    #[serde(rename = "altimeterSetting_hPa")]
    altimeter_setting: f32,
    wind_direction: f32,
    wind_speed: f32,
}


#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectionStatus {
    Waiting,
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Fms {
    pub as_string: String,
    pub fms_lines: Vec<SimAircraftFmsLine>,
}


#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum SimAircraftFmsLine {
    #[serde(rename_all = "camelCase")]
    Arc  { start_point: Position, end_point: Position, center: Position, #[serde(rename = "radius_m")] radius_m: f32, start_true_bearing: f32, end_true_bearing: f32, clockwise: bool },

    #[serde(rename_all = "camelCase")]
    Line { start_point: Position, end_point: Position },
}

impl From<SimAircraftFmsLine> for FmsGraphic {
    fn from(value: SimAircraftFmsLine) -> Self {
        match value {
            SimAircraftFmsLine::Line { start_point, end_point } => {
                FmsGraphic::Line(FmsLine { start: start_point, end: end_point })
            },
            SimAircraftFmsLine::Arc { center, radius_m, start_true_bearing, end_true_bearing, clockwise, .. } => {
                FmsGraphic::Arc(
                    FmsArc {
                        state: FmsArcState::Uninitialised { centre: center, radius_m, start_bearing_true: start_true_bearing, end_bearing_true: end_true_bearing, clockwise },
                    }
                )
            },
        }
    }
}