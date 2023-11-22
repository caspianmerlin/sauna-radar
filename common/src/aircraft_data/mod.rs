use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::{position::Position, util};

use self::fms_graphics::FmsGraphic;

pub mod fms_graphics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftUpdate {
    pub callsign: String,
    pub data: AircraftData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftData {
    pub position: Position, //
    pub heading_mag: f32, //
    pub heading_true: f32, //
    pub track_mag: f32, //
    pub track_true: f32, //
    pub pitch: f32, //
    pub bank: f32, //
    pub indicated_airspeed: f32, //
    pub mach_number: f32, //
    pub ground_speed: f32, //
    pub vertical_speed: f32, //
    pub wind_direction: f32, //
    pub wind_speed: f32, //
    pub on_ground: bool, //
    pub altimeter_setting_hpa: f32, //
    pub autopilot: Autopilot,
    pub fms_string: String, //
    pub fms_graphics: Vec<FmsGraphic>, //
    pub sim_rate: f32, //
    pub is_paused: bool, //
    pub connection_status: ConnectionStatus, //
}





/// Autopilot
/// This can be deserialised directly from Sauna API
/// as well as used locally
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Autopilot {
    pub selected_heading: u32,
    pub selected_altitude: u32,
    pub selected_vertical_speed: i32,
    pub selected_fpa: f32,
    pub selected_speed_units: SpeedUnits,
    pub selected_speed: i32,



    pub current_lateral_mode: LateralMode,
    pub armed_lateral_modes: Vec<LateralMode>,
    pub current_vertical_mode: VerticalMode,
    pub armed_vertical_modes: Vec<VerticalMode>,
    pub current_thrust_mode: ThrustMode,
    pub armed_thrust_modes: Vec<ThrustMode>,
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
pub enum SpeedUnits {
    Knots,
    Mach,
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransponderMode {
    Standby,
    ModeC,
    Ident,
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