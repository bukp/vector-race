use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use sdl2::pixels::Color;

use crate::interface::Cell;

/// Represent a tile on the map and all its properties
///
/// Prototype, miss a lot of features for now
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum Tile {
    Empty,
    Road,
    Wall,
    Gravel,
    Dirt,
    Ice,
}

impl Tile {
    /// Create a tile from a character, is used to genarate a map from a file
    pub fn read_tile(tile_char: &str) -> Result<Self, String> {
        Ok(match tile_char {
            " " => Self::Empty,
            "W" => Self::Wall,
            "R" => Self::Road,
            "G" => Self::Gravel,
            "D" => Self::Dirt,
            "I" => Self::Ice,
            x => {
                return Err(match x.len() {
                    0 => "nul character".to_string(),
                    1 => "unknown character ".to_string() + x,
                    _ => "too many characters".to_string(),
                })
            }
        })
    }

    /// Get the color for a tile, is very likely to be replaced by some kind of texture
    pub fn tile_color(&self) -> Color {
        match self {
            Self::Empty => Color::WHITE,
            Self::Road => Color::GREY,
            Self::Wall => Color::BLACK,
            Self::Gravel => Color::RGB(100, 100, 100),
            Self::Dirt => Color::RED,
            Self::Ice => Color::BLUE,
        }
    }
}

/// Represent the map and the objects/players on it
#[derive(Debug, Clone)]
pub struct GameMap {
    terrain: HashMap<Cell, Tile>,
    default_tile: Tile,
}

impl GameMap {
    /// Create a new empty map
    pub fn empty() -> Self {
        GameMap {
            terrain: HashMap::new(),
            default_tile: Tile::Empty,
        }
    }

    /// Set tile infos, return true if the tile has been changed (new is not equal to former)
    pub fn set_tile<T: Into<Cell>>(&mut self, position: T, tile: Tile) -> bool {
        match self.terrain.insert(position.into(), tile) {
            None => true,
            Some(x) if x != tile => true,
            _ => false,
        }
    }

    /// Get tile infos from its position
    pub fn get_tile<T: Into<Cell>>(&self, position: T) -> Tile {
        match self.terrain.get(&position.into()) {
            Some(x) => *x,
            None => self.default_tile,
        }
    }

    /// Generate a new map from a file
    ///
    /// Prototype, miss a lot of features for now
    pub fn generate_from_file(path: &Path) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|x| x.to_string())?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|x| x.to_string())?;

        let mut map = GameMap::empty();
        for (y, i) in content.split('\n').enumerate() {
            for (x, j) in i.trim().split('|').enumerate() {
                map.set_tile(
                    (x as i32, y as i32),
                    Tile::read_tile(j).map_err(|err| format!("{} at ({};{})", err, x, y))?,
                );
            }
        }

        Ok(map)
    }

    /// Iterate over every tile in the map
    pub fn iter_tiles(&self) -> impl Iterator<Item = ((i32, i32), &Tile)> {
        self.terrain.iter().map(|x| ((x.0 .0, x.0 .1), x.1))
    }
}
