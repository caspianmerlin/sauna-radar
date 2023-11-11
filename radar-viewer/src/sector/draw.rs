use crate::radar::position_calc::PositionCalculator;



pub trait Draw {
    fn draw(&mut self, position_calculator: &PositionCalculator, recalculate_coordinates: bool);
}