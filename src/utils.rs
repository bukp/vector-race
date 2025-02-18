/// Represent the different sides of a square
#[derive(Debug, Clone, Copy)]
pub enum Side {
    Up,
    Right,
    Down,
    Left,
}

impl Side {
    pub fn dir(&self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Right => (1, 0),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
        }
    }

    pub fn from_dir(dir: (i32, i32)) -> Result<Self, ()> {
        match dir {
            (0, -1) => Ok(Self::Up),
            (1, 0) => Ok(Self::Right),
            (0, 1) => Ok(Self::Down),
            (-1, 0) => Ok(Self::Left),
            _ => Err(()),
        }
    }

    pub fn turn_right(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }

    pub fn turn_left(&self) -> Self {
        match self {
            Self::Right => Self::Up,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
            Self::Up => Self::Left,
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Up, Self::Right, Self::Down, Self::Left].into_iter()
    }

    pub fn iter_dir() -> impl Iterator<Item = (i32, i32)> {
        Self::iter().map(|x| x.dir())
    }
}

/// Same as a 'Side' but also include diagonals
#[derive(Debug, Clone, Copy)]
pub enum Directions {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    Left,
    DownLeft,
    UpLeft,
}

impl Directions {
    pub fn dir(&self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::UpRight => (1, -1),
            Self::Right => (1, 0),
            Self::DownRight => (1, 1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::DownLeft => (-1, 1),
            Self::UpLeft => (-1, -1),
        }
    }

    pub fn from_dir(dir: (i32, i32)) -> Result<Self, ()> {
        match dir {
            (0, -1) => Ok(Self::Up),
            (1, -1) => Ok(Self::UpRight),
            (1, 0) => Ok(Self::Right),
            (1, 1) => Ok(Self::DownRight),
            (0, 1) => Ok(Self::Down),
            (-1, 0) => Ok(Self::Left),
            (-1, 1) => Ok(Self::DownLeft),
            (-1, -1) => Ok(Self::UpLeft),
            _ => Err(()),
        }
    }

    pub fn turn_right(&self) -> Self {
        match self {
            Self::Up => Self::UpRight,
            Self::UpRight => Self::Right,
            Self::Right => Self::DownRight,
            Self::DownRight => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::DownLeft,
            Self::DownLeft => Self::UpLeft,
            Self::UpLeft => Self::Up,
        }
    }

    pub fn turn_left(&self) -> Self {
        match self {
            Self::UpRight => Self::Up,
            Self::Right => Self::UpRight,
            Self::DownRight => Self::Right,
            Self::Down => Self::DownRight,
            Self::Left => Self::Down,
            Self::DownLeft => Self::Left,
            Self::UpLeft => Self::DownLeft,
            Self::Up => Self::UpLeft,
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::Up,
            Self::UpRight,
            Self::Right,
            Self::DownRight,
            Self::Down,
            Self::Left,
            Self::DownLeft,
            Self::UpLeft,
        ]
        .into_iter()
    }

    pub fn iter_dir() -> impl Iterator<Item = (i32, i32)> {
        Self::iter().map(|x| x.dir())
    }
}
