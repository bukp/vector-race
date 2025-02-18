//#![windows_subsystem = "windows"]
#![allow(dead_code)]

pub mod game;
pub mod interface;
pub mod utils;

fn main() {
    let game_map = game::map::GameMap::generate_from_file(std::path::Path::new("maps\\test.trk"))
        .expect("unable to load map");

    let context = interface::Context::init();
    game::launch(context, game_map);
}
