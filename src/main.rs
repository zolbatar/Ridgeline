use crate::geo::load::{create_geo, load};
use crate::geo::paths::draw_all_paths;
use crate::gfx::sdl::Sdl;
use crate::gfx::skia::Skia;
use crate::input::{handle_mouse_button_down, handle_mouse_button_up, handle_mouse_motion, handle_mouse_wheel};
use sdl2::event::Event;
use std::process::exit;

mod app_state;
mod geo;
mod gfx;
mod input;

fn main() {
    let mut sdl = Sdl::new();
    let mut skia = Skia::new(&sdl);

    create_geo();
    let paths = load().expect("Failed to load geojson");

    loop {
        // Start of frame
        sdl.frame_start();
        skia.set_matrix(&sdl);
        skia.set_zoom_target(&sdl);
        draw_all_paths(&mut skia, &paths);

        // Events
        for event in sdl.event_loop.poll_iter() {
            match event {
                Event::Quit {
                    ..
                } => exit(0),

                Event::MouseWheel {
                    direction,
                    precise_y,
                    ..
                } => {
                    handle_mouse_wheel(&mut skia, direction, precise_y);
                }

                Event::MouseButtonDown {
                    mouse_btn,
                    ..
                } => {
                    handle_mouse_button_down(&mut skia, mouse_btn);
                }
                Event::MouseButtonUp {
                    mouse_btn,
                    ..
                } => {
                    handle_mouse_button_up(&mut skia, mouse_btn);
                }

                Event::MouseMotion {
                    x,
                    y,
                    xrel,
                    yrel,
                    ..
                } => {
                    handle_mouse_motion(&mut skia, sdl.centre, x, y, xrel, yrel);
                }

                _ => {}
            }
        }

        // Finish up
        skia.set_matrix(&sdl);
        sdl.show_fps(&mut skia);
        unsafe {
            skia.flush();
        }
        sdl.frame_end();
    }
}
