use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::{position::Position, util};

use self::fms_graphics::FmsGraphic;

pub mod fms_graphics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftUpdate {
    callsign: String,
    data: AircraftData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftData {
    position: Position,
    heading_mag: f32,
    heading_true: f32,
    track_mag: f32,
    track_true: f32,
    pitch: f32,
    bank: f32,
    indicated_airspeed: f32,
    ground_speed: f32,
    vertical_speed: f32,
    wind_direction: f32,
    wind_speed: f32,
    on_ground: bool,
    altimeter_setting_hpa: f32,
    autopilot: Autopilot,
    fms_string: String,
    fms_graphics: Vec<FmsGraphic>,
    sim_rate: f32,
    is_paused: f32,
    connection_status: ConnectionStatus,
}





/// Autopilot
/// This can be deserialised directly from Sauna API
/// as well as used locally
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Autopilot {
    selected_heading: u32,
    selected_altitude: u32,
    selected_vertical_speed: i32,
    selected_fpa: f64,
    selected_speed: Speed,



    current_lateral_mode: LateralMode,
    armed_lateral_modes: Vec<LateralMode>,
    current_vertical_mode: VerticalMode,
    armed_vertical_modes: Vec<VerticalMode>,
    current_thrust_mode: ThrustMode,
    armed_thrust_modes: Vec<ThrustMode>,
}

/// Autopilot lateral mode
/// This can be deserialised directly from Sauna API
/// as well as used locally
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LateralMode {
    Bank,
    #[serde(rename = "HDG")]
    Heading,
    Track,
    Lnav,
    #[serde(rename = "TO")]
    TakeOff,
    #[serde(rename = "GA")]
    GoAround,
    #[serde(rename = "APCH")]
    Approach,
    #[serde(rename = "LDG")]
    Landing,
    Taxi,
}
impl Display for LateralMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            LateralMode::Bank => "BANK",
            LateralMode::Heading => "HDG",
            LateralMode::Track => "TRK",
            LateralMode::Lnav => "LNAV",
            LateralMode::TakeOff => "T/O",
            LateralMode::GoAround => "G/A",
            LateralMode::Approach => "APCH",
            LateralMode::Landing => "LAND",
            LateralMode::Taxi => "TAXI",
        })
    }
}




/// Autopilot vertical mode
/// This can be deserialised directly from Sauna API
/// as well as used locally
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerticalMode {
    #[serde(rename = "ALT")]
    AltitudeHold,
    #[serde(rename = "FLCH")]
    FlightLevelChange,
    #[serde(rename = "VS")]
    VerticalSpeed,
    #[serde(rename = "FPA")]
    FlightPathAngle,
    #[serde(rename = "ASEL")]
    AltitudeSelect,
    #[serde(rename = "VNAV")]
    VNAV,
    #[serde(rename = "TOGA")]
    TOGA,
    #[serde(rename = "LDG")]
    Landing,
    Taxi,
    #[serde(rename = "APCH")]
    Approach,
}

impl Display for VerticalMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            VerticalMode::AltitudeHold => "ALT",
            VerticalMode::FlightLevelChange => "FLCH",
            VerticalMode::VerticalSpeed => "V/S",
            VerticalMode::FlightPathAngle => "FPA",
            VerticalMode::AltitudeSelect => "ASEL",
            VerticalMode::VNAV => "VNAV",
            VerticalMode::TOGA => "TO/GA",
            VerticalMode::Landing => "LAND",
            VerticalMode::Taxi => "TAXI",
            VerticalMode::Approach => "APCH",
        })
    }
}


/// Autopilot thrust mode
/// This can be deserialised directly from Sauna API
/// as well as used locally
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThrustMode {
    Speed,
    Thrust,
    Taxi,
}

impl Display for ThrustMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ThrustMode::Speed => "SPEED",
            ThrustMode::Thrust => "THRUST",
            ThrustMode::Taxi => "TAXI",
        })
    }
}


/// Autopilot speed mode
/// This can be deserialised directly from Sauna API
/// as well as used locally
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpeedMode {
    Manual,
    Fms,
}
impl Display for SpeedMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SpeedMode::Manual => "MANUAL",
            SpeedMode::Fms => "FMS",
        })
    }
}

/// Speed units
/// This can be deserialised directly from Sauna API
/// as well as used locally
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "selectedSpeedUnits", content = "selectedSpeed")]
pub enum Speed {
    Knots(i32),
    Mach(i32),
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransponderMode {
    Standby,
    ModeC,
    Ident,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Fms {
    pub as_string: String,
    pub fms_lines: Vec<SimAircraftFmsLine>,
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum SimAircraftFmsLine {
    #[serde(rename_all = "camelCase")]
    Arc  { start_point: Position, end_point: Position, center: Position, #[serde(rename = "radius_m")] radius_m: f32, start_true_bearing: f32, end_true_bearing: f32, clockwise: bool },

    #[serde(rename_all = "camelCase")]
    Line { start_point: Position, end_point: Position },
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Waiting(i32),
    Connecting,
    Connected,
    Disconnected,
}
impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConnectionStatus::Waiting(delay) => format!("Waiting ({})", util::seconds_to_time_string(*delay)),
                ConnectionStatus::Connecting => "Connecting".to_owned(),
                ConnectionStatus::Connected => "Connected".to_owned(),
                ConnectionStatus::Disconnected => "Disconnected".to_owned(),
            }
        )
    }
}