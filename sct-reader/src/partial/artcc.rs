use crate::{position::Position, sector::Line};

#[derive(Debug, Clone)]
pub struct MultiLine {
    pub name: String,
    pub lines: Vec<Line>,
}
