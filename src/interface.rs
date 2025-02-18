use std::collections::HashSet;

use sdl2::mouse::MouseButton;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::video::Window;
use sdl2::EventPump;

use crate::game::map::{GameMap, Tile};
use crate::utils::Side;

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

/// Represent a position on the window in pixel, therefore is most of the time non-negative
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPosition(pub i32, pub i32);

impl From<(i32, i32)> for WindowPosition {
    fn from(x: (i32, i32)) -> Self {
        WindowPosition(x.0, x.1)
    }
}

/// Represent an in world position, the integer part is the cell's position
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPosition(pub f32, pub f32);

impl From<(f32, f32)> for WorldPosition {
    fn from(x: (f32, f32)) -> Self {
        WorldPosition(x.0, x.1)
    }
}

impl WorldPosition {
    /// Calculate cell indices corresponding to world position
    pub fn cell(&self) -> Cell {
        (self.0.floor() as i32, self.1.floor() as i32).into()
    }
}

/// Represent a cell's position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell(pub i32, pub i32);

impl From<(i32, i32)> for Cell {
    fn from(x: (i32, i32)) -> Self {
        Cell(x.0, x.1)
    }
}

impl Cell {
    /// Give the world position of the top left corner of the cell
    pub fn start_point(&self) -> WorldPosition {
        (self.0 as f32, self.1 as f32).into()
    }

    /// Give the world position of the center of the cell
    pub fn center_point(&self) -> WorldPosition {
        (self.0 as f32 + 0.5, self.1 as f32 + 0.5).into()
    }
}

/// Structure representing the camera viewing the world, used to render it and interact with it (click, slide, zoom)
#[derive(Debug, Clone)]
pub struct View {
    start_point: WorldPosition, // Top right corner in world position
    cam_size: (u16, u16),       // Size of the view
    game_map: GameMap,
    cell_size: u16, // Size of the cell's representation in pixel on the window
}

impl View {
    /// Create a new View
    pub fn new<T: Into<WorldPosition>>(
        start_pos: T,
        cam_size: (u32, u32),
        game_map: GameMap,
        cell_size: u32,
    ) -> Self {
        View {
            start_point: start_pos.into(),
            cam_size: (
                cam_size.0.try_into().unwrap(),
                cam_size.1.try_into().unwrap(),
            ),
            game_map,
            cell_size: cell_size.try_into().unwrap(),
        }
    }

    /// Render the view on the given canvas
    pub fn render<T: RenderTarget>(&self, canvas: &mut Canvas<T>) {

        let mut border_cells = HashSet::new();

        // Render each cell to the canvas
        for ((x, y), _tile_type) in self.game_map.iter_tiles() {
            if let Tile::Empty = self.game_map.get_tile((x, y)) {
                continue;
            }

            let current_tile = self.game_map.get_tile((x, y));

            // Look for each surrounding tile from top to bottom and left to right
            let mut surrounding = [(Tile::Empty, (0, 0)); 4];

            // Get every surrounding tile and its position
            for (i, p) in Side::iter_dir().enumerate() {
                surrounding[i] = (
                    self.game_map.get_tile((x + p.0, y + p.1)),
                    (x + p.0, y + p.1),
                );
            }

            // Filter the borders for the ones with two different cells (which is the point of a border)
            for (_, p) in surrounding.into_iter().filter(|x| x.0 != current_tile) {
                border_cells.insert((Cell::from((x, y)), Cell::from(p)));
            }
        }

        // Just help for readibility
        fn move_cell(cell: Cell, dir: Side) -> Cell {
            Cell::from((cell.0 + dir.dir().0, cell.1 + dir.dir().1))
        }

        while !border_cells.is_empty() {
            let mut border = Vec::new();

            let first = *border_cells.iter().next().unwrap();
            let tile = self.game_map.get_tile(first.0);

            let mut current = first;

            // the current direction to follow the border with the right hand (from the inside)
            let mut current_dir =
                Side::from_dir((current.1 .0 - current.0 .0, current.1 .1 - current.0 .1))
                    .unwrap()
                    .turn_left();

            loop {
                // Get rid of that border so that it isn't inspected again
                border_cells.remove(&current);
                
                // If the cell to the right is in, the area, we must turn and move
                if self
                    .game_map
                    .get_tile(move_cell(current.0, current_dir.turn_right()))
                    == tile
                {
                    current_dir = current_dir.turn_right();
                    current.0 = move_cell(current.0, current_dir);
                    
                    // Otherwise, if the cell in front of the current onr is in the area, move on
                } else if self.game_map.get_tile(move_cell(current.0, current_dir)) == tile {
                    current.0 = move_cell(current.0, current_dir);

                // Finally, if it isn't in al well, turn left
                } else {
                    current_dir = current_dir.turn_left();
                }
                
                // recalculate the border faced cell
                current.1 = move_cell(current.0, current_dir.turn_right());

                // Add the tile to the border list in the order if it is a border
                if self.game_map.get_tile(current.1) != tile {
                    border.push(current);
                }
                
                if current == first {
                    break;
                }
            }

            let (inside, outside) = (first.0.center_point(), first.1.center_point());
            let mut last_point = WorldPosition::from(((inside.0 + outside.0)/2., (inside.1 + outside.1)/2.));
            // Render the border
            for (cell_in, cell_out) in border {
                let (inside, outside) = (cell_in.center_point(), cell_out.center_point());
                let point = WorldPosition::from(((inside.0 + outside.0)/2., (inside.1 + outside.1)/2.));
                
                canvas.set_draw_color(tile.tile_color());

                let start_point = self.get_window_pos(last_point);
                let end_point = self.get_window_pos(point);
                canvas.draw_line((start_point.0, start_point.1), (end_point.0, end_point.1)).unwrap();


                last_point = point
            }
        }
    }

