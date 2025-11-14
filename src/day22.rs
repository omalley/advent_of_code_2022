use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use itertools::Itertools;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
enum Spot {
  Wall,
  Floor,
  Void,
}

impl Spot {
  fn parse(c: char) -> Self {
    match c {
      '#' => Spot::Wall,
      '.' => Spot::Floor,
      ' ' => Spot::Void,
      _ => panic!("Can't parse '{c}'"),
    }
  }
}

#[derive(Debug)]
enum Move {
  Left,
  Right,
  Forward(i32),
}

#[derive(Debug)]
struct Map {
  walls: Vec<Vec<Spot>>,
  size: (i32, i32),
  face_size: i32,
}

impl Map {
  fn parse(input: &str) -> Self {
    let walls:Vec<Vec<Spot>> = input.lines()
      .map(|l| l.chars().map(Spot::parse).collect())
      .collect();
    let width = walls.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
    let height = walls.len() as i32;
    // Flattened cubes are either 3x4 or 2x5
    let long_side = width.max(height);
    let short_side = width.min(height);
    if long_side / short_side >= 2 {
      Map{walls, size: (width, height), face_size: short_side / 2}
    } else {
      Map{walls, size: (width, height), face_size: short_side / 3}
    }
  }

  /// Wrap coordinates within the map's ranges
  fn wrap_locations(&self, xy: (i32, i32)) -> (i32,i32) {
    (xy.0.rem_euclid(self.size.0), xy.1.rem_euclid(self.size.1))
  }

  fn get(&self, x: i32, y: i32) -> Spot {
    if (0..self.size.0).contains(&x) && (0..self.size.1).contains(&y) {
      let row = &self.walls[y as usize];
      if x < row.len() as i32 {
        return row[x as usize];
      }
    }
    Spot::Void
  }
}

#[derive(Debug)]
pub struct InputType {
  map: Map,
  moves: Vec<Move>,
}

impl InputType {

  fn parse_moves(line: &str) -> Vec<Move> {
    line.trim().chars().group_by(|ch| ch.is_ascii_digit())
      .into_iter()
      .map(| (is_digit, mut itr) |
        if is_digit {
          Move::Forward(itr.collect::<String>().parse::<i32>().unwrap())
        } else {
          match itr.next() {
            Some('R') => Move::Right,
            Some('L') => Move::Left,
            x => panic!("Can't parse {:?}", x),
      }}).collect()
  }
}

type OutputType = i32;

pub fn generator(input: &str) -> InputType {
  let (board_text, move_text) = input.split_once("\n\n").unwrap();
  InputType{map: Map::parse(board_text),
    moves: InputType::parse_moves(move_text)}
}

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
enum Direction {
  Right,
  Down,
  Left,
  Up,
}

impl Direction {
  fn left(&self) -> Self {
    match self {
      Direction::Up => Direction::Left,
      Direction::Left => Direction::Down,
      Direction::Down => Direction::Right,
      Direction::Right => Direction::Up,
    }
  }

  fn right(&self) -> Self {
    match self {
      Direction::Up => Direction::Right,
      Direction::Left => Direction::Up,
      Direction::Down => Direction::Left,
      Direction::Right => Direction::Down,
    }
  }

  fn delta(&self) -> (i32, i32) {
    match self {
      Direction::Up => (0, -1),
      Direction::Left => (-1, 0),
      Direction::Down => (0, 1),
      Direction::Right => (1, 0),
    }
  }

  fn advance(&self, xy: (i32, i32)) -> (i32, i32) {
    let delta = self.delta();
    (xy.0 + delta.0, xy.1 + delta.1)
  }

  fn flip(&self) -> Self {
    match self {
      Direction::Up => Direction::Down,
      Direction::Down => Direction::Up,
      Direction::Left => Direction::Right,
      Direction::Right => Direction::Left,
    }
  }
}

/// A map coordinate & facing direction for describing the source and destination of the jumps
/// through the void.
#[derive(Clone,Debug,Eq,Hash,PartialEq)]
struct VoidConnection {
  xy: (i32, i32),
  facing: Direction,
}

#[derive(Debug)]
struct State<Func>
  where Func: Fn(VoidConnection) -> VoidConnection,
{
  x: i32,
  y: i32,
  facing: Direction,
  void_jump: Func,
}

