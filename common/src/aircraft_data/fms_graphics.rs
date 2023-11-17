use serde::{Serialize, Deserialize};

use crate::position::Position;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum FmsGraphic {
    Line(FmsLine),
    Arc(FmsArc),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FmsArc {
    pub state: FmsArcState,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum FmsArcState {
    Initialised{ lines: Vec<FmsLine> },
    Uninitialised { centre: Position, radius_m: f32, start_bearing_true: f32, end_bearing_true: f32, clockwise: bool }, 
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FmsLine {
    pub start: Position,
    pub end: Position,
}