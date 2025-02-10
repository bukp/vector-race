//#![windows_subsystem = "windows"]

mod interface;

use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use interface::{Mouse, View};

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("vector-race", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut mouse = Mouse::new();

    let mut window_size = (
        canvas.output_size().unwrap().0,
        canvas.output_size().unwrap().1,
    );
    let mut world_view = View::new((0., 0.), window_size, 24);

    'running: loop {
        let covered_cells = (
            world_view.get_cell_from_window((0, 0)),
            world_view.get_cell_from_window(world_view.get_size()),
        );

        let cell_size = world_view.get_cell_size();
        for x in covered_cells.0 .0..=covered_cells.1 .0 {
            for y in covered_cells.0 .1..=covered_cells.1 .1 {
                canvas.set_draw_color(if (x + y) % 2 == 0 {
                    Color::RGB(220, 220, 220)
                } else {
                    Color::WHITE
                });
                let (x, y) = world_view.get_window_pos(world_view.get_cell_world_pos((x, y)));
                canvas
                    .fill_rect(Rect::new(x, y, cell_size, cell_size))
                    .unwrap();
            }
        }

        if let Some(pos) = mouse.position {
            let cell = world_view.get_cell_from_window(pos);
            if covered_cells.0 .0 <= cell.0
                && cell.0 <= covered_cells.1 .0
                && covered_cells.0 .1 <= cell.1
                && cell.1 <= covered_cells.1 .1
            {
                let (x, y) = world_view.get_window_pos(world_view.get_cell_world_pos(cell));
                canvas.set_draw_color(Color::GRAY);
                canvas
                    .fill_rect(Rect::new(x, y, cell_size, cell_size))
                    .unwrap();
            }
        };

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(..),
                    ..
                } => {
                    window_size = (
                        canvas.output_size().unwrap().0,
                        canvas.output_size().unwrap().1,
                    );
                    world_view.resize(window_size)
                }
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

        canvas.present();
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();
    }
}
