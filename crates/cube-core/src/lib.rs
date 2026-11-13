//! Exact integer cube rules. Rendering, clocks, randomness sources and I/O live outside.
use serde::Serialize;

pub mod journal;

pub const FACES: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];
pub const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
pub const MAX_MOVES: usize = 128;

/// A sticker direction in the fixed world frame: +X right, +Y up, +Z front.
pub const NORMALS: [[i8; 3]; 6] = [
    [0, 1, 0],
    [1, 0, 0],
    [0, 0, 1],
    [0, -1, 0],
    [-1, 0, 0],
    [0, 0, -1],
];

/// Facelets are row-major, each face viewed directly from outside, URFDLB order.
pub fn facelet_position(face: usize, cell: usize) -> [i8; 3] {
    let row = (cell / 3) as i8;
    let col = (cell % 3) as i8;
    match face {
        0 => [col - 1, 1, row - 1],
        1 => [1, 1 - row, 1 - col],
        2 => [col - 1, 1 - row, 1],
        3 => [col - 1, -1, 1 - row],
        4 => [-1, 1 - row, col - 1],
        5 => [1 - col, 1 - row, -1],
        _ => panic!("invalid face"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Sticker {
    normal: [i8; 3],
    color: char,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Piece {
    position: [i8; 3],
    stickers: Vec<Sticker>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cube {
    // Eight corners and twelve edges; orientation is encoded by sticker normals.
    corners: Vec<Piece>,
    edges: Vec<Piece>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    face: usize,
    turns: u8,
}

impl Move {
    /// Outward face normal and signed clockwise quarter turns (inverse is -1).
    pub fn rotation(self) -> ([i8; 3], i8) {
        (
            NORMALS[self.face],
            if self.turns == 3 {
                -1
            } else {
                self.turns as i8
            },
        )
    }

    pub fn notation(self) -> String {
        let suffix = match self.turns {
            1 => "",
            2 => "2",
            _ => "'",
        };
        format!("{}{suffix}", FACES[self.face])
    }

    pub fn inverse(self) -> Self {
        Self {
            face: self.face,
            turns: 4 - self.turns,
        }
    }
}

pub fn parse_moves(text: &str) -> Result<Vec<Move>, String> {
    let tokens: Vec<_> = text.split_whitespace().collect();
    if tokens.is_empty() || tokens.len() > MAX_MOVES {
        Err(format!(
            "Supply 1 to {MAX_MOVES} moves, separated by spaces."
        ))
    } else {
        tokens
            .into_iter()
            .map(|token| {
                let mut chars = token.chars();
                let face = chars
                    .next()
                    .and_then(|face| FACES.iter().position(|candidate| *candidate == face));
                let turns = match (chars.next(), chars.next()) {
                    (None, None) => Some(1),
                    (Some('2'), None) => Some(2),
                    (Some('\''), None) => Some(3),
                    _ => None,
                };
                match (face, turns) {
                    (Some(face), Some(turns)) => Ok(Move { face, turns }),
                    _ => Err(format!(
                        "Invalid move {token:?}. Use U D L R F B, optionally followed by ' or 2."
                    )),
                }
            })
            .collect()
    }
}

fn rotate([x, y, z]: [i8; 3], axis: usize, direction: i8) -> [i8; 3] {
    match axis {
        0 => [x, -direction * z, direction * y],
        1 => [direction * z, y, -direction * x],
        _ => [-direction * y, direction * x, z],
    }
}

impl Default for Cube {
    fn default() -> Self {
        let mut corners = Vec::new();
        let mut edges = Vec::new();
        for x in -1_i8..=1 {
            for y in -1_i8..=1 {
                for z in -1_i8..=1 {
                    let position = [x, y, z];
                    let stickers: Vec<_> = NORMALS
                        .iter()
                        .enumerate()
                        .filter_map(|(face, normal)| {
                            let axis = normal
                                .iter()
                                .position(|value| *value != 0)
                                .expect("face axis");
                            (position[axis] == normal[axis]).then_some(Sticker {
                                normal: *normal,
                                color: FACES[face],
                            })
                        })
                        .collect();
                    match stickers.len() {
                        3 => corners.push(Piece { position, stickers }),
                        2 => edges.push(Piece { position, stickers }),
                        _ => {}
                    }
                }
            }
        }
        Self { corners, edges }
    }
}

// Cyclic face order distinguishes a physical corner from its mirror image.
const CORNER_FACES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [0, 2, 4],
    [0, 4, 5],
    [0, 5, 1],
    [3, 2, 1],
    [3, 4, 2],
    [3, 5, 4],
    [3, 1, 5],
];
const EDGE_FACES: [[usize; 2]; 12] = [
    [0, 1],
    [0, 2],
    [0, 4],
    [0, 5],
    [3, 1],
    [3, 2],
    [3, 4],
    [3, 5],
    [2, 1],
    [2, 4],
    [5, 4],
    [5, 1],
];

struct DecodedPieces {
    pieces: Vec<Piece>,
    permutation: Vec<usize>,
    orientation_sum: usize,
}

fn decode_pieces<const N: usize>(
    values: &[char],
    layout: &[[usize; N]],
) -> Result<DecodedPieces, String> {
    let mut result = DecodedPieces {
        pieces: Vec::new(),
        permutation: Vec::new(),
        orientation_sum: 0,
    };
    for faces in layout {
        let mut position = [0; 3];
        for face in faces {
            for (axis, value) in position.iter_mut().enumerate() {
                *value += NORMALS[*face][axis];
            }
        }
        let stickers: Vec<_> = faces
            .iter()
            .map(|face| {
                let cell = (0..9)
                    .find(|cell| facelet_position(*face, *cell) == position)
                    .expect("piece facelet coordinate");
                Sticker {
                    normal: NORMALS[*face],
                    color: values[face * 9 + cell],
                }
            })
            .collect();
        let identity = layout
            .iter()
            .enumerate()
            .find_map(|(piece, colors)| {
                (0..N)
                    .find(|orientation| {
                        (0..N).all(|index| {
                            stickers[(index + orientation) % N].color == FACES[colors[index]]
                        })
                    })
                    .map(|orientation| (piece, orientation))
            })
            .ok_or("The link contains an impossible or mirrored piece.")?;
        if result.permutation.contains(&identity.0) {
            return Err("The link repeats a cube piece.".into());
        }
        result.permutation.push(identity.0);
        result.orientation_sum += identity.1;
        result.pieces.push(Piece { position, stickers });
    }
    Ok(result)
}

fn parity(permutation: &[usize]) -> usize {
    permutation
        .iter()
        .enumerate()
        .map(|(i, value)| {
            permutation[i + 1..]
                .iter()
                .filter(|other| *other < value)
                .count()
        })
        .sum::<usize>()
        % 2
}

impl Cube {
    /// Parse a fixed-center URFDLB position and prove it is reachable by legal turns.
    pub fn from_facelets(text: &str) -> Result<Self, String> {
        let values: Vec<_> = text.chars().collect();
        if values.len() != 54
            || FACES
                .iter()
                .any(|face| values.iter().filter(|value| *value == face).count() != 9)
        {
            return Err(
                "A cube link must contain exactly nine stickers of each URFDLB color.".into(),
            );
        }
        if FACES
            .iter()
            .enumerate()
            .any(|(face, color)| values[face * 9 + 4] != *color)
        {
            return Err("The link's center colors must use the fixed URFDLB frame.".into());
        }
        let corners = decode_pieces(&values, &CORNER_FACES)?;
        let edges = decode_pieces(&values, &EDGE_FACES)?;
        if corners.orientation_sum % 3 != 0
            || edges.orientation_sum % 2 != 0
            || parity(&corners.permutation) != parity(&edges.permutation)
        {
            Err("The link has an impossible twist, flip, or piece permutation.".into())
        } else {
            Ok(Self {
                corners: corners.pieces,
                edges: edges.pieces,
            })
        }
    }
    pub fn apply(&mut self, movement: Move) {
        let normal = NORMALS[movement.face];
        let axis = normal
            .iter()
            .position(|value| *value != 0)
            .expect("face axis");
        // Clockwise from outside is a negative right-handed rotation about the outward normal.
        for _ in 0..movement.turns {
            for piece in self.corners.iter_mut().chain(self.edges.iter_mut()) {
                if piece.position[axis] == normal[axis] {
                    piece.position = rotate(piece.position, axis, -normal[axis]);
                    for sticker in &mut piece.stickers {
                        sticker.normal = rotate(sticker.normal, axis, -normal[axis]);
                    }
                }
            }
        }
    }

    pub fn facelets(&self) -> String {
        let mut output = String::with_capacity(54);
        for (face, normal) in NORMALS.iter().enumerate() {
            for cell in 0..9 {
                let position = facelet_position(face, cell);
                let color = self
                    .corners
                    .iter()
                    .chain(self.edges.iter())
                    .find(|piece| piece.position == position)
                    .and_then(|piece| {
                        piece
                            .stickers
                            .iter()
                            .find(|sticker| sticker.normal == *normal)
                    })
                    .map(|sticker| sticker.color);
                let color = if cell == 4 {
                    FACES[face]
                } else {
                    color.expect("every non-center facelet has a sticker")
                };
                output.push(color);
            }
        }
        output
    }

    pub fn is_solved(&self) -> bool {
        self.facelets() == SOLVED
    }
}

/// Deterministic legal-move scramble, not uniform random-state sampling.
pub fn scramble(seed: u32, length: usize) -> Result<Vec<Move>, String> {
    if !(1..=100).contains(&length) {
        Err("Shuffle length must be between 1 and 100.".to_string())
    } else {
        let mut state = if seed == 0 { 0x9e37_79b9 } else { seed };
        let mut output = Vec::with_capacity(length);
        while output.len() < length {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let face = (state % 6) as usize;
            if output
                .last()
                .is_none_or(|previous: &Move| previous.face != face)
            {
                output.push(Move {
                    face,
                    turns: ((state / 6) % 3 + 1) as u8,
                });
            }
        }
        Ok(output)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub facelets: String,
    pub face_order: &'static str,
    pub revision: u32,
    pub move_count: u32,
    pub solved: bool,
}

#[derive(Default)]
pub struct Session {
    cube: Cube,
    revision: u32,
    move_count: u32,
}

impl Session {
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            facelets: self.cube.facelets(),
            face_order: "URFDLB",
            revision: self.revision,
            move_count: self.move_count,
            solved: self.cube.is_solved(),
        }
    }

    pub fn apply_moves(&mut self, text: &str, expected_revision: u32) -> Result<Snapshot, String> {
        let moves = parse_moves(text)?;
        if expected_revision != self.revision {
            Err(format!(
                "Stale revision: expected {}, received {expected_revision}. Read the cube again.",
                self.revision
            ))
        } else {
            let revision = self
                .revision
                .checked_add(1)
                .ok_or("Session revision exhausted.")?;
            let count = self
                .move_count
                .checked_add(moves.len() as u32)
                .ok_or("Move count exhausted.")?;
            for movement in moves {
                self.cube.apply(movement);
            }
            self.revision = revision;
            self.move_count = count;
            Ok(self.snapshot())
        }
    }

    pub fn load(&mut self, facelets: &str, expected_revision: u32) -> Result<Snapshot, String> {
        if expected_revision != self.revision {
            return Err("Stale revision. Read the cube again.".into());
        }
        let cube = Cube::from_facelets(facelets)?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or("Session revision exhausted.")?;
        self.cube = cube;
        self.revision = revision;
        self.move_count = 0;
        Ok(self.snapshot())
    }

    pub fn shuffle(&mut self, seed: u32, length: usize) -> Result<Snapshot, String> {
        let moves = scramble(seed, length)?;
        let revision = self
            .revision
            .checked_add(1)
            .ok_or("Session revision exhausted.")?;
        self.cube = Cube::default();
        for movement in moves {
            self.cube.apply(movement);
        }
        self.revision = revision;
        self.move_count = 0;
        Ok(self.snapshot())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_positions_round_trip_reachable_cubes() {
        for seed in 0..128 {
            let mut cube = Cube::default();
            for movement in scramble(seed, 100).unwrap() {
                cube.apply(movement);
            }
            let imported = Cube::from_facelets(&cube.facelets()).unwrap();
            assert_eq!(imported.facelets(), cube.facelets());
            let mut session = Session::default();
            let loaded = session.load(&cube.facelets(), 0).unwrap();
            assert_eq!(loaded.move_count, 0);
            assert_eq!(loaded.revision, 1);
        }
    }

    #[test]
    fn impossible_shared_positions_are_rejected() {
        let mut impossible = Vec::new();
        impossible.push(SOLVED[..53].to_string());
        impossible.push(SOLVED.replacen('U', "R", 1));
        for indices in [
            vec![5, 10],
            vec![8, 9, 20],
            vec![9, 20],
            vec![10, 19],
            vec![4, 13],
        ] {
            let mut chars: Vec<_> = SOLVED.chars().collect();
            let original = chars.clone();
            for (i, target) in indices.iter().enumerate() {
                chars[*target] = original[indices[(i + 1) % indices.len()]];
            }
            impossible.push(chars.into_iter().collect());
        }
        for position in impossible {
            assert!(
                Cube::from_facelets(&position).is_err(),
                "accepted {position}"
            );
        }
        let mut session = Session::default();
        session.apply_moves("R", 0).unwrap();
        let before = session.snapshot();
        assert!(session.load("bad", 1).is_err());
        assert!(session.load(SOLVED, 0).is_err());
        assert_eq!(session.snapshot().facelets, before.facelets);
    }

    #[test]
    fn solved_and_facelet_coordinates() {
        assert_eq!(Cube::default().facelets(), SOLVED);
        for (face, normal) in NORMALS.iter().enumerate() {
            assert_eq!(facelet_position(face, 4), *normal);
        }
    }

    #[test]
    fn moves_have_inverses_and_order_four() {
        for face in FACES {
            let movement = parse_moves(&face.to_string()).unwrap()[0];
            let mut cube = Cube::default();
            cube.apply(movement);
            assert!(!cube.is_solved());
            cube.apply(movement.inverse());
            assert_eq!(cube, Cube::default());
            for _ in 0..4 {
                cube.apply(movement);
            }
            assert_eq!(cube, Cube::default());
        }
    }

    #[test]
    fn sequences_keep_all_stickers_and_reverse_exactly() {
        for seed in 0..64 {
            let moves = scramble(seed, 100).unwrap();
            let mut cube = Cube::default();
            for movement in &moves {
                cube.apply(*movement);
            }
            let facelets = cube.facelets();
            for color in FACES {
                assert_eq!(facelets.chars().filter(|value| *value == color).count(), 9);
            }
            for movement in moves.iter().rev() {
                cube.apply(movement.inverse());
            }
            assert_eq!(cube, Cube::default());
        }
    }

    #[test]
    fn invalid_batches_and_stale_commands_are_atomic() {
        let mut session = Session::default();
        for invalid in ["", "R wat", "r", "R3", "U2'", "🦀"] {
            assert!(session.apply_moves(invalid, 0).is_err());
            assert_eq!(session.snapshot().facelets, SOLVED);
            assert_eq!(session.snapshot().revision, 0);
        }
        assert!(session.apply_moves("R", 1).is_err());
        assert!(session.apply_moves("R U", 0).is_ok());
        let before = session.snapshot();
        assert!(session.apply_moves("F", 0).is_err());
        assert_eq!(session.snapshot().facelets, before.facelets);
        assert_eq!(session.snapshot().move_count, 2);
        assert!(session.apply_moves("U' R'", 1).unwrap().solved);
    }

    #[test]
    fn seeded_shuffle_is_reproducible_and_bounded() {
        assert_eq!(scramble(42, 20).unwrap(), scramble(42, 20).unwrap());
        assert!(scramble(1, 0).is_err());
        assert!(scramble(1, 101).is_err());
        let mut session = Session::default();
        let state = session.shuffle(42, 20).unwrap();
        assert!(!state.solved);
        assert_eq!(state.revision, 1);
        assert_eq!(state.move_count, 0);
    }
}

#[cfg(test)]
mod reference_tests {
    use super::*;

    // Generated from cubejs 1.3.2's independent cubie move tables (MIT).
    #[test]
    fn singmaster_moves_match_independent_reference() {
        let fixtures = [
            (
                "U",
                "UUUUUUUUUBBBRRRRRRRRRFFFFFFDDDDDDDDDFFFLLLLLLLLLBBBBBB",
            ),
            (
                "U'",
                "UUUUUUUUUFFFRRRRRRLLLFFFFFFDDDDDDDDDBBBLLLLLLRRRBBBBBB",
            ),
            (
                "U2",
                "UUUUUUUUULLLRRRRRRBBBFFFFFFDDDDDDDDDRRRLLLLLLFFFBBBBBB",
            ),
            (
                "R",
                "UUFUUFUUFRRRRRRRRRFFDFFDFFDDDBDDBDDBLLLLLLLLLUBBUBBUBB",
            ),
            (
                "R'",
                "UUBUUBUUBRRRRRRRRRFFUFFUFFUDDFDDFDDFLLLLLLLLLDBBDBBDBB",
            ),
            (
                "R2",
                "UUDUUDUUDRRRRRRRRRFFBFFBFFBDDUDDUDDULLLLLLLLLFBBFBBFBB",
            ),
            (
                "F",
                "UUUUUULLLURRURRURRFFFFFFFFFRRRDDDDDDLLDLLDLLDBBBBBBBBB",
            ),
            (
                "F'",
                "UUUUUURRRDRRDRRDRRFFFFFFFFFLLLDDDDDDLLULLULLUBBBBBBBBB",
            ),
            (
                "F2",
                "UUUUUUDDDLRRLRRLRRFFFFFFFFFUUUDDDDDDLLRLLRLLRBBBBBBBBB",
            ),
            (
                "D",
                "UUUUUUUUURRRRRRFFFFFFFFFLLLDDDDDDDDDLLLLLLBBBBBBBBBRRR",
            ),
            (
                "D'",
                "UUUUUUUUURRRRRRBBBFFFFFFRRRDDDDDDDDDLLLLLLFFFBBBBBBLLL",
            ),
            (
                "D2",
                "UUUUUUUUURRRRRRLLLFFFFFFBBBDDDDDDDDDLLLLLLRRRBBBBBBFFF",
            ),
            (
                "L",
                "BUUBUUBUURRRRRRRRRUFFUFFUFFFDDFDDFDDLLLLLLLLLBBDBBDBBD",
            ),
            (
                "L'",
                "FUUFUUFUURRRRRRRRRDFFDFFDFFBDDBDDBDDLLLLLLLLLBBUBBUBBU",
            ),
            (
                "L2",
                "DUUDUUDUURRRRRRRRRBFFBFFBFFUDDUDDUDDLLLLLLLLLBBFBBFBBF",
            ),
            (
                "B",
                "RRRUUUUUURRDRRDRRDFFFFFFFFFDDDDDDLLLULLULLULLBBBBBBBBB",
            ),
            (
                "B'",
                "LLLUUUUUURRURRURRUFFFFFFFFFDDDDDDRRRDLLDLLDLLBBBBBBBBB",
            ),
            (
                "B2",
                "DDDUUUUUURRLRRLRRLFFFFFFFFFDDDDDDUUURLLRLLRLLBBBBBBBBB",
            ),
            (
                "R U R' U'",
                "UULUUFUUFRRUBRRURRFFDFFUFFFDDRDDDDDDBLLLLLLLLBRRBBBBBB",
            ),
            (
                "F R U R' U' F'",
                "UULUUFUBLUUURRRRRRRUFFFFFFFDDDDDDDDDBLFLLLLLLBRRBBBBBB",
            ),
            (
                "R2 U B' L D2 F R' U2",
                "LFULUBDLFUDBBRLBDDRBRFFUDBDLRLUDRUDRUUBLLRRRFLUFFBDFFB",
            ),
        ];
        for (algorithm, expected) in fixtures {
            let mut cube = Cube::default();
            for movement in parse_moves(algorithm).unwrap() {
                cube.apply(movement);
            }
            assert_eq!(cube.facelets(), expected, "{algorithm}");
        }
    }
}
