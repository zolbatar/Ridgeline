use crate::geo::load::load;
use crate::geo::paths::draw_all_paths;
use crate::gfx::sdl::Sdl;
use crate::gfx::skia::Skia;
use sdl2::event::Event;
use std::process::exit;

mod geo;
mod gfx;

fn main() {
    let mut sdl = Sdl::new();
    let mut skia = Skia::new(&sdl);

    //create_geo();
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
