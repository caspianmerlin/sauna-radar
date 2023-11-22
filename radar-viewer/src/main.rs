#![allow(unused)]

use std::{fs::File, io::{BufReader, BufWriter, Write}, sync::{mpsc::{self, Receiver, TryRecvError, Sender}, Mutex, Arc, atomic::{AtomicBool, Ordering}}, thread, net::TcpStream, path::PathBuf, ops::DerefMut, error::Error};

use app::Application;
use args::Args;
use asr::Asr;
use clap::Parser;
use fd_lock::{RwLockWriteGuard, RwLock};

use macroquad::{
    prelude::{Color, Vec2, BLACK, BLUE, WHITE},
    shapes::{draw_poly_lines, draw_triangle, draw_line},
    text::{draw_text, load_ttf_font_from_bytes, TextParams, draw_text_ex},
    window::{self, clear_background, next_frame}, ui::{root_ui, hash, widgets::{Window, Group, Button, InputText, ComboBox}, Layout, Skin}, input::is_key_pressed, miniquad::KeyCode, texture::Image, math::RectOffset, color::GREEN,
};

use sct_reader::reader::SctReader;
use sector::Sector;

mod args;
mod asr;
mod radar;
mod sector;
mod console;
mod aircraft;
mod util;
mod app;
mod ipc;





#[macroquad::main("Sauna Radar")]
async fn main() -> Result<(), Box<dyn Error>> {

    // Attempt to obtain exclusive write access to the lockfile, creating it if it is not there.
    // If this fails, another instance of this application is already running so we close.
    let lock_file_path = util::get_config_dir().unwrap().join(".radarlockfile");
    let mut lock_file = fd_lock::RwLock::new(File::create(lock_file_path).expect("Unable to create lock file"));
    let lock_file_guard = lock_file.try_write().expect("Another instance of this application is already running. Closing...");

    let mut app = Application::new()?;

    loop {


        



        // if radar_display_manager.is_initialised() {
        //     if is_key_pressed(KeyCode::F1) {
        //         show_help = !show_help;
        //     }
        //     if is_key_pressed(KeyCode::Enter) {
        //         if let Some(text_command) = try_parse_text_command(&text_input, &aircraft) {
        //             ipc_message_sender.send(IpcMessage::TextCommand(text_command)).ok();
        //             text_input.clear();
        //         }

        //         root_ui().set_input_focus(txt_input_id);
        //     }
        //     if is_key_pressed(KeyCode::Tab) {
        //         if !text_input.is_empty() {
        //             let input = text_input.to_uppercase();
        //             for aircraft in aircraft.iter() {
        //                 if aircraft.callsign.contains(&input) {
        //                     text_input = format!("{}, ", aircraft.callsign);
        //                     break;
        //                 }
        //             }
        //         }
                
        //     }
        //     else if is_key_pressed(KeyCode::F11) {
        //         full_screen = !full_screen;
        //         window::set_fullscreen(full_screen);
        //     }
        //     else if is_key_pressed(KeyCode::F6) {
        //         radar_display_manager.cycle_back();
        //     }
        //     else if is_key_pressed(KeyCode::F7) {
        //         radar_display_manager.cycle();
        //     }
        //     radar_display_manager.update(&mut aircraft);

        //     clear_background(radar_display_manager.background_colour());
        //     radar_display_manager.draw(&mut aircraft);

        //     if show_help {
        //         draw_text("F1 - Show / hide help    F2 - Toggle FMS lines    F3 - Filters    F6 - Previous display    F7 - Next display    F11 - Toggle fullscreen", 10., 20.0, 20., WHITE);
        //     }
        // }

        // else {
        //     clear_background(radar_display_manager.background_colour());
        //     if let Ok(new_radar_displays) = rx.try_recv() {
        //         let radar_displays = new_radar_displays.into_iter().map(|(sector, colours, zoom_level)| {
        //             RadarDisplay::new(sector, zoom_level, colours)
        //         }).collect();
        //         radar_display_manager.store(radar_displays);
        //     }
        // }

        
        // root_ui().push_skin(&editbox_skin);
        // InputText::new(txt_input_id).position(Vec2::new(10.0, window::screen_height() - 30.0)).size(Vec2::new(window::screen_width() - 20., 20.0))
        // .ui(root_ui().deref_mut(), &mut text_input);
        // root_ui().pop_skin();

        app.update();
        app.draw();
        macroquad_profiler::profiler(Default::default());
        
        next_frame().await
    }
}


// fn try_parse_text_command(txt: &str, aircraft_list: &Vec<AircraftRecord>) -> Option<TextCommandRequest> {
//     let mut split = txt.split(&[',', ' ']).filter(|x| !x.is_empty());
//     let callsign = split.next()?.to_string();
//     if !aircraft_list.iter().any(|aircraft| aircraft.callsign == callsign) {
//         return None;
//     }
//     let command = split.next()?.to_string();
//     let args = split.map(|arg| arg.to_owned()).collect::<Vec<_>>();
//     let request = TextCommandRequest {
//         callsign,
//         command,
//         args
//     };
//     Some(request)
// }