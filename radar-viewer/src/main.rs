use std::{io::BufReader, fs::File, thread, sync::mpsc};

use args::Args;
use clap::Parser;
use macroquad::{window::{clear_background, next_frame, self}, prelude::{Color, WHITE, BLACK, GREEN, BLUE, PINK, Vec2}, text::draw_text, shapes::{draw_line, draw_triangle_lines, draw_poly_lines, draw_triangle}};
use sct_reader::{reader::SctReader, sector::Sector, line::ColouredLine};
use sct_reader::line::Line as SectorLine;

mod args;

const WINDOW_HT_N_MI: f32 = 50.0;

#[macroquad::main("Sauna Radar")]
async fn main() {
    // Get command line args
    let args = Args::parse();
    let mut sector: Option<Sector> = None;

    let mut lines = Vec::new();
    let mut fixes = Vec::new();
    let mut regions = Vec::new();
    
    // Attempt to load sector file
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let sector = SctReader::new(BufReader::new(File::open(args.sector_file).unwrap())).try_read().unwrap();
        tx.send(sector).unwrap();
    });
    
    window::set_fullscreen(true);


    let mut position_calculator = None;

    loop {
        if sector.is_none() {
            if let Ok(new_sector) = rx.try_recv() {
                sector = Some(new_sector);
                let sector = sector.as_ref().unwrap();
                position_calculator = Some(PositionCalculator::new(sector.sector_info.default_centre_pt.lat as f32, sector.sector_info.default_centre_pt.lon as f32, WINDOW_HT_N_MI, sector.sector_info.n_mi_per_deg_lat, sector.sector_info.n_mi_per_deg_lon));
                let position_calculator = position_calculator.as_ref().unwrap();
                // .filter(|line| position_calculator.is_within_screen_bounds(line.start().lat as f32, line.start().lon as f32) || position_calculator.is_within_screen_bounds(line.end().lat as f32, line.end().lon as f32)
                for artcc_entry in sector.artcc_entries.iter().chain(sector.artcc_low_entries.iter()).chain(sector.artcc_high_entries.iter()).chain(sector.geo_entries.iter()).chain(sector.star_entries.iter()) {
                    for line in artcc_entry.lines.iter() {
                        let new_line = position_calculator.convert_line(line);
                        lines.push(new_line);
                    }
                }
                for fix in sector.fixes.iter() {
                    let fix_y = position_calculator.lat_to_window_y(fix.position.lat as f32);
                    let fix_x = position_calculator.lon_to_window_x(fix.position.lon as f32);
                    fixes.push(Fix { x: fix_x, y: fix_y });
                }

                for region_group in sector.regions.iter() {
                    for polygon in region_group.regions.iter() {
                        let mut vertices = polygon.vertices.iter().map(|position| {
                            [position.lon as f32, position.lat as f32]
                        }).flatten().collect::<Vec<_>>();
                        let indices = match earcutr::earcut(&vertices, &vec![], 2) {
                            Ok(indices) => indices,
                            Err(_) => continue,
                        };
                        vertices = vertices.chunks_exact(2).map(|chunk| {
                            let x = position_calculator.lon_to_window_x(chunk[0]);
                            let y = position_calculator.lat_to_window_y(chunk[1]);
                            [x, y]
                        }).flatten().collect();
                        let colour = Color::from_rgba(polygon.colour.r, polygon.colour.g, polygon.colour.b, 255);
                        let filled_polygon = FilledPolygon {
                            indices,
                            vertices,
                            colour,
                        };
                        regions.push(filled_polygon);
                    }
                }
        }
        }
        let state = if sector.is_some() {
            State::Loaded
        } else {
            State::Loading
        };

        match state {
            State::Loading => {
                clear_background(BLUE);
                draw_text("LOADING...", 5.0, 20.0, 20.0, WHITE);
            },
            State:: Loaded => {

                clear_background(BLACK);

                for region in regions.iter() {
                    region.draw();
                }

                for line in lines.iter() {
                    line.draw();
                }
                for fix in fixes.iter() {
                    fix.draw();
                }

                
                

                // for fix in sector.fixes.iter() {
                //     let fix_y = position_calculator.lat_to_window_y(fix.position.lat as f32);
                //     let fix_x = position_calculator.lon_to_window_x(fix.position.lon as f32);
                //     
                // }



                let fps_text = format!("FPS: {}", macroquad::time::get_fps());
                draw_text(&fps_text, 5.0, 20.0, 20.0, WHITE);

                macroquad_profiler::profiler(Default::default());
                // Prototype drawing

            }
        }

        
        
        next_frame().await
    }
}