    pub fn get_map_mut(&mut self) -> &mut GameMap {
        &mut self.game_map
    }

    /// Slide the view by a vector in pixel representing the slide on the window
    pub fn slide(&mut self, vector: (i32, i32)) {
        let vector = (
            vector.0 as f32 / self.cell_size as f32,
            vector.1 as f32 / self.cell_size as f32,
        );
        self.start_point = (self.start_point.0 + vector.0, self.start_point.1 + vector.1).into();
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
    pub fn zoom<T: Into<WorldPosition>>(&mut self, zoom_factor: f32, center_point: T) {
        assert!(zoom_factor > 0., "use of negative zoom factor");
        let center_point = center_point.into();

        //Calculate new size of cells on display in pixel
        let cs = (self.cell_size as f32 * zoom_factor).round() as u16;
        let cs = u16::clamp(cs, 8, 80);

        //Calculate the real scale factor if the new cell size has been clamped (scale factor = 1 / zoom factor)
        let scale_factor = self.cell_size as f32 / cs as f32;

        //Apply new size
        self.cell_size = cs;

        //Replace the camera to make the cursor point to the same world location
        let start_point = (
            center_point.0 + (self.start_point.0 - center_point.0) * scale_factor,
            center_point.1 + (self.start_point.1 - center_point.1) * scale_factor,
        );

        // Replace the start point to avoid graphic glithes
        self.start_point = (
            (start_point.0 * self.cell_size as f32).round() / self.cell_size as f32,
            (start_point.1 * self.cell_size as f32).round() / self.cell_size as f32,
        )
            .into()
    }

    /// Calculate the world position in pixel corresponding to a pixel position on the window
    pub fn get_world_pos<T: Into<WindowPosition>>(&self, window_pos: T) -> WorldPosition {
        let window_pos: WindowPosition = window_pos.into();
        (
            window_pos.0 as f32 / self.cell_size as f32 + self.start_point.0,
            window_pos.1 as f32 / self.cell_size as f32 + self.start_point.1,
        )
            .into()
    }

    /// Calculate the window position in pixels given a world position
    pub fn get_window_pos<T: Into<WorldPosition>>(&self, world_pos: T) -> WindowPosition {
        let world_pos = world_pos.into();
        (
            ((world_pos.0 - self.start_point.0) * self.cell_size as f32).round() as i32,
            ((world_pos.1 - self.start_point.1) * self.cell_size as f32).round() as i32,
        )
            .into()
    }
}

/// Structure representing the mouse for the game to keep track of the inputs
#[derive(Debug, Clone)]
pub struct Mouse {
    pub clicked: Option<(MouseButton, (i32, i32))>, // Current clicked button and the window position it was clicked
    pub position: Option<(i32, i32)>, // Current window position of the mouse, None if the mouse is outisde the window
                                      // If the mouse is clicked, the mouse should be guaranteed to have a position (Some)
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
