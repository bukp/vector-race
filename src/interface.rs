use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::video::Window;
use sdl2::EventPump;

use crate::game::map::GameMap;

/// Represent all the context needed to access the window
pub struct Context {
    pub window: Canvas<Window>,
    pub event_pump: EventPump,
}

impl Context {
    /// Initialise everything to get sdl ready to draw on screen
    pub fn init() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("vector-race", 800, 600)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            window: canvas,
            event_pump,
        }
    }

    /// Return the size of the window in pixel
    pub fn get_window_size(&self) -> (u32, u32) {
        self.window.output_size().unwrap()
    }
}

/// Structure representing the camera viewing the world, used to render it and interact with it (click, slide, zoom)
#[derive(Debug, Clone)]
pub struct View {
    start_point: (f32, f32), // Top right corner in world position
    cam_size: (u16, u16),    // Size of the view
    game_map: GameMap,
    cell_size: u16,          // Size of the cell's representation in pixel on the window
}

impl View {
    /// Create a new View
    pub fn new(start_pos: (f32, f32), cam_size: (u32, u32), game_map: GameMap, cell_size: u32) -> Self {
        View {
            start_point: start_pos,
            cam_size: (
                cam_size.0.try_into().unwrap(),
                cam_size.1.try_into().unwrap(),
            ),
            game_map,
            cell_size: cell_size.try_into().unwrap(),
        }
    }

    /// Render the view on the given canvas
    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>, mouse: &Mouse) {
        
        // Determine which cells are visible from the view
        let covered_cells = (
            self.get_cell_from_window((0, 0)),
            self.get_cell_from_window(self.get_size()),
        );

        let cell_size = self.get_cell_size();
        
        // Render each cell to the canvas
        for x in covered_cells.0 .0..=covered_cells.1 .0 {
            for y in covered_cells.0 .1..=covered_cells.1 .1 {

                // Get the right color to draw the cell
                canvas.set_draw_color(self.game_map.get_tile((x, y)).tile_color());

                // Draw with a different color if the mouse is on
                if let Some(pos) = mouse.position {
                    if self.get_cell_from_window(pos) == (x, y) {
                        canvas.set_draw_color(Color::RED);
                    }
                }
                
                // Draw the cell at the correct location
                let (x, y) = self.get_window_pos(self.get_cell_world_pos((x, y)));
                canvas
                    .fill_rect(Rect::new(x, y, cell_size, cell_size))
                    .unwrap();
            }
        }
    }

    /// Slide the view by a vector in pixel representing the slide on the window
    pub fn slide(&mut self, vector: (i32, i32)) {
        let vector = (
            vector.0 as f32 / self.cell_size as f32,
            vector.1 as f32 / self.cell_size as f32,
        );
        self.start_point = (self.start_point.0 + vector.0, self.start_point.1 + vector.1);
    }

    /// Return the size in pixel of the view
    pub fn get_size<T: From<u16>>(&self) -> (T, T) {
        (self.cam_size.0.into(), self.cam_size.0.into())
    }

    /// Return the current size in pixel of a cell on the window
    pub fn get_cell_size<T: From<u16>>(&self) -> T {
        self.cell_size.into()
    }

    /// Resize the view around the start_point (top left corner) with a new size in pixels
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.cam_size = (
            new_size.0.try_into().unwrap(),
            new_size.1.try_into().unwrap(),
        )
    }

    /// Zoom in/out the view with the given zoom_factor around the center_point.
    ///
    /// A zoom_factor > 1 will zoom in and a zoom_factor < 1 will zoom out.
    pub fn zoom(&mut self, zoom_factor: f32, center_point: (f32, f32)) {
        assert!(zoom_factor > 0., "use of negative zoom factor");

        //Calculate new size of cells on display in pixel
        let cs = (self.cell_size as f32 * zoom_factor).round() as u16;
        let cs = u16::clamp(cs, 8, 80);

        //Calculate the real scale factor if the new cell size has been clamped (scale factor = 1 / zoom factor)
        let scale_factor = self.cell_size as f32 / cs as f32;

        //Apply new size
        self.cell_size = cs;

        //Replace the camera to make the cursor point to the same world location
        self.start_point = (
            center_point.0 + (self.start_point.0 - center_point.0) * scale_factor,
            center_point.1 + (self.start_point.1 - center_point.1) * scale_factor,
        );

        // Replace the start point to avoid graphic glithes
        self.start_point = (
            (self.start_point.0 * self.cell_size as f32).round() / self.cell_size as f32,
            (self.start_point.1 * self.cell_size as f32).round() / self.cell_size as f32
        )
    }

    /// Calculate the world position in pixel corresponding to a pixel position on the window
    pub fn get_world_pos(&self, window_pos: (i32, i32)) -> (f32, f32) {
        (
            window_pos.0 as f32 / self.cell_size as f32 + self.start_point.0,
            window_pos.1 as f32 / self.cell_size as f32 + self.start_point.1,
        )
    }

    /// Calculate the window position in pixels given a world position
    pub fn get_window_pos(&self, world_pos: (f32, f32)) -> (i32, i32) {
        (
            ((world_pos.0 - self.start_point.0) * self.cell_size as f32).round() as i32,
            ((world_pos.1 - self.start_point.1) * self.cell_size as f32).round() as i32,
        )
    }

    /// Return top left corner's world position of a cell
    pub fn get_cell_world_pos(&self, cell_pos: (i32, i32)) -> (f32, f32) {
        (cell_pos.0 as f32, cell_pos.1 as f32)
    }

    /// Calculate cell indices from window position
    pub fn get_cell_from_window(&self, window_pos: (i32, i32)) -> (i32, i32) {
        let pos = self.get_world_pos(window_pos);
        (pos.0.floor() as i32, pos.1.floor() as i32)
    }
}

/// Structure representing the mouse for the game to keep track of the inputs
#[derive(Debug, Clone)]
pub struct Mouse {
    pub clicked: Option<(MouseButton, (i32, i32))>, // Current clicked button and the window position it was clicked
    pub position: Option<(i32, i32)>, // Current window position of the mouse, None if the mouse is outisde the window
                                      // If the mouse is clicked, the mouse should be garanteed to have a position (Some)
}

impl Mouse {
    /// Create a new Mouse
    pub fn new() -> Self {
        Mouse {
            clicked: None,
            position: None,
        }
    }

    /// Move the mouse to a new position,
    ///
    /// Return a vector of the mouse movement from las position if the mouse was already in the window
    pub fn move_to(&mut self, pos: (i32, i32)) -> Option<(i32, i32)> {
        let vec = if let Some((x, y)) = self.position {
            Some((x - pos.0, y - pos.1))
        } else {
            None
        };
        self.position = Some(pos);
        vec
    }

    /// Return the clicked button and the position it was clicked
    pub fn get_click(&self) -> Option<(MouseButton, (i32, i32))> {
        self.clicked
    }

    /// Simulate a mouse click
    pub fn click(&mut self, button: MouseButton, pos: (i32, i32)) {
        self.clicked = Some((button, (pos.0, pos.1)));
        self.position = Some((pos.0, pos.1));
    }

    /// Simulate the end of a mouse click
    pub fn click_up(&mut self, mouse_button: MouseButton, pos: (i32, i32)) -> Option<(i32, i32)> {
        self.position = Some(pos);
        if let Some((btn, (x, y))) = self.clicked {
            if btn == mouse_button {
                self.clicked = None;
                return Some((x - pos.0, y - pos.1));
            }
        }
        None
    }

    /// Reset the mouse informations
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
