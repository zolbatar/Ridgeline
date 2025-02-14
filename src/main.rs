use crate::game::player::{Player, PlayerType};
use crate::geo::cities::draw_all_cities;
use crate::geo::load::{create_geo, load};
use crate::geo::paths::draw_all_paths;
use crate::gfx::sdl::Sdl;
use crate::gfx::skia::Skia;
use crate::input::{handle_mouse_button_down, handle_mouse_button_up, handle_mouse_motion, handle_mouse_wheel};
use sdl2::event::Event;
use std::collections::HashSet;
use std::process::exit;

mod app_state;
mod game;
mod geo;
mod gfx;
mod input;

fn main() {
    let mut sdl = Sdl::new();
    let mut skia = Skia::new(&sdl);

    // Create and load geo data
    create_geo();
    let wanted_regions: HashSet<u16> = HashSet::from([154, 39, 155]);
    //let wanted_regions: HashSet<u16> = HashSet::from([154, 39, 155, 151, 15, 145]);
    //let wanted_regions: HashSet<u16> = HashSet::from([53,143,30,151,419,15,21,154,35,34,39,202,145,155]);
    let geo_and_cities = load(&wanted_regions, 500.0).expect("Failed to load geojson");

    // Create player(s)
    let mut players = vec![Player::new(PlayerType::NotAssigned), Player::new(PlayerType::Player)];
    players[0].assign_all(&geo_and_cities);
    let city = players[0].cities.remove(0);
    players[1].change_ownership(city);

    loop {
        // Start of frame
        sdl.frame_start();
        skia.set_matrix(&sdl);
        skia.set_zoom_target(&sdl);
        draw_all_paths(&mut skia, &geo_and_cities.geo_with_path);
        draw_all_cities(&mut skia, &players);

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
