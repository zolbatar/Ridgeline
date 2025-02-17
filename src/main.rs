use crate::app_state::AppState;
use crate::game::player::{Player, PlayerType};
use crate::geo::boundary::draw_country;
use crate::geo::cities::draw_all_cities;
use crate::geo::dem::draw_dem;
use crate::geo::load::{create_geo, load};
use crate::geo::ways::draw_ways;
use crate::gfx::sdl::Sdl;
use crate::gfx::skia::Skia;
use crate::input::{handle_mouse_button_down, handle_mouse_button_up, handle_mouse_motion, handle_mouse_wheel};
use sdl2::event::Event;
use std::process::exit;
// https://osdatahub.os.uk/downloads/open/OpenRoads

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
    let geo_and_cities = load(25.0).expect("Failed to load geojson");

    // App state
    let mut app_state = AppState {
        players: vec![Player::new(PlayerType::NotAssigned), Player::new(PlayerType::Player)],
        selected_city: None,
    };

    // Create player(s)
    app_state.players[0].assign_all(&geo_and_cities);
    let city = app_state.players[0].cities.remove(0);
    app_state.selected_city = Some(city.clone());
    app_state.players[1].change_ownership(city);
    //    app_state.zoom_to_selected(&mut skia);

    loop {
        // Start of frame
        sdl.frame_start();
        skia.set_matrix(&sdl);
        skia.set_zoom_target(&sdl);
        draw_country(&mut skia, &geo_and_cities.geo_with_path);
        draw_ways(&mut skia, &geo_and_cities.ways);
        draw_dem(&mut skia, &geo_and_cities.dem);
        draw_all_cities(&mut skia, &app_state);

        // Events
        for event in sdl.event_loop.poll_iter() {
            match event {
                Event::Quit {
                    ..
                } => exit(0),

                Event::TextInput {
                    timestamp: _timestamp,
                    window_id: _window_id,
                    text,
                } => match text.to_uppercase().as_str() {
                    "Z" => app_state.zoom_to_selected(&mut skia),
                    "X" => app_state.zoom_out(&mut skia),
                    _ => {}
                },

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