impl<Func> State<Func>
  where Func: Fn(VoidConnection) -> VoidConnection,
{
  fn new(input: &InputType, void_jump: Func) -> Self {
    let x = input.map.walls[0].iter().enumerate()
      .find(|(_, &s)| s == Spot::Floor).unwrap().0 as i32;
    State{x, y: 0, facing: Direction::Right, void_jump}
  }

  fn forward(&mut self, distance: i32, map: &Map) {
    for _ in 0..distance {
      let mut next = VoidConnection{xy: (self.x, self.y), facing: self.facing};
      next.xy = next.facing.advance(next.xy);
      loop {
        match map.get(next.xy.0, next.xy.1) {
          Spot::Floor => {
            (self.x, self.y) = next.xy;
            self.facing = next.facing;
            break;
          },
          Spot::Wall => return,
          Spot::Void => next = (self.void_jump)(next),
        }
      }
    }
  }

  fn execute(&mut self, mv: &Move, map: &Map) {
    match mv {
      Move::Left => self.facing = self.facing.left(),
      Move::Right => self.facing = self.facing.right(),
      Move::Forward(n) => self.forward(*n, map),
    }
  }

  fn score(&self) -> i32 {
    1000 * (self.y + 1) + 4 * (self.x + 1) + self.facing as i32
  }
}

fn part1_void_jump(src: &VoidConnection, map: &Map) -> VoidConnection {
  let mut next = map.wrap_locations(src.facing.advance(src.xy));
  while map.get(next.0, next.1) == Spot::Void {
    next = map.wrap_locations(src.facing.advance(next));
  }
  VoidConnection{xy: next, facing: src.facing}
}

pub fn part1(input: &InputType) -> OutputType {
  let mut state =
    State::new(input, |s| part1_void_jump(&s, &input.map));
  for mv in &input.moves {
    state.execute(mv, &input.map);
  }
  state.score()
}

/// Iterators that will follow an outside edge.
struct EdgeIterator<'a> {
  cube: &'a Cube<'a>,
  /// x-y in the map
  map_xy: (i32, i32),
  /// the direction we are currently moving
  moving: Direction,
  /// The edge that we are following.
  /// It is always 90 degrees from the moving direction.
  following: Direction,
  /// Did we just make a turn?
  made_turn: bool,
}

impl<'a> EdgeIterator<'a> {
  fn next(&mut self) -> Option<VoidConnection> {
    let result = VoidConnection{xy: self.map_xy, facing: self.following.flip()};
    let (x,y) = self.moving.advance(self.map_xy);
    let following = self.following.advance((x, y));
    // Do we need to turn?
    if self.cube.map.get(following.0, following.1) == Spot::Void {
      self.made_turn = true;
      (self.moving, self.following) = (self.following, self.moving.flip());
      self.map_xy = following;
    } else if self.cube.map.get(x, y) != Spot::Void {
      // Have we reached an inside turn? If so, we've lapped the cube and should stop.
      return None
    } else {
      // Otherwise, we are just continuing straight
      self.made_turn = false;
      self.map_xy = (x, y);
    }
    Some(result)
  }
}

impl<'a> Debug for EdgeIterator<'a> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "EdgeIterator{{ map_xy: {:?}, moving: {:?}, following: {:?}, made_turn: {} }}",
           self.map_xy, self.moving, self.following, self.made_turn)
  }
}

/// An iterator that will traverse two directions from an inner corner to find the pairs of
/// edges that should connect together.
#[derive(Debug)]
struct JoinIterator<'a> {
  left: EdgeIterator<'a>,
  right: EdgeIterator<'a>,
  is_done: bool,
}

impl<'a> JoinIterator<'a> {
  fn new(cube: &'a Cube, xy: (i32, i32), vert: Direction, horiz: Direction) -> Self {
    let x = if horiz == Direction::Left {
      xy.0 * cube.face_size
    } else {
      (xy.0 + 1) * cube.face_size - 1
    };
    let y = if vert == Direction::Up {
      xy.1 * cube.face_size
    } else {
      (xy.1 + 1) * cube.face_size - 1
    };
    let h_itr = EdgeIterator{cube, map_xy: (x, y),
      moving: horiz.flip(), following: vert, made_turn: false};
    let v_itr = EdgeIterator{cube, map_xy: (x, y),
      moving: vert.flip(), following: horiz, made_turn: false};
    JoinIterator{left: h_itr, right: v_itr, is_done: false}
  }

