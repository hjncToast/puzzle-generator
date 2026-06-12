use crate::parser::{Orbit, Puzzle, Move};
use crate::moves::{apply_cycle_to_orbit};
use std::path::Path;
use crate::state::State;


//lehmer encoding: turn permutation into integer
pub fn lehmer_encode(perm: &[u8]) -> u32 {
    let n = perm.len();
    let mut rank = 0u32;
    let mut factorial = 1u32;

    for i in (0..n).rev() {
        let smaller = perm[i+1..].iter().filter(|&&x| x < perm[i]).count();
        rank += smaller as u32 * factorial;
        factorial *= (n - i) as u32;
    }
    rank
}


pub fn lehmer_decode(mut rank: u32, n: usize) -> Vec<u8> {
    let mut available: Vec<u8> = (0..n as u8).collect();
    let mut perm = vec![0u8; n];

    let mut factorial = (1..n as u32).product::<u32>(); // (n-1)!

    for i in 0..n {
        let idx = (rank / factorial) as usize;
        perm[i] = available.remove(idx);
        rank %= factorial;
        if factorial > 1 { factorial /= (n - i - 1) as u32; }
    }
    perm
}

// orientation encoding: turn orientation array into integer
pub fn orientation_encode(orient: &[u8], modulus: u8) -> u32 {
    orient.iter().fold(0u32, |acc, &o| acc * modulus as u32 + o as u32)
}

pub fn orientation_decode(mut val: u32, n: usize, modulus: u8) -> Vec<u8> {
    let mut orient = vec![0u8; n];
    for i in (0..n).rev() {
        orient[i] = (val % modulus as u32) as u8;
        val /= modulus as u32;
    }
    orient
}

//used to see if the amount of pieces can fit into u16 move tables
pub fn check_coordinate_sizes(orbits: &[Orbit]) -> Result<(), String> {
    const MAX_STATES: u32 = 65535; // u16 max

    for orbit in orbits {
        // permutation is orbit.size factorial!
        let perm_states: u32 = (1..=orbit.size as u32).product();
        if perm_states > MAX_STATES {
            return Err(format!(
                "Orbit {} has {}! = {} permutation states, exceeds u16 limit. Split this orbit into smaller groups.",
                orbit.id, orbit.size, perm_states
            ));
        }

        // orientation is orbit.modulus ^ orbit.size
        let orient_states = (orbit.modulus as u32).pow(orbit.size as u32);
        if orient_states > MAX_STATES {
            return Err(format!(
                "Orbit {} has {}^{} = {} orientation states, exceeds u16 limit. Split this orbit into smaller groups.",
                orbit.id, orbit.modulus, orbit.size, orient_states
            ));
        }
    }

    Ok(())
}




pub struct PuzzleCoords {
    pub perm: Vec<u16>,   // one per orbit
    pub orient: Vec<u16>, // one per orbit
}

//use the encoding functions to encode the perm/orient within each orbit within the puzzle
pub fn encode_state(state: &State, puzzle: &Puzzle) -> PuzzleCoords {
    let perm = state.permutation.iter()
        .map(|p| lehmer_encode(p) as u16)
        .collect();

    let orient = state.orientation.iter().enumerate()
        .map(|(i, o)| orientation_encode(o, puzzle.orbits[i].modulus) as u16)
        .collect();

    PuzzleCoords { perm, orient }
}


pub struct CoordinateGroup {
    pub orbit_id: u8,
    pub piece_indices: Vec<u8>,
}

//this function is mainly used if the amount of pieces is too big to fit into the u16 size
//build the groups
pub fn build_coordinate_groups(orbits: &[Orbit]) -> Vec<CoordinateGroup> {
    let mut groups = Vec::new();

    for orbit in orbits {
        let pieces: Vec<u8> = (0..orbit.size).collect();

        // check if full orbit fits without splitting
        let full_perm: u32 = (1..=orbit.size as u32).product();
        if full_perm <= u16::MAX as u32 {
            groups.push(CoordinateGroup {
                orbit_id: orbit.id,
                piece_indices: pieces,
            });
            continue;
        }

        //if bigger, 50/50 split. Doesn't work for over 16 pieces tho duh

        // 50/50 split
        let half = orbit.size as usize / 3;
        groups.push(CoordinateGroup {
            orbit_id: orbit.id,
            piece_indices: pieces[..half].to_vec(),
        });
        groups.push(CoordinateGroup {
            orbit_id: orbit.id,
            piece_indices: pieces[half..].to_vec(),
        });
    }

    groups
}



fn max_group_size() -> usize {
    let mut n = 1;
    let mut factorial: u32 = 1;
    loop {
        let next = factorial.saturating_mul((n + 1) as u32);
        if next > u16::MAX as u32 { return n; }
        factorial = next;
        n += 1;
    }
}

//for a 4x4+ with lots of xcentres, the 24 xcentres overflow in groups of 8, but 12 3x3 edges overflow as 8 then 4
//otherwise same as build coordinate groups function
pub fn build_coordinate_groups_eight_overflow(orbits: &[Orbit]) -> Vec<CoordinateGroup> {
    let max_size = max_group_size();

    orbits.iter().flat_map(|orbit| {
        let pieces: Vec<u8> = (0..orbit.size).collect();
        pieces.chunks(max_size)
            .map(|chunk| CoordinateGroup {
                orbit_id: orbit.id,
                piece_indices: chunk.to_vec(),
            })
            .collect::<Vec<_>>()
    }).collect()
}



pub fn build_move_table(
    group: &CoordinateGroup,
    puzzle: &Puzzle,
) -> Result<Vec<u16>, String> {
    let orbit = &puzzle.orbits[group.orbit_id as usize];
    let n = group.piece_indices.len();

    // safety check
    let num_states: u32 = (1..=n as u32).product();
    if num_states > u16::MAX as u32 {
        return Err(format!(
            "Coordinate group on orbit {} has {} states, exceeds u16 limit.",
            group.orbit_id, num_states
        ));
    }
    let num_states = num_states as usize;
    let num_moves = puzzle.moves.len();

    //table size is equal to states * moves. Every move not just the core 6 RUFBLD moves. so for 3x3 that's 54 moves
    let mut table = vec![0u16; num_states * num_moves];

    for coord in 0..num_states {
        //from 0 -> number of states. Take a state integer, turn it into a permutation, see what all possible moves do to the state
        //then re-encode that resulting state and add it all to the table
        let local_perm = lehmer_decode(coord as u32, n);

        // build the permutation from decoded coord, using the correct pieces of that group (incase there is more that fit in u16)
        //groups of orbits (like 12 edges split up) use their own IDs
        let mut full_perm: Vec<u8> = (0..orbit.size).collect();
        for (local_idx, &piece_idx) in group.piece_indices.iter().enumerate() {
            full_perm[piece_idx as usize] = group.piece_indices[local_perm[local_idx] as usize];
        }
        let mut full_orient = vec![0u8; orbit.size as usize];

        for (move_id, mv) in puzzle.moves.iter().enumerate() {
            let mut temp_perm = full_perm.clone();
            let mut temp_orient = full_orient.clone();
            apply_move_to_orbit(&mut temp_perm, &mut temp_orient, mv, group.orbit_id as usize, orbit);

            // for each move, track what it does and reencode the state using the correct IDs
            let result_local: Vec<u8> = group.piece_indices.iter()
                .map(|&slot| {
                    let piece_there = temp_perm[slot as usize];
                    group.piece_indices.iter().position(|&p| p == piece_there).unwrap() as u8
                })
                .collect();

            table[coord * num_moves + move_id] = lehmer_encode(&result_local) as u16;
        }
    }

    Ok(table)
}



pub fn apply_move_to_orbit(perm: &mut Vec<u8>, orient: &mut Vec<u8>, mv: &Move, orbit_id: usize, orbit: &Orbit) {
    for cycle in mv.cycles.iter().filter(|c| c.orbit_id as usize == orbit_id) {
        apply_cycle_to_orbit(perm, orient, cycle, orbit);
    }
}



pub fn save_table(table: &[u16], path: &str) {
    let bytes: &[u8] = bytemuck::cast_slice(table);
    std::fs::write(path, bytes).unwrap();
}

pub fn load_table(path: &str) -> Option<Vec<u16>> {
    if !Path::new(path).exists() { return None; }
    let bytes = std::fs::read(path).unwrap();
    Some(bytemuck::cast_slice(&bytes).to_vec())
}

pub fn get_or_build_table(group: &CoordinateGroup, puzzle: &Puzzle, path: &str) -> Vec<u16> {
    std::fs::create_dir_all("cache").unwrap();
    if let Some(table) = load_table(path) {
        println!("loaded table from {}", path);
        return table;
    }
    println!("building table for {}...", path);
    let table = build_move_table(group, puzzle).unwrap();
    save_table(&table, path);
    table
}

//let table = get_or_build_table(&group, &pzl, "cache/orbit0_perm.bin");

// Storage
// Flat Vec<u16> indexed as table[coord * num_moves + move_id] is fastest for cache access since the solver knows the coord and iterates over moves:
// rust// convert nested vec to flat on the way out
// let flat: Vec<u16> = table.into_iter().flatten().collect();

// // lookup
// let new_coord = flat[coord * num_moves + move_id];
// And to cache to disk it's just raw bytes:
// rust// write
// let bytes: &[u8] = bytemuck::cast_slice(&flat);
// std::fs::write("cp_move_table.bin", bytes).unwrap();

// // read back
// let bytes = std::fs::read("cp_move_table.bin").unwrap();
// let flat: Vec<u16> = bytemuck::cast_slice(&bytes).to_vec();
// bytemuck is the standard crate for this — zero-copy reinterpretation of byte slices as typed slices.





// What's left in this file

// Combination+permutation encoding (binomial, combination_encode/decode) — this is the actual blocker. Without it, any orbit >8 pieces can't build a valid move table at all (your current panic).
// Replace/extend build_move_table to use the new encoding for subset groups — full-orbit groups (≤8) can keep using plain Lehmer as a fast path, subset groups use combo+perm.
// Update build_coordinate_groups to choose group size based on C(n,k)*k! fitting u16, not just n!.
// Pruning tables (BFS) — straightforward once move tables work, can come after.

// see how move tables affect x centres
// definition file will say what is 'the same' then the coordinate builder should use combination encoding for those pieces not perm encoding