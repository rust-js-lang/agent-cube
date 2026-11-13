use cube_core::{Cube, Move, scramble};

fn main() {
    for seed in 0..64 {
        let moves = scramble(seed, 100).expect("valid shuffle");
        let algorithm = moves
            .iter()
            .copied()
            .map(Move::notation)
            .collect::<Vec<_>>()
            .join(" ");
        let mut cube = Cube::default();
        for movement in moves {
            cube.apply(movement);
        }
        println!("{seed}\t{algorithm}\t{}", cube.facelets());
    }
}