enum State {
    Loading,
    Loaded,
}

#[derive(Debug)]
pub struct PositionCalculator {
    window_ht_n_mi: f32,
    n_mi_per_deg_lat: f32,
    n_mi_per_deg_lon: f32,
    origin_lat: f32,
    origin_lon: f32,
}

impl PositionCalculator {
    pub fn new(window_centre_lat: f32, window_centre_lon: f32, window_ht_n_mi: f32, n_mi_per_deg_lat: f32, n_mi_per_deg_lon: f32) -> PositionCalculator {
        let mut position_calculator = PositionCalculator {
            window_ht_n_mi,
            n_mi_per_deg_lat,
            n_mi_per_deg_lon,
            origin_lat: 0.0,
            origin_lon: 0.0,
        };
        position_calculator.update_centre_lat_lon(window_centre_lat, window_centre_lon);
        position_calculator
    }
    pub fn update_centre_lat_lon(&mut self, window_centre_lat: f32, window_centre_lon: f32) {
        let half_window_ht_px = window::screen_height() / 2.0;
        let lat_offset = half_window_ht_px / self.pixels_per_deg_lat();
        let origin_lat = window_centre_lat + lat_offset;

        let half_window_wi_px = window::screen_width() / 2.0;
        let lon_offset = half_window_wi_px / self.pixels_per_deg_lon();
        let origin_lon = window_centre_lon - lon_offset;

        self.origin_lat = origin_lat;
        self.origin_lon = origin_lon;
    }
    pub fn pixels_per_n_mi(&self) -> f32 {
        window::screen_height() / self.window_ht_n_mi
    }
    pub fn pixels_per_deg_lat(&self) -> f32 {
        self.pixels_per_n_mi() * self.n_mi_per_deg_lat
    }
    pub fn pixels_per_deg_lon(&self) -> f32 {
        self.pixels_per_n_mi() * self.n_mi_per_deg_lon
    }
    pub fn lat_to_window_y(&self, lat: f32) -> f32 {
        let deg_offset_from_origin = self.origin_lat - lat;
        let px_offset_from_origin = deg_offset_from_origin * self.pixels_per_deg_lat();
        px_offset_from_origin
    }
    pub fn lon_to_window_x(&self, lon: f32) -> f32 {
        let deg_offset_from_origin = lon - self.origin_lon;
        let px_offset_from_origin = deg_offset_from_origin * self.pixels_per_deg_lon();
        px_offset_from_origin
    }
    pub fn convert_line(&self, line: &ColouredLine) -> Line {
        let colour = if let Some(colour) = line.colour() {
            Color::from_rgba(colour.r, colour.g, colour.b, 255)
        } else {
            Color::from_rgba(38, 94, 97, 255)
        };
        let start_y = self.lat_to_window_y(line.start().lat as f32);
        let start_x = self.lon_to_window_x(line.start().lon as f32);

        let end_y = self.lat_to_window_y(line.end().lat as f32);
        let end_x = self.lon_to_window_x(line.end().lon as f32);

        Line {
            start_x,
            start_y,
            end_x,
            end_y,
            colour
        }
    }
    pub fn window_ht_deg(&self) -> f32 {
        self.window_ht_n_mi / self.n_mi_per_deg_lat
    }
    pub fn window_wi_deg(&self) -> f32 {
        let window_wi_n_mi = window::screen_width() / self.pixels_per_n_mi();
        window_wi_n_mi / self.n_mi_per_deg_lon
    }
    pub fn is_within_screen_bounds(&self, lat: f32, lon: f32) -> bool {
        let top_lat = self.origin_lat;
        let bottom_lat = self.origin_lat - self.window_ht_deg();
        let left_lon = self.origin_lon;
        let right_lon = self.origin_lon + self.window_wi_deg();

        let lat_a = f32::min(top_lat, bottom_lat);
        let lat_b = f32::max(top_lat, bottom_lat);

        let lon_a = f32::min(left_lon, right_lon);
        let lon_b = f32::max(left_lon, right_lon);
        (lat_a..=lat_b).contains(&lat)
        &&
        (lon_a..=lon_b).contains(&lon)
    }
}



