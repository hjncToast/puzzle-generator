use std::collections::HashMap;

#[derive(Debug)]
pub struct PieceInfo {
    pub index: u16,
    pub modulus: u8,
}

// & means it is a reference, not a copy. 
//'drone means the struct must live as long as the thing its pointing to
fn assign<'a>(
    map: &mut HashMap<&'a str, PieceInfo>,
    next_index: &mut u16,
    name: &'a str,
    modulus: u8,
) {
    //if the key exists, return the value, if not, .or_insert_with runs the anomymous function to create the new value, insert it and return a reference
    map.entry(name).or_insert_with(||{
        let info = PieceInfo{
            //assign index the value of next_index. * is needed because next_index: &mut u16 is a pointer, * reads the value that next_index points to.
            index: *next_index,
            modulus,
        };
    //this means, follow the pointer and increment that value, rust is interesting aaaa
    *next_index += 1;
    info
  });
}


pub fn initialise_pieces(src: &str) -> Result<HashMap<&str, PieceInfo>, String> {
    let mut map: HashMap<&str, PieceInfo> = HashMap::new();
    let mut next_index: u16 = 0;

    // First pass: override modulus if you don't want it inferred from capital letter count. Not used often in definition files but useful to have
    for line in src.lines() {
        if let Some((left, right)) = line.split_once(':') {
            if left.trim().chars().all(|c| c.is_ascii_digit()) {
                let modulus: u8 = left.trim().parse().unwrap();
                for name in right.split_whitespace() {
                    if map.contains_key(name) {
                        return Err(format!("piece '{name}' has duplicate override"));
                    }
                    assign(&mut map, &mut next_index, name, modulus);
                }
            }
        }
    }

    // Second pass: move definition lines
    for line in src.lines() {
        if let Some((left, right)) = line.split_once(':') {
            if !left.trim().chars().all(|c| c.is_ascii_digit()) {
                for group in right.split('(').skip(1).filter_map(|s| s.split(')').next()) {
                    for name in group.split_whitespace() {
                        let name = strip_orientation(name);
                        let modulus = infer_modulus(name);
                        assign(&mut map, &mut next_index, name, modulus);
                    }
                }
            }
        }
    }

    Ok(map)
}


// counts Uppercase letters to use for modulus count
fn infer_modulus(piece: &str) -> u8 {
    let name = piece.split('[').next().unwrap_or(piece);
    match name.chars().filter(|c| c.is_uppercase()).count() {
        0 | 1 => 1,
        n => n as u8,
    }
}

//strips the notation for orientation off the name
fn strip_orientation(token: &str) -> &str {
    // only look for +/- after the closing ']' if brackets exist
    let search_from = token.rfind(']').map(|i| i + 1).unwrap_or(0);
    
    if let Some(i) = token[search_from..].find(['+', '-']) {
        &token[..search_from + i]
    } else {
        token
    }
}




#[derive(Debug)]
pub struct Orbit {
    pub id: u8,           // unique ID, assigned in order of first orbit discovered. 3x3 corners would all have the same ID for example, but skewb corners different
    pub size: u8,         // number of pieces in this orbit
    pub modulus: u8,      //amount of orientations per piece
    pub permutation: Vec<u8>,    //location of each piece. Index is location, value is the piece ID
    pub orientation: Vec<u8>,     //each individual piece's orientation. Index is piece, value is orientation
}

#[derive(Debug)]
pub struct Cycle {
    pub orbit_id: u8,
    pub slots_to_permute: Vec<u8>,    // [1,2,3,4] — which slots in the orbit are involved
    pub orientation_offset: Vec<i8>,  // [1,1,1,1] — orientation change per slot
}

#[derive(Debug)]
pub struct Move {
    pub name: String,
    pub cycles: Vec<Cycle>,
}

#[derive(Debug)]
pub struct Puzzle {
    pub orbits: Vec<Orbit>,
    pub moves: Vec<Move>,
    pub piece_names: Vec<Vec<String>>,  // [orbit_id][local_id] -> name
    pub name_to_local: HashMap<String, (u8, u8)>,  // "UFR" -> (orbit_id, local_id)
}


//find the orbits of pieces based on the cycles
// parent is an array where the index is the piece and the value is who it points to
// index:  0  1  2  3  4  5  6  7
// value:  0  1  2  3  4  5  6  7 initially everyone points to themselves

//after finding unions, it might look like this
// index:  0  1  2  3  4  5  6  7
// value:  7  7  7  7  7  7  7  7   // all corners point to 7

#[derive(Debug)]
pub struct UnionFind {
    parent: Vec<usize>,
}

//self is like .this in javascript, ooooh
impl UnionFind {
    fn new(n: usize) -> Self {
        Self { parent: (0..n).collect() }
        //creates a range of numbers from 0-pieces.length. every index of the array points to itself like this[0,1,2,3]
    }

    //follow a chain of pointers until you hit a piece that points to itself, it's the root so return it
    fn find(&mut self, x: usize) -> usize {
        // not the root, follow to the next value it points to
        if self.parent[x] != x {
            // path compression to make future look ups 
            // faster. Make the piece point directly to the root instead of all around then end up at the root
            self.parent[x] = self.find(self.parent[x]); 
        }
        self.parent[x]
    }

    //if there's an orbit match, union function will make the root of A point to B.
    fn union(&mut self, a: usize, b: usize) {
        let a = self.find(a);
        let b = self.find(b);
        self.parent[a] = b; // merge the two groups
    }
}

//returns a Unionfind containing parent which is the array where the index's are the pieces and the 
//value is the piece index that it points to
//successfully spits out 3 unique values, and 23 items. (centres, corners, edges) 23 pieces total
pub fn build_union_find(src: &str, pieces: &HashMap<&str, PieceInfo>) -> UnionFind {
    let mut uf = UnionFind::new(pieces.len());
    //for each move line
    for line in src.lines() {
        //split off the right side after :
        let Some((_left, right)) = line.split_once(':') else { continue };
        //get the cycles in that move
        let groups = right.split('(').skip(1).filter_map(|s| s.split(')').next());
        //for each cycle in the group of cycles
        for group in groups {
                let ids: Vec<usize> = group
                    //split each piece into tokens
                    .split_whitespace()
                    //look up each piece's global index
                    .map(|token| pieces[strip_orientation(token)].index as usize)
                    //collect into an array 
                    .collect();

                //rust for(;;) loop. For integers 1 to ids.length - 1, do {}
                for i in 1..ids.len() {
                    uf.union(ids[0], ids[i]);
                }
            }
        }
    uf
}





pub fn build_orbits(pieces: &HashMap<&str, PieceInfo>, uf: &mut UnionFind)
  -> (Vec<Orbit>, Vec<(u8, u8)>)
  {
    // Sort by the numeric index value (0, 1, 2...)
    let mut sorted_pieces: Vec<&PieceInfo> = pieces.values().collect();
    sorted_pieces.sort_by_key(|info| info.index);

    let mut orbits: Vec<Orbit> = Vec::new();

    // index of array is the global piece ID containing a (x,y)
    // x is the orbit id, y is the local id of that piece in the orbit
    let mut piece_to_orbit: Vec<(u8, u8)> = vec![(0, 0); pieces.len()];

    let mut unique_roots: Vec<usize> = Vec::new();

    for piece in sorted_pieces {

        let global_id = piece.index as usize;
        let root = uf.find(global_id);

        let orbit_id: u8; 
        //does what i imagine uniqueroots.contains(root) does
        //check if we've seen the root before
        if let Some(index) = unique_roots.iter().position(|&r| r == root) {
            //if we have, the orbit ID of this piece is the index 
            orbit_id = index as u8;
        }
        else { //add it and grab the index (length) so it goes from 0->n, then create the orbit
            orbit_id = unique_roots.len() as u8;
            unique_roots.push(root);

            orbits.push(Orbit {
                id: orbit_id,
                size: 0, // Starts at 0, updated below
                modulus: piece.modulus,
                orientation: Vec::new(),
                permutation: Vec::new(),
            });
        }
    
        //for the current orbit, 
        let current_orbit = &mut orbits[orbit_id as usize];
        //so we can get local IDs from 0-n
        let local_id = current_orbit.size;

        current_orbit.size += 1;
        current_orbit.permutation.push(local_id);
        current_orbit.orientation.push(0);

        piece_to_orbit[global_id] = (orbit_id, local_id);
    }
    (orbits, piece_to_orbit)
  }




pub fn build_moves(src: &str, pieces: &HashMap<&str, PieceInfo>, piece_to_orbit: &Vec<(u8, u8)>) -> Vec<Move> {
    let mut moves = Vec::new();

    for line in src.lines() {
        let Some((left, right)) = line.split_once(':') else { continue };
        
        // skip override lines
        if left.trim().chars().all(|c| c.is_ascii_digit()) { continue };

        let mut cycles = Vec::new();

        //each cycle in brackets (blah blah1 blah2) in the definition file
        for defcycle in right.split('(').skip(1).filter_map(|s| s.split(')').next()) {
            //each piece in the cycle
            let tokens: Vec<&str> = defcycle.split_whitespace().collect();
            //incase it crashes from empty tokens
            if tokens.is_empty() { continue }
            
            //first piece name without orientation from def file, to get orbit ID for the cycle
            let first_name = strip_orientation(tokens[0]);

            //get global ID from piece[].index, then use that in piece_to_orbit to find orbit_id
            let (orbit_id, _) = piece_to_orbit[pieces[first_name].index as usize];

            let mut slots_to_permute = Vec::new();
            let mut orientation_offset = Vec::new();

            //add each piece to the cycle individually
            for token in &tokens {
                let name = strip_orientation(token);
                let (_, local_index) = piece_to_orbit[pieces[name].index as usize];
                let offset = parse_offset(token);
                slots_to_permute.push(local_index);
                orientation_offset.push(offset);
            }

            //all the cycles that one move does
            cycles.push(Cycle { orbit_id, slots_to_permute, orientation_offset });
        }

        //add move name and cycles
        moves.push(Move { name: left.trim().to_string(), cycles });
    }
    moves
}





