use std::{fs::File, io::BufReader, sync::mpsc, thread};

use args::Args;
use asr::Asr;
use clap::Parser;
use macroquad::{
    prelude::{Color, Vec2, BLACK, BLUE, GREEN, PINK, WHITE},
    shapes::{draw_line, draw_poly_lines, draw_triangle, draw_triangle_lines},
    text::draw_text,
    window::{self, clear_background, next_frame},
};
use sct_reader::line::Line as SectorLine;
use sct_reader::{line::ColouredLine, reader::SctReader, sector::Sector};

mod args;
mod asr;
mod radar;



#[macroquad::main("Sauna Radar")]
async fn main() {
    // Get command line args
    let args = Args::parse();
    let mut sector: Option<Sector> = None;
    let mut asr: Option<Asr> = None;

    let mut lines = Vec::new();
    let mut fixes = Vec::new();
    let mut airports = Vec::new();
    let mut regions = Vec::new();

    // Attempt to load sector file
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let asr = args.asr_file.map(|path| Asr::from_file(path).unwrap());
        let mut sector = SctReader::new(BufReader::new(File::open(args.sector_file).unwrap()))
            .try_read()
            .unwrap();
        // Set centre airport if there is one
        if let Some(centre_airport) = args.centre_airport {
            if let Some(airport) = sector
                .airports
                .iter()
                .find(|x| x.identifier == centre_airport)
            {
                sector.sector_info.default_centre_pt = airport.position
            }
        }
        tx.send((sector, asr)).unwrap();
    });

    window::set_fullscreen(true);

    let mut position_calculator = None;

    loop {
        if sector.is_none() {
            if let Ok((new_sector, new_asr)) = rx.try_recv() {
                sector = Some(new_sector);
                asr = new_asr;
                let sector = sector.as_ref().unwrap();
                position_calculator = Some(PositionCalculator::new(
                    sector.sector_info.default_centre_pt.lat as f32,
                    sector.sector_info.default_centre_pt.lon as f32,
                    WINDOW_HT_N_MI,
                    sector.sector_info.n_mi_per_deg_lat,
                    sector.sector_info.n_mi_per_deg_lon,
                ));
                let position_calculator = position_calculator.as_ref().unwrap();
                // .filter(|line| position_calculator.is_within_screen_bounds(line.start().lat as f32, line.start().lon as f32) || position_calculator.is_within_screen_bounds(line.end().lat as f32, line.end().lon as f32)

                for entry in sector.artcc_entries.iter().filter(|entry| {
                    asr.as_ref()
                        .map(|asr| asr.artcc_boundary.contains(&entry.name))
                        .unwrap_or(true)
                }) {
                    for line in entry.lines.iter() {
                        let new_line = position_calculator.convert_line(line, LineType::Artcc);
                        lines.push(new_line);
                    }
                }

                for entry in sector.artcc_low_entries.iter().filter(|entry| {
                    asr.as_ref()
                        .map(|asr| asr.artcc_low_boundary.contains(&entry.name))
                        .unwrap_or(true)
                }) {
                    for line in entry.lines.iter() {
                        let new_line = position_calculator.convert_line(line, LineType::ArtccLow);
                        lines.push(new_line);
                    }
                }

                for entry in sector
                    .artcc_high_entries
                    .iter()
                    .filter(|entry| {
                        asr.as_ref()
                            .map(|asr| asr.artcc_high_boundary.contains(&entry.name))
                            .unwrap_or(true)
                    }) {
                        for line in entry.lines.iter() {
                            let new_line = position_calculator.convert_line(line, LineType::ArtccHigh);
                            lines.push(new_line);
                        }
                    };


                    for entry in sector
                    .low_airways
                    .iter()
                    .filter(|entry| {
                        asr.as_ref()
                            .map(|asr| asr.low_airways.contains(&entry.name))
                            .unwrap_or(true)
                    }) {
                        for line in entry.lines.iter() {
                            let new_line = position_calculator.convert_line(line, LineType::AirwayLow);
                            lines.push(new_line);
                        }
                    };

                    for entry in sector
                    .high_airways
                    .iter()
                    .filter(|entry| {
                        asr.as_ref()
                            .map(|asr| asr.high_airways.contains(&entry.name))
                            .unwrap_or(true)
                    }) {
                        for line in entry.lines.iter() {
                            let new_line = position_calculator.convert_line(line, LineType::AirwayHigh);
                            lines.push(new_line);
                        }
                    };

                    for entry in sector
                    .sid_entries
                    .iter()
                    .filter(|entry| {
                        asr.as_ref()
                            .map(|asr| asr.sids.contains(&entry.name))
                            .unwrap_or(true)
                    }) {
                        for line in entry.lines.iter() {
                            let new_line = position_calculator.convert_line(line, LineType::Sid);
                            lines.push(new_line);
                        }
                    };

                    for entry in sector
                    .star_entries
                    .iter()
                    .filter(|entry| {
                        asr.as_ref()
                            .map(|asr| asr.stars.contains(&entry.name))
                            .unwrap_or(true)
                    }) {
                        for line in entry.lines.iter() {
                            let new_line = position_calculator.convert_line(line, LineType::Star);
                            lines.push(new_line);
                        }
                    };

                    for entry in sector
                    .geo_entries
                    .iter()
                    .filter(|entry| {
                        asr.as_ref()
                            .map(|asr| asr.geo.contains(&entry.name))
                            .unwrap_or(true)
                    }) {
                        for line in entry.lines.iter() {
                            let new_line = position_calculator.convert_line(line, LineType::Geo);
                            lines.push(new_line);
                        }
                    };

                for fix in sector.fixes.iter().filter(|entry| {
                    asr.as_ref()
                        .map(|asr| asr.fixes.contains(&entry.identifier))
                        .unwrap_or(true)
                }) {
                    let fix_y = position_calculator.lat_to_window_y(fix.position.lat as f32);
                    let fix_x = position_calculator.lon_to_window_x(fix.position.lon as f32);
                    fixes.push(Fix { x: fix_x, y: fix_y });
                }

                for airport in sector.airports.iter().filter(|entry| {
                    asr.as_ref()
                        .map(|asr| asr.airports.contains(&entry.identifier))
                        .unwrap_or(true)
                }) {
                    let fix_y = position_calculator.lat_to_window_y(airport.position.lat as f32);
                    let fix_x = position_calculator.lon_to_window_x(airport.position.lon as f32);
                    airports.push(Airport { x: fix_x, y: fix_y });
                }

                for region_group in sector.regions.iter().filter(|entry| {
                    asr.as_ref()
                        .map(|asr| asr.regions.contains(&entry.name))
                        .unwrap_or(true)
                }) {
                    for polygon in region_group.regions.iter() {
                        let mut vertices = polygon
                            .vertices
                            .iter()
                            .map(|position| [position.lon as f32, position.lat as f32])
                            .flatten()
                            .collect::<Vec<_>>();
                        let indices = match earcutr::earcut(&vertices, &vec![], 2) {
                            Ok(indices) => indices,
                            Err(_) => continue,
                        };
                        vertices = vertices
                            .chunks_exact(2)
                            .map(|chunk| {
                                let x = position_calculator.lon_to_window_x(chunk[0]);
                                let y = position_calculator.lat_to_window_y(chunk[1]);
                                [x, y]
                            })
                            .flatten()
                            .collect();
                        let colour = Color::from_rgba(
                            polygon.colour.r,
                            polygon.colour.g,
                            polygon.colour.b,
                            255,
                        );
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
            }
            State::Loaded => {
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

                for airport in airports.iter() {
                    airport.draw();
                }

                // for fix in sector.fixes.iter() {
                //     let fix_y = position_calculator.lat_to_window_y(fix.position.lat as f32);
                //     let fix_x = position_calculator.lon_to_window_x(fix.position.lon as f32);
                //
                // }



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



pub struct Fix {
    pub x: f32,
    pub y: f32,
}
impl Fix {
    pub fn draw(&self) {
        draw_poly_lines(
            self.x,
            self.y,
            3,
            5.0,
            30.0,
            1.0,
            Color::from_rgba(38, 94, 97, 255),
        );
    }
}

pub struct Airport {
    pub x: f32,
    pub y: f32,
}
impl Airport {
    pub fn draw(&self) {
        draw_poly_lines(
            self.x,
            self.y,
            4,
            5.0,
            45.0,
            1.0,
            Color::from_rgba(38, 94, 97, 255),
        );
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


