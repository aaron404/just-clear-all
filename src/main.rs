use std::env;

use jca;

fn main() {

    let mut board = jca::Board::new();

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let level = args.remove(0).parse::<u32>().expect("Not a number");

    for y in 0 .. jca::BOARD_SIZE {
        for x in 0 .. jca::BOARD_SIZE {
            board.board[x][y] = args[y * jca::BOARD_SIZE + x].parse::<u8>().expect("Not a number");
            if board.board[x][y] == 3 {
                // board.board[x][y] = 0;
            }
        }
    }

    // println!("board: {}", board);
    unsafe {
        board.collapse_simd2();
    }


    jca::set_level(level);
    // let mut b = jca::Board::from_rows([
    //     0x_3_2_3_3_2_2_4_3,
    //     0x_2_4_3_3_3_3_3_8,
    //     0x_2_2_3_3_3_2_2_3,
    //     0x_2_3_2_2_3_4_2_2,
    //     0x_2_4_2_3_3_3_3_2,
    //     0x_2_2_2_2_2_2_4_2,
    //     0x_3_4_3_3_2_3_2_3,
    //     0x_2_4_2_2_2_4_3_3,
    // ]);

    board.solve();

    // solve(board.clone());
}
