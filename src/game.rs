use crate::interface::{Context, Mouse, View};
use map::GameMap;
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;

pub mod map;

pub fn launch(mut context: Context, game_map: GameMap) {
    let mut world_view = View::new((0., 0.), context.get_window_size(), game_map, 24);
    let mut mouse = Mouse::new();

    'running: loop {

        // Render the view on the window
        world_view.render(&mut context.window, &mouse);

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
                        world_view.slide(vector);
                    }
                }
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    mouse.click(mouse_btn, (x, y));
                }
                Event::MouseButtonUp {
                    mouse_btn, x, y, ..
                } => {
                    mouse.click_up(mouse_btn, (x, y));
                }
                Event::MouseWheel { y, .. } => {
                    let factor = if y > 0 { 1.12 } else { 1. / 1.12 };
                    world_view.zoom(factor, world_view.get_world_pos(mouse.position.unwrap()));
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