  fn next(&mut self) -> Option<(VoidConnection, VoidConnection)> {
    if !self.is_done {
      if let Some(left) = self.left.next() {
        if let Some(right) = self.right.next() {
          self.is_done = self.left.made_turn && self.right.made_turn;
          return Some((left, right));
        }
      }
    }
    None
  }
}

#[derive(Debug)]
struct Cube<'a> {
  map: &'a Map,
  // a map of where on the flattened map is an edge
  overview: Vec<Vec<bool>>,
  overview_size: (i32, i32),
  face_size: i32,
}

impl<'a> Cube<'a> {
  fn new(map: &'a Map) -> Self {
    let mut overview = Vec::new();
    for y in (0..map.size.1).step_by(map.face_size as usize) {
      let mut row = Vec::new();
      for x in (0..map.size.0).step_by(map.face_size as usize) {
        if map.get(x,y) != Spot::Void {
          row.push(true);
        } else {
          row.push(false);
        }
      }
      overview.push(row);
    }
    Cube{map, overview, face_size: map.face_size,
      overview_size: (map.size.0/map.face_size, map.size.1/map.face_size)}
  }

  /// Get the face at the given position in the overview if one is there.
  fn get(&self, x: i32, y:i32) -> bool {
    if (0..self.overview_size.0).contains(&x) && (0..self.overview_size.1).contains(&y) {
      return self.overview[y as usize][x as usize]
    }
    false
  }

  /// Find all of the inside corners in the overview, because those represent the ends of the
  /// cuts that flattened the cube. Return a list of JoinIterators that will cover the
  /// perimeter.
  fn find_inside_corners(&self) -> Vec<JoinIterator<'_>> {
    let mut result = Vec::new();
    for y in 0..self.overview_size.1 {
      for x in 0..self.overview_size.0 {
        if !self.get(x, y) {
          if self.get(x, y - 1) {
            if self.get(x - 1, y) {
              result.push(JoinIterator::new(self, (x, y),
                                         Direction::Up, Direction::Left));
            } else if self.get(x + 1, y) {
              result.push(JoinIterator::new(self, (x, y),
                                         Direction::Up, Direction::Right));
            }
          } else if self.get(x, y + 1) {
            if self.get(x - 1, y) {
              result.push(JoinIterator::new(self, (x, y),
                                         Direction::Down, Direction::Left));
            } else if self.get(x + 1, y) {
              result.push(JoinIterator::new(self, (x, y),
                                         Direction::Down, Direction::Right));
            }
          }
        }
      }
    }
    result
  }

  fn build_wrap_map(&self) -> HashMap<VoidConnection, VoidConnection> {
    let mut result = HashMap::new();
    for mut start in self.find_inside_corners() {
      while let Some((left, right)) = start.next() {
        let left_target = right.facing.flip();
        result.insert(left.clone(),
                      VoidConnection{xy: left_target.advance(right.xy),
                        facing: left_target});
        let right_target = left.facing.flip();
        result.insert(right.clone(),
                      VoidConnection{xy: right_target.advance(left.xy),
                        facing: right_target});
      }
    }
    result
  }
}

pub fn part2(input: &InputType) -> OutputType {
  let cube = Cube::new(&input.map);
  let wrap_map = cube.build_wrap_map();
  let mut state =
    State::new(input, |s| wrap_map.get(&s).unwrap().clone());
  for mv in &input.moves {
    state.execute(mv, &input.map);
  }
  state.score()
}

#[cfg(test)]
mod tests {
  use crate::day22::{generator, part1, part2};

  #[test]
  fn test_part1() {
    let input = generator(INPUT);
    assert_eq!(6032, part1(&input));
  }

  #[test]
  fn test_part2() {
    assert_eq!(5031, part2(&generator(INPUT)));
  }

  const INPUT: &str =
"        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

10R5L5R10L4R5L5
";
}
