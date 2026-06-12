use crate::parser::{Puzzle, Orbit};



#[derive(Debug)]
pub struct State {
    // one vec per orbit, indexed by orbit_id
    pub permutation: Vec<Vec<u8>>,
    pub orientation: Vec<Vec<u8>>,
}

impl State {
    pub fn solved(puzzle: &Puzzle) -> Self {
        State {
            permutation: puzzle.orbits.iter().map(|o| o.permutation.clone()).collect(),
            orientation: puzzle.orbits.iter().map(|o| o.orientation.clone()).collect(),
        }
    }
}