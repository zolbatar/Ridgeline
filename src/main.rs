use crate::geo::load::{create_geo, load};
use crate::geo::paths::draw_all_paths;
use crate::gfx::sdl::Sdl;
use crate::gfx::skia::Skia;
use sdl2::event::Event;
use sdl2::mouse::{MouseButton, MouseWheelDirection};
use skia_safe::Point;
use std::process::exit;

mod app_state;
mod geo;
mod gfx;

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

const THRESHOLD: i32 = 64;

pub fn handle_mouse_wheel(skia: &mut Skia, _direction: MouseWheelDirection, precise_y: f32) {
    let delta = precise_y * 0.05;
    skia.zoom += delta;
    skia.zoom = skia.zoom.clamp(skia.zoom_min, skia.zoom_max);
}

pub fn handle_mouse_motion(skia: &mut Skia, centre: Point, x: i32, y: i32, x_rel: i32, y_rel: i32) {
    let mut mp = Point::new(x as f32, y as f32);
    mp.x -= centre.x as f32;
    mp.y -= centre.y as f32;
    mp.x /= skia.zoom;
    mp.y /= skia.zoom;
    mp.x += skia.target.x;
    mp.y += skia.target.y;
    if skia.panning {
        // Calculate mouse movement delta
        if x_rel.abs() < THRESHOLD && y_rel.abs() < THRESHOLD {
            // Update camera target based on mouse movement
            skia.target.x -= x_rel as f32 / skia.zoom;
            skia.target.y -= y_rel as f32 / skia.zoom;
        }
    } else {
        //terrain.hover(mp);
    }
}

pub fn handle_mouse_button_down(skia: &mut Skia, button: MouseButton) {
    if button == MouseButton::Right {
        skia.panning = true;
    } else if button == MouseButton::Left {
        //terrain.click(app_state.hover);
    }
}

pub fn handle_mouse_button_up(skia: &mut Skia, button: MouseButton) {
    if button == MouseButton::Right {
        skia.panning = false;
    }
}