// Work out window dimensions
// Decide how many nms = window height (e.g. 70)
// Window height px / nms = pixels per nautical mile
// We know screen centre lat_long
// let deg_diff = pos.lat - screen_centre.lat;
// let offset_px_from_centre = deg_diff * nm_per_lat * pixels_per_nautical mile
// let offset_lat_from_top = screen_height_px - (offset_px_from_centre - (screen_ht_px / 0.5));

// window_dimensions = 800.0 * 600;
// window_ht_px = 600.0;
// window_ht_nm = 70.0;
// px_per_nm = 8.0;

// screen_centre_lat_lon = 50.0, 20.0;
// origin_lat = ((screen_ht_px / 2.0) / (px_per_nautical_mile * 60.0))
// our_pos = 51.0, 20.0;
// deg_diff_lat = 51.0 - 50.0 = 1.0;
// offset_px_from_centre = 1.0 * 60.0 * 8.0 = 480.0;
// offset_px_from_top = 


pub struct Line {
    pub start_x: f32,
    pub start_y: f32,

    pub end_x: f32,
    pub end_y: f32,
    pub colour: Color,
}
impl Line {
    pub fn draw(&self) {
        if (self.start_x < window::screen_width() && self.start_y < window::screen_height())
        ||
        (self.end_x < window::screen_width() && self.end_y < window::screen_height()) {
            draw_line(self.start_x, self.start_y, self.end_x, self.end_y, 1.0, self.colour);
        }
    }
}

pub struct Fix {
    pub x: f32,
    pub y: f32,
}
impl Fix {
    pub fn draw(&self) {
        draw_poly_lines(self.x, self.y, 3, 4.0,30.0, 1.0, Color::from_rgba(38, 94, 97, 255));
    }
}

pub struct FilledPolygon {
    pub indices: Vec<usize>,
    pub vertices: Vec<f32>,
    pub colour: Color,
}
impl FilledPolygon {
    pub fn draw(&self) {
        for triangle in self.indices.chunks_exact(3) {
            let index_a = triangle[0] * 2;
            let index_b = triangle[1] * 2;
            let index_c = triangle[2] * 2;
            let vertex_a = Vec2::new(self.vertices[index_a], self.vertices[index_a + 1]);
            let vertex_b = Vec2::new(self.vertices[index_b], self.vertices[index_b + 1]);
            let vertex_c = Vec2::new(self.vertices[index_c], self.vertices[index_c + 1]);

            draw_triangle(vertex_a, vertex_b, vertex_c, self.colour);
        }
    }
}

// 0.0, 0.0,    100.0, 0.0,    100.0, 100.0,    0.0, 100.0,   20.0, 20.0,    80.0, 20.0,    80.0, 80.0,    20.0,80.
// [3,0,4, 5,4,0, 3,4,7, 5,0,1, 2,3,7, 6,5,1, 2,7,6, 6,1,2]