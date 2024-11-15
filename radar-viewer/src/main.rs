#![allow(unused)]

use std::{error::Error, fs::{create_dir_all, File}, io::{BufReader, BufWriter, Write}, net::TcpStream, ops::DerefMut, path::PathBuf, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender, TryRecvError}, Arc, Mutex}, thread};

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
mod logger;
mod api_link;



#[macroquad::main("Sauna Radar")]
async fn main() -> Result<(), Box<dyn Error>> {

    // Attempt to obtain exclusive write access to the lockfile, creating it if it is not there.
    // If this fails, another instance of this application is already running so we close.
    let lock_file_path = util::get_config_dir().unwrap().join(".radarlockfile");

    // Create directory
    create_dir_all(&lock_file_path)?;

    let mut lock_file = fd_lock::RwLock::new(File::create(lock_file_path).expect("Unable to create lock file"));
    let lock_file_guard = lock_file.try_write().expect("Another instance of this application is already running. Closing...");

    let program_wants_to_terminate: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let mut app = Application::new(Arc::clone(&program_wants_to_terminate))?;

    while !program_wants_to_terminate.load(Ordering::Relaxed) {
        app.update();
        app.draw();
        macroquad_profiler::profiler(Default::default());
        
        next_frame().await
    }

    Ok(())
}