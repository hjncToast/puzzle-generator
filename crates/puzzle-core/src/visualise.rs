use crate::parser::{Puzzle};
use crate::state::{State};

fn face_colour(face: char) -> &'static str {
    match face {
        'U' => "⬜", 'D' => "🟨",
        'F' => "🟩", 'B' => "🟦",
        'R' => "🟥", 'L' => "🟧",
        _   => "⬛",
    }
}

pub enum PrintType {
    FullCube,
    LL,
    UF,
}



pub fn sticker(piece: &str, face: char, state: &State, puzzle: &Puzzle) -> &'static str {
  
    let Some(&(orbit_id, local_id)) = puzzle.name_to_local.get(piece) else {
        return "⬛";
    };
    let o = orbit_id as usize;

    // local_id is the home slot of this piece
    // find which piece is currently sitting in that slot
    let piece_there = state.permutation[o][local_id as usize] as usize;
    let name = &puzzle.piece_names[o][piece_there];

    // which sticker index does this face letter correspond to in the home piece
    let caps: Vec<char> = piece.chars().filter(|c| c.is_uppercase()).collect();
    let sticker_idx = caps.iter().position(|&c| c == face).unwrap();

    // apply orientation to find what's actually showing
    let orient = state.orientation[o][local_id as usize] as usize;
    let caps_there: Vec<char> = name.chars().filter(|c| c.is_uppercase()).collect();
    let letter = caps_there[(sticker_idx + orient) % caps_there.len()];

    face_colour(letter)
}


pub fn visualise(state: &State, puzzle: &Puzzle, print_type: PrintType) -> String {
    match print_type {
        PrintType::FullCube => print_cube(state, puzzle),
        PrintType::LL       => print_ll(state, puzzle),
        PrintType::UF       => print_uf(state, puzzle),
    }
}






pub fn print_ll(state: &State, puzzle: &Puzzle) -> String {
    let s = |piece: &str, face: char| sticker(piece, face, state, puzzle);
    let mut lines: Vec<String> = Vec::new();

    lines.push(format!("⬛{}{}{}⬛",       s("ULB",'B'), s("UB",'B'),  s("UBR",'B')));
    lines.push(format!("{}{}{}{}{}",       s("ULB",'L'), s("ULB",'U'), s("UB",'U'),  s("UBR",'U'), s("UBR",'R')));
    lines.push(format!("{}{}{}{}{}",       s("UL",'L'),  s("UL",'U'),  s("U",'U'),   s("UR",'U'),  s("UR",'R')));
    lines.push(format!("{}{}{}{}{}",       s("UFL",'L'), s("UFL",'U'), s("UF",'U'),  s("URF",'U'), s("URF",'R')));
    lines.push(format!("⬛{}{}{}⬛",       s("UFL",'F'), s("UF",'F'),  s("URF",'F')));

    lines.join("\n")
}

pub fn print_uf(state: &State, puzzle: &Puzzle) -> String {
    let s = |piece: &str, face: char| sticker(piece, face, state, puzzle);
    let mut lines: Vec<String> = Vec::new();

    // U face
    lines.push(format!("{}{}{}{}{}", s("ULB",'L'), s("ULB",'U'), s("UB",'U'),  s("UBR",'U'), s("UBR",'R')));
    lines.push(format!("{}{}{}{}{}", s("UL",'L'),  s("UL",'U'),  s("U",'U'),   s("UR",'U'),  s("UR",'R')));
    lines.push(format!("{}{}{}{}{}", s("UFL",'L'), s("UFL",'U'), s("UF",'U'),  s("URF",'U'), s("URF",'R')));
    // F face
    lines.push(format!("{}{}{}{}{}", s("UFL",'L'), s("UFL",'F'), s("UF",'F'),  s("URF",'F'), s("URF",'R')));
    lines.push(format!("{}{}{}{}{}", s("FL",'L'),  s("FL",'F'),  s("F",'F'),   s("FR",'F'),  s("FR",'R')));
    lines.push(format!("{}{}{}{}{}", s("DLF",'L'), s("DLF",'F'), s("DF",'F'),  s("DFR",'F'), s("DFR",'R')));

    lines.join("\n")
}

pub fn print_cube(state: &State, puzzle: &Puzzle) -> String {
    let s = |piece: &str, face: char| sticker(piece, face, state, puzzle);
    let mut lines: Vec<String> = Vec::new();

    // U face
    lines.push(format!("⬛⬛⬛ {}{}{}", s("ULB",'U'), s("UB",'U'),  s("UBR",'U')));
    lines.push(format!("⬛⬛⬛ {}{}{}", s("UL",'U'),  s("U",'U'),   s("UR",'U')));
    lines.push(format!("⬛⬛⬛ {}{}{}", s("UFL",'U'), s("UF",'U'),  s("URF",'U')));
    lines.push(String::new());

    // Middle layer
    lines.push(format!("{}{}{} {}{}{} {}{}{} {}{}{}",
        s("ULB",'L'), s("UL",'L'),  s("UFL",'L'),
        s("UFL",'F'), s("UF",'F'),  s("URF",'F'),
        s("URF",'R'), s("UR",'R'),  s("UBR",'R'),
        s("UBR",'B'), s("UB",'B'),  s("ULB",'B'),
    ));
    lines.push(format!("{}{}{} {}{}{} {}{}{} {}{}{}",
        s("BL",'L'),  s("L",'L'),   s("FL",'L'),
        s("FL",'F'),  s("F",'F'),   s("FR",'F'),
        s("FR",'R'),  s("R",'R'),   s("BR",'R'),
        s("BR",'B'),  s("B",'B'),   s("BL",'B'),
    ));
    lines.push(format!("{}{}{} {}{}{} {}{}{} {}{}{}",
        s("DBL",'L'), s("DL",'L'),  s("DLF",'L'),
        s("DLF",'F'), s("DF",'F'),  s("DFR",'F'),
        s("DFR",'R'), s("DR",'R'),  s("DRB",'R'),
        s("DRB",'B'), s("DB",'B'),  s("DBL",'B'),
    ));
    lines.push(String::new());

    // D face
    lines.push(format!("⬛⬛⬛ {}{}{}", s("DLF",'D'), s("DF",'D'),  s("DFR",'D')));
    lines.push(format!("⬛⬛⬛ {}{}{}", s("DL",'D'),  s("D",'D'),   s("DR",'D')));
    lines.push(format!("⬛⬛⬛ {}{}{}", s("DBL",'D'), s("DB",'D'),  s("DRB",'D')));

    lines.join("\n")
}




pub fn visualise_generic(state: &State, puzzle: &Puzzle) -> String {
    let mut out = String::new();

    for orbit in &puzzle.orbits {
        let o = orbit.id as usize;
        let names = &puzzle.piece_names[o];

        // header row — slot names
        out.push_str(&format!("orbit id: {}\n", orbit.id));
        out.push_str("slot:      ");
        for name in names {
            out.push_str(&format!("{:<6}", name));
        }
        out.push('\n');

        // one row per orientation sticker index
        for sticker_idx in 0..orbit.modulus as usize {
            // get the face letter for sticker_idx from each slot's home piece name
            // to use as row label e.g. "U:", "F:", "R:"
            out.push_str(&format!("orient {}:  ", sticker_idx));

            for slot in 0..orbit.size as usize {
                // which piece is in this slot
                let piece_there = state.permutation[o][slot] as usize;
                let name = &puzzle.piece_names[o][piece_there];
                let orient = state.orientation[o][slot] as usize;

                let caps: Vec<char> = name.chars().filter(|c| c.is_uppercase()).collect();
                let letter = caps[(sticker_idx + orient) % caps.len()];
                out.push_str(&format!("{:<5}", face_colour(letter)));
            }
            out.push('\n');
        }

        out.push('\n');
    }

    out
}