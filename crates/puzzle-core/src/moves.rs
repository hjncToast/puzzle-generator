use crate::parser::{Puzzle, Move, Orbit, Cycle};
use crate::state::{State};




//apply a cycle to one orbit
pub fn apply_cycle_to_orbit(
    perm: &mut Vec<u8>,
    orient: &mut Vec<u8>,
    cycle: &Cycle,
    orbit: &Orbit,
) {
      let slots = &cycle.slots_to_permute;

      let last_perm = perm[*slots.last().unwrap() as usize];
      let last_orient = orient[*slots.last().unwrap() as usize];

      for i in (1..slots.len()).rev() {
          perm[slots[i] as usize] = perm[slots[i-1] as usize];
          orient[slots[i] as usize] = (orient[slots[i-1] as usize] as i8 - cycle.orientation_offset[i])
              .rem_euclid(orbit.modulus as i8) as u8;
      }

      perm[slots[0] as usize] = last_perm;
      orient[slots[0] as usize] = (last_orient as i8 - cycle.orientation_offset[0])
          .rem_euclid(orbit.modulus as i8) as u8;
}


//regular apply move (apply all cycles to their respective orbits)
pub fn apply_move(state: &mut State, mv: &Move, puzzle: &Puzzle) {
    for cycle in &mv.cycles {
        let orbit = &puzzle.orbits[cycle.orbit_id as usize];
        let o = cycle.orbit_id as usize;
        apply_cycle_to_orbit(&mut state.permutation[o], &mut state.orientation[o], cycle, orbit);
    }
}




// pub fn apply_move(state: &mut State, mv: &Move, puzzle: &Puzzle) {
//     for cycle in &mv.cycles {
//       println!("{:#?}", cycle);
//         let orbit = &puzzle.orbits[cycle.orbit_id as usize];
//         let o = cycle.orbit_id as usize;
//         let slots = &cycle.slots_to_permute;

//         // save the last piece, shift everything forward, place last at front
//         let last_perm = state.permutation[o][*slots.last().unwrap() as usize];
//         let last_orient = state.orientation[o][*slots.last().unwrap() as usize];

//         for i in (1..slots.len()).rev() {
//             state.permutation[o][slots[i] as usize] = state.permutation[o][slots[i-1] as usize];
//             state.orientation[o][slots[i] as usize] = (state.orientation[o][slots[i-1] as usize] as i8 - cycle.orientation_offset[i]).rem_euclid(orbit.modulus as i8) as u8
//         }

//         state.permutation[o][slots[0] as usize] = last_perm;
//         state.orientation[o][slots[0] as usize] = (last_orient as i8 - cycle.orientation_offset[0]).rem_euclid(orbit.modulus as i8) as u8
//     }
// }

pub fn apply_alg(state: &mut State, moves_str: &str, puzzle: &Puzzle) {
    for name in moves_str.split_whitespace() {
        if let Some(mv) = puzzle.moves.iter().find(|m| m.name == name) {
            apply_move(state, mv, puzzle);
        } else {
            println!("Warning: move '{}' not found", name);
        }
    }
}