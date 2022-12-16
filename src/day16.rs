use std::collections::HashMap;
use priority_queue::PriorityQueue;

type InputType = Caves;
type OutputType = u64;

const TIME_LIMIT: u64 = 30;

#[derive(Debug)]
pub struct Valve {
  name: String,
  flow: u64,
  next: Vec<String>,
}

impl Valve {
  fn parse(line: &str) -> Self {
    let words: Vec<&str> = line.split_whitespace().collect();
    let name = words[1].to_string();
    // remove "rate=" and ";"
    let mut flow_str = words[4][5..].to_string();
    flow_str.pop();
    let flow = flow_str.parse::<u64>().unwrap();
    let next: Vec<String> = words[9..].join(" ").split(", ").map(|s| s.to_string()).collect();
    Valve{name, flow, next}
  }
}

#[derive(Debug)]
pub struct Caves {
  start: usize,
  flows: Vec<u64>,
  /// Distance[from][dest]
  distances: Vec<Vec<u8>>,
}

impl Caves {
  fn parse(input: &str) -> Self {
    let valves: Vec<Valve> = input.lines().map(|l| Valve::parse(l)).collect();
    let mut map: HashMap<&str, usize> = HashMap::new();
    let mut flows: Vec<u64> = vec![0; valves.len()];
    let mut distances = vec![vec![valves.len() as u8; valves.len()]; valves.len()];
    for (i, v) in valves.iter().enumerate() {
      map.insert(&v.name, i);
      flows[i] = v.flow;
    }
    let start = *map.get("AA").unwrap();
    // Set the distance to 1 for every direct connection
    for (from, v) in valves.iter().enumerate() {
      distances[from][from] = 0;
      for dest in v.next.iter().map(|n| *map.get(n.as_str()).unwrap()) {
        distances[from][dest] = 1;
      }
    }
    // compute the shortest distances for all pairs
    for i in 0..distances.len() {
      for j in 0..distances.len() {
        for k in 0..distances.len() {
          distances[j][k] = u8::min(distances[j][k], distances[j][i] + distances[i][k]);
        }
      }
    }
    Caves{start, flows, distances}
  }

  fn print_distances(&self) {
    for row in &self.distances {
      for dist in row {
        print!("{dist:3}");
      }
      println!();
    }
  }
}

pub fn generator(input: &str) -> InputType {
  Caves::parse(input)
}

#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct State {
  /// bit map of the valves we could open
  shut: u64,
  location: usize,
  remaining_time: u64,
  total_flow: u64,
}

impl State {
  fn new(input: &Caves) -> Self {
    let shut = input.flows.iter().enumerate()
      .filter(|(_, &f)| f > 0)
      .fold(0, |acc, (i, _)| acc | (1 << i));
    State{shut, location: input.start, remaining_time: TIME_LIMIT, total_flow: 0}
  }

  fn move_to(&self, dest: usize, caves: &Caves) -> Self {
    let shut = self.shut & !(1 << dest);
    let remaining_time = self.remaining_time - caves.distances[self.location][dest] as u64 - 1;
    let total_flow = self.total_flow + remaining_time * caves.flows[dest];
    State{location: dest, shut, remaining_time, total_flow}
  }

  fn next(&self, caves: &Caves) -> Vec<Self> {
    let mut result = Vec::new();
    let mut closed_valves = self.shut;
    while closed_valves != 0 {
      let zeros = closed_valves.trailing_zeros() as usize;
      if caves.distances[self.location][zeros] as u64 + 1 < self.remaining_time {
        result.push(self.move_to(zeros, caves));
      }
      closed_valves &= !(1 << zeros);
    }
    result
  }
}

pub fn part1(input: &InputType) -> OutputType {
  let mut queue = PriorityQueue::new();
  queue.push(State::new(input), 0);
  let mut max = 0;
  while !queue.is_empty() {
    let (state, _) = queue.pop().unwrap();
    let next = state.next(input);
    if next.is_empty() {
      max = max.max(state.total_flow);
    } else {
      for next_state in next {
        let next_flow = next_state.total_flow;
        queue.push(next_state, next_flow);
      }
    }
  }
  max
}

pub fn part2(_: &InputType) -> OutputType {
  0
}

#[cfg(test)]
mod tests {
  use crate::day16::{generator, part1};


  #[test]
  fn test_part1() {
    assert_eq!(1651, part1(&generator(INPUT)));
  }

  #[test]
  fn test_part2() {
    //assert_eq!(93, part2(&generator(INPUT)));
  }

  const INPUT: &str =
"Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II";
}