use std::collections::{HashMap, HashSet};

use sdl2::mouse::MouseButton;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

use crate::game::map::{GameMap, Tile};
use maths::{sdf_multiple_polygons, Side};

pub use maths::{Cell, WindowPosition, WorldPosition};

pub mod maths;

const PRE_RENDERING_CELL_SIZE: u32 = 128;

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

        let canvas = window
            .into_canvas()
            .present_vsync()
            .accelerated()
            .build()
            .unwrap();
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
pub struct View<'a> {
    start_point: WorldPosition, // Top right corner in world position
    cam_size: (u16, u16),       // Size of the view
    game_map: GameMap,
    cell_size: u16, // Size of the cell's representation in pixel on the window
    texture: (
        &'a TextureCreator<WindowContext>,
        Option<Texture<'a>>,
        Option<(Cell, Cell)>,
    ), // Texture creator, actual texture, texture cell coverage
}

impl<'a> View<'a> {
    /// Create a new View
    pub fn new<'b, T: Into<WorldPosition>>(
        texture_creator: &'b TextureCreator<WindowContext>,
        start_pos: T,
        cam_size: (u32, u32),
        game_map: GameMap,
        cell_size: u32,
    ) -> View<'b> {
        View {
            start_point: start_pos.into(),
            cam_size: (
                cam_size.0.try_into().unwrap(),
                cam_size.1.try_into().unwrap(),
            ),
            game_map,
            cell_size: cell_size.try_into().unwrap(),
            texture: (texture_creator, None, None),
        }
    }

    /// Render the view on the given canvas
    pub fn render(&mut self, canvas: &mut Canvas<Window>) {
        if self.texture.1.is_none() {
            self.pre_render();
        }

        let tex = self.texture.1.as_ref().unwrap();

        let cell_coverage = self.texture.2.unwrap();

        let start = self.get_window_pos(cell_coverage.0.start_point());

        let end = self.get_window_pos(cell_coverage.1.end_point());

        canvas
            .copy(
                tex,
                None,
                Rect::new(
                    start.0,
                    start.1,
                    (end.0 - start.0) as u32,
                    (end.1 - start.1) as u32,
                ),
            )
            .unwrap();
    }

    /// Render the view on a texture, the given canvas is only used for creating the texture
    pub fn pre_render(&mut self) {
        // Start cell (top left) and last cell (bottom right)
        let mut cell_range = ((0, 0), (0, 0));

        for ((x, y), _) in self.game_map.iter_tiles() {
            if x < cell_range.0 .0 {
                cell_range.0 .0 = x
            };
            if x > cell_range.1 .0 {
                cell_range.1 .0 = x
            };
            if y < cell_range.0 .1 {
                cell_range.0 .1 = y
            };
            if y > cell_range.1 .1 {
                cell_range.1 .1 = y
            };
        }

        self.texture.2 = Some((Cell::from(cell_range.0), Cell::from(cell_range.1)));

        let texture_size = (
            (cell_range.1 .0 - cell_range.0 .0 + 1) as u32 * PRE_RENDERING_CELL_SIZE,
            (cell_range.1 .1 - cell_range.0 .1 + 1) as u32 * PRE_RENDERING_CELL_SIZE,
        );

        let mut texture = self
            .texture
            .0
            .create_texture(
                None,
                sdl2::render::TextureAccess::Streaming,
                texture_size.0,
                texture_size.1,
            )
            .unwrap();

        let mut border_cells = HashSet::new();

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

        // Vector containing all the borders
        let mut borders_list: HashMap<Tile, Vec<Vec<WorldPosition>>> = HashMap::new();

        // Just help for readibility
        fn move_cell(cell: Cell, dir: Side) -> Cell {
            Cell::from((cell.0 + dir.dir().0, cell.1 + dir.dir().1))
        }

        while !border_cells.is_empty() {
            let mut border = Vec::new();

            let first = *border_cells.iter().next().unwrap();
            let tile = self.game_map.get_tile(first.0);

            let mut current = first;
            border.push(current);

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

            let border = border
                .into_iter()
                .map(|x| {
                    let (inside, outside) = (x.0.center_point(), x.1.center_point());
                    WorldPosition::from(((inside.0 + outside.0) / 2., (inside.1 + outside.1) / 2.))
                })
                .collect();

            if borders_list.contains_key(&tile) {
                borders_list.get_mut(&tile).unwrap().push(border);
            } else {
                borders_list.insert(tile, vec![border]);
            }
        }

        let starting_world_pos = Cell::from(cell_range.0).start_point();

        texture
            .with_lock(None, |buffer, pitch| {
                for x in 0..texture_size.0 {
                    'l: for y in 0..texture_size.1 {
                        let world_pos = WorldPosition::from((
                            starting_world_pos.0 + x as f32 / PRE_RENDERING_CELL_SIZE as f32,
                            starting_world_pos.1 + y as f32 / PRE_RENDERING_CELL_SIZE as f32,
                        ));

                        let offset = (y as usize * pitch) + (x as usize * 4);

                        for (tile, borders) in &borders_list {
                            let dist = sdf_multiple_polygons(&world_pos, borders);

                            if dist <= 0. {
                                let a = (((-dist * 10.).sin() * 64.).round()) as u8;
                                let color = tile.tile_color();
                                buffer[offset] = color.b;
                                buffer[offset + 1] = color.g + a;
                                buffer[offset + 2] = color.r;
                                buffer[offset + 3] = 255;

                                continue 'l;
                            }
                        }
                        buffer[offset] = 255;
                        buffer[offset + 1] = 255;
                        buffer[offset + 2] = 255;
                        buffer[offset + 3] = 255;
                    }
                }
            })
            .unwrap();

        self.texture.1 = Some(texture);
    }

    pub fn get_map_mut(&mut self) -> &mut GameMap {
        &mut self.game_map
    }

    /// Slide the view by a vector in pixel representing the slide on the window
    pub fn move_camera(&mut self, vector: (i32, i32)) {
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