pub fn parse_offset(token: &str) -> i8 {
    let search_from = token.rfind(']').map(|i| i + 1).unwrap_or(0);
    let suffix = &token[search_from..];
    if let Some(i) = suffix.find('+') {
        suffix[i+1..].parse().unwrap_or(0)
    } else if let Some(i) = suffix.find('-') {
        -suffix[i+1..].parse::<i8>().unwrap_or(0)
    } else {
        0
    }
}

pub fn inverse(mv: &Move) -> Move {
    Move {
        name: format!("{}'", mv.name),
        cycles: mv.cycles.iter().map(|cycle| {
            Cycle {
                orbit_id: cycle.orbit_id,
                slots_to_permute: cycle.slots_to_permute.iter().rev().cloned().collect(),
                orientation_offset: cycle.orientation_offset.iter().rev().cloned().collect(),
            }
        }).collect(),
    }
}

// pub fn double(mv: &Move) -> Move {
//     Move {
//         name: format!("{}2", mv.name),
//         cycles: mv.cycles.iter().map(|cycle| {
//             let n = cycle.slots_to_permute.len();
//             Cycle {
//                 orbit_id: cycle.orbit_id,
//                 slots_to_permute: (0..n)
//                     .map(|i| cycle.slots_to_permute[(i + 2) % n])
//                     .collect(),
//                 orientation_offset: (0..n)
//                     .map(|i| cycle.orientation_offset[i] + cycle.orientation_offset[(i + 1) % n])
//                     .collect(),
//             }
//         }).collect(),
//     }
// }


pub fn double(mv: &Move) -> Move {
    Move {
        name: format!("{}2", mv.name),
        cycles: mv.cycles.iter().flat_map(|cycle| {
            let n = cycle.slots_to_permute.len();
            let mut visited = vec![false; n];
            let mut subcycles = Vec::new();

            for start in 0..n {
                if visited[start] { continue; }
                let mut new_slots = Vec::new();
                let mut new_offsets = Vec::new();
                let mut i = start;
                loop {
                    visited[i] = true;
                    let n1 = (i + 1) % n;
                    let n2 = (i + 2) % n;
                    new_slots.push(cycle.slots_to_permute[n2]);
                    new_offsets.push(cycle.orientation_offset[n1] + cycle.orientation_offset[n2]);
                    i = n2;
                    if i == start { break; }
                }
                // drop trivial cycles
                if new_slots.len() > 1 || new_offsets[0] != 0 {
                    subcycles.push(Cycle {
                        orbit_id: cycle.orbit_id,
                        slots_to_permute: new_slots,
                        orientation_offset: new_offsets,
                    });
                }
            }
            subcycles
        }).collect(),
    }
}


pub fn build_all_moves(moves: Vec<Move>) -> Vec<Move>
{
  let inverses: Vec<Move> = moves.iter().map(inverse).collect();
  let doubles: Vec<Move> = moves.iter().map(double).collect();
    let mut all_moves = moves;
    all_moves.extend(inverses);
    all_moves.extend(doubles);
    all_moves
}





pub fn build_puzzle(src: &str, pieces: &HashMap<&str, PieceInfo>) -> Result<Puzzle, String> {
    let mut uf = build_union_find(src, pieces);
    let (orbits, piece_to_orbit) = build_orbits(pieces, &mut uf);

    //make the empty array first, for every orbit, make a vec of empty strings with the orbit's length, then compile it into an array of all of them
    let mut piece_names: Vec<Vec<String>> = orbits.iter()
    .map(|o| vec![String::new(); o.size as usize])
    .collect();

    // for each piece name, find its orbit and local id and slot it in
    for (name, info) in pieces {
        let global_id = info.index as usize;
        let (orbit_id, local_id) = piece_to_orbit[global_id];
        piece_names[orbit_id as usize][local_id as usize] = name.to_string();
    }

    let moves = build_moves(src, pieces, &piece_to_orbit);
    let all_moves = build_all_moves(moves);

    let mut name_to_local = HashMap::new();
    for (orbit_id, names) in piece_names.iter().enumerate() {
        for (local_id, name) in names.iter().enumerate() {
            name_to_local.insert(name.clone(), (orbit_id as u8, local_id as u8));
        }
    }

    Ok(Puzzle { orbits, moves: all_moves, piece_names, name_to_local })
}



//check if pieces works