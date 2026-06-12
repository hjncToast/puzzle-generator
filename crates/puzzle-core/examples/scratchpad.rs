use puzzle_core::{PieceInfo, initialise_pieces, build_union_find, build_orbits, build_moves, build_puzzle};
use std::collections::HashMap;
use puzzle_core::state::{State};
use puzzle_core::moves::{apply_move, apply_alg};
use puzzle_core::visualise::{visualise, visualise_generic, PrintType};
use puzzle_core::movetable::{check_coordinate_sizes, lehmer_encode, lehmer_decode, orientation_decode, orientation_encode, get_or_build_table, build_coordinate_groups};

fn main() {
    let drone = "
    U: (UF UL UB UR) (URF UFL ULB UBR)
    D: (DF DR DB DL) (DFR DRB DBL DLF)
    R: (UR BR DR FR) (URF-1 UBR+1 DRB-1 DFR+1)
    F: (UF+1 FR+1 DF+1 FL+1) (URF+1 DFR-1 DLF+1 UFL-1)
    L: (UL FL DL BL) (UFL+1 DLF-1 DBL+1 ULB-1)
    B: (UB+1 BL+1 DB+1 BR+1) (UBR-1 ULB+1 DBL-1 DRB+1)
    u: (UF UL UB UR) (URF UFL ULB UBR) (FR+1 FL+1 BL+1 BR+1) (R F L B)
    r: (UR BR DR FR) (URF-1 UBR+1 DRB-1 DFR+1) (UF+1 UB+1 DB+1 DF+1) (F U B D)
    f: (UF+1 FR+1 DF+1 FL+1) (URF+1 DFR-1 DLF+1 UFL-1) (UR+1 DR+1 DL+1 UL+1) (U R D L)
    d: (DF DR DB DL) (DFR DRB DBL DLF) (FR+1 BR+1 BL+1 FL+1) (F R B L)
    l: (UL FL DL BL) (UFL+1 DLF-1 DBL+1 ULB-1) (UF+1 DF+1 DB+1 UB+1) (F D B U)
    b: (UB+1 BL+1 DB+1 BR+1) (UBR-1 ULB+1 DBL-1 DRB+1) (UR+1 UL+1 DL+1 DR+1) (U L D R)
    M: (UF+1 DF+1 DB+1 UB+1) (U F D B)
    S: (UR+1 DR+1 DL+1 UL+1) (U R D L)
    E: (FR+1 BR+1 BL+1 FL+1) (F R B L)
    x: (UR BR DR FR) (URF-1 UBR+1 DRB-1 DFR+1) (UL BL DL FL) (UFL+1 ULB-1 DBL+1 DLF-1) (UF+1 UB+1 DB+1 DF+1) (F U B D)
    y: (UF UL UB UR) (URF UFL ULB UBR) (DF DL DB DR) (DFR DLF DBL DRB) (FR+1 FL+1 BL+1 BR+1) (R F L B)
    z: (UF+1 FR+1 DF+1 FL+1) (URF+1 DFR-1 DLF+1 UFL-1) (UB+1 BR+1 DB+1 BL+1) (UBR-1 DRB+1 DBL-1 ULB+1) (UR+1 DR+1 DL+1 UL+1) (U R D L)
    ";

    // match initialise_pieces(drone.trim()) {
    //     Ok(map) => {
    //         let mut pieces: Vec<(&&str, &PieceInfo)> = map.iter().collect();

    //         // 2. Sort by the numeric index value (0, 1, 2...)
    //         pieces.sort_by_key(|&(_, info)| info.index);

    //         println!("--- Pieces Sorted by Index (Order of Discovery) ---");
    //         // 3. Loop over the sorted vector
    //         for (name, info) in pieces {
    //             println!(
    //                 "Piece: {:<12} | Index: {:<2} | Modulus: {}",
    //                 name, info.index, info.modulus
    //             );
    //         }
    //     }
    //     Err(err) => println!("Error: {}", err),
    // }

    //println!("{:#?}", initialise_pieces(drone));


    //let mut pieces = initialise_pieces(drone.trim()).unwrap();

    // //println!("\n=== UNION FIND DISCOVERY ===");
    //let mut uf = build_union_find(drone.trim(), &pieces);

    // //println!("{:#?}", pieces.len());

    // //println!("{:#?}", uf);

    //let (orbits, piece_to_orbit) = build_orbits(&pieces, &mut uf);

    // //println!("{:#?}", orbits);
    // //println!("{:#?}", piece_to_orbit);

    // let moves = build_moves(drone.trim(), &pieces, &piece_to_orbit);

    // println!("{:#?}", moves);

    // let pzl = build_puzzle(drone.trim(), &pieces);

    // //println!("{:#?}", pzl);


    // let pzl = build_puzzle(drone.trim(), &pieces).unwrap();
    // let state = State::solved(&pzl);

    // println!("{:#?}", state);


    let pieces = initialise_pieces(drone.trim()).unwrap();

    let pzl = build_puzzle(drone.trim(), &pieces).unwrap();

    let mut state = State::solved(&pzl);

    println!("=== SOLVED ===");
    println!("{}", visualise(&state, &pzl, PrintType::UF));
    //println!("{:?}", &state.orientation);


    println!("{}", visualise_generic(&state, &pzl));


    //print moves test
    // for mv in &pzl.moves {
    //   println!("{}", mv.name);
    // }

    //println!("{:#?}", pzl.moves);


    let groups: Vec<_> = build_coordinate_groups(&pzl.orbits)
    .into_iter()
    .filter(|g| g.piece_indices.len() == pzl.orbits[g.orbit_id as usize].size as usize)
    .collect();

    for (i, group) in groups.iter().enumerate() {
        let path = format!("cache/orbit{}_group{}_perm.bin", group.orbit_id, i);
        let table = get_or_build_table(group, &pzl, &path);
        println!("group {} ({:?}): table has {} entries", i, group.piece_indices, table.len());
    }


}

