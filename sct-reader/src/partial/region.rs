use crate::{position::{Position, Valid}, colour::Colour};



#[derive(Debug)]
pub struct PartialRegionGroup {
    pub name: String,
    pub regions: Vec<PartialRegion>,
}
impl PartialRegionGroup {
    pub fn new(name: String) -> PartialRegionGroup {
        PartialRegionGroup { name, regions: vec![] }
    }
}

#[derive(Debug, Default)]
pub struct PartialRegion {
    pub colour: Option<Colour>,
    pub vertices: Vec<Position<Valid>>,
}

