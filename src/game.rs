use crate::interface::{Context, Mouse, View};
use map::GameMap;
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;

pub mod map;

pub fn launch(mut context: Context, game_map: GameMap) {
    let texture_creator = context.window.texture_creator();
    let mut world_view = View::new(
        &texture_creator,
        (0., 0.),
        context.get_window_size(),
        game_map,
        36,
    );
    let mut mouse = Mouse::new();

    'running: loop {
        // Render the view on the window

        world_view.render(&mut context.window);

        // Pull and handle all events
        let events = context.event_pump.poll_iter().collect::<Vec<Event>>();
        for event in events {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(..),
                    ..
                } => world_view.resize(context.get_window_size()),
                Event::Window {
                    win_event: WindowEvent::Leave,
                    ..
                } => {
                    mouse.reset();
                }
                Event::MouseMotion { x, y, .. } => {
                    if let (Some(vector), Some((MouseButton::Middle | MouseButton::Left, _))) =
                        (mouse.move_to((x, y)), mouse.get_click())
                    {
                        world_view.move_camera(vector);
                    };
                    if let Some((MouseButton::Right, _)) = mouse.get_click() {
                        let tile = world_view.get_world_pos(mouse.position.unwrap()).cell();
                        if world_view.get_map_mut().set_tile(tile, map::Tile::Dirt) {
                            world_view.pre_render(&mut context.window);
                        }
                    }
                }
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    mouse.click(mouse_btn, (x, y));
                    if let Some((MouseButton::Right, _)) = mouse.get_click() {
                        let tile = world_view.get_world_pos(mouse.position.unwrap()).cell();
                        if world_view.get_map_mut().set_tile(tile, map::Tile::Dirt) {
                            world_view.pre_render(&mut context.window);
                        }
                    }
                }
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    mouse.click_up(mouse_btn, (x, y));
                }
                Event::MouseWheel { y, .. } => {
                    let factor = if y > 0 { 1.12 } else { 1. / 1.12 };
                    if let Some(pos) = mouse.position {
                        world_view.zoom(factor, world_view.get_world_pos(pos));
                    }
                }
                _ => {}
            }
        }

        // Refresh window
        context.window.present();
        context.window.set_draw_color(Color::WHITE);
        context.window.clear();
    }
}
