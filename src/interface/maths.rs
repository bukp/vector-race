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

    /// Give the world position of the bottom right corner of the cell
    pub fn end_point(&self) -> WorldPosition {
        (self.0 as f32 + 1., self.1 as f32 + 1.).into()
    }
}

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

/// Distance between two points
pub fn dist(a: &WorldPosition, b: &WorldPosition) -> f32 {
    ((a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)).sqrt()
}

/// Vector formed by two points
pub fn vec(a: &WorldPosition, b: &WorldPosition) -> (f32, f32) {
    (b.0 - a.0, b.1 - a.1)
}

/// dot-product of two vectors
pub fn dot(v: &(f32, f32), w: &(f32, f32)) -> f32 {
    v.0 * w.0 + v.1 * w.1
}

/// Function retourning the signed distance to a segment, negative at the right of the line and negative at the left
/// 
/// If start and end are conbined, function will panic because there is an infinity of possible lines
pub fn sdf_segment(
    tested_point: &WorldPosition,
    start: &WorldPosition,
    end: &WorldPosition,
) -> f32 {
    // Calculate carthesian form (ax + bx + c = 0)
    let (a, b, c) = (
        end.1 - start.1,
        start.0 - end.0,
        start.1 * (end.0 - start.0) - start.0 * (end.1 - start.1),
    );

    // Distance to the line
    let d = (a * tested_point.0 + b * tested_point.1 + c) / (a * a + b * b).sqrt();

    // If one angle is obtuse, the closest point on the line won't be on the segment so it is either the start or the end
    if dot(&vec(start, end), &vec(start, tested_point)) <= 0.
        || dot(&vec(end, start), &vec(end, tested_point)) <= 0.
    {
        if d.abs() < 10e-6 {
            return f32::MAX;
        }

        // Min between the two
        dist(&tested_point, &start).min(dist(&tested_point, &end)) * d.signum()

    // If all angles are acute, the closest point is the closest on the line
    } else {
        d
    }
}

/// Function retourning the signed distance to a polygon, negative inside, positive outside
///
/// The polygon points have to be given clockwise or it will be inside-out and the first and last point of the polygon have to be the same for it to be close
///
/// Will fail if the polygon provided has less than two points
pub fn sdf_polygon(tested_point: &WorldPosition, polygon: &Vec<WorldPosition>) -> f32 {
    let mut last = polygon.get(0).unwrap();
    let mut dist = f32::MAX;
    let mut i = 1;
    while i < polygon.len() {
        let point = polygon.get(i).unwrap();
        let d = -sdf_segment(tested_point, last, point);
        last = point;

        if d.abs() < dist.abs() {
            dist = d;
        }

        i += 1
    }
    dist
}

/// Function retourning the signed distance to a group of polygon clockwise, negative inside, positive outside
///
/// An inverse polygon will be rendered as a hole
///
/// Will fail if any polygon is invalid
pub fn sdf_multiple_polygons(
    tested_point: &WorldPosition,
    polygons: &Vec<Vec<WorldPosition>>,
) -> f32 {
    let polygons = polygons;

    let mut dist = f32::MAX;
    for polygon in polygons {
        let d = sdf_polygon(&tested_point, polygon);

        if d.abs() < dist.abs() {
            dist = d;
        }
    }
    dist
}
