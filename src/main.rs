use std::cmp::min;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::hash::Hash;
use std::time::Instant;


const BOARD_SIZE: usize = 8;
const DELTAS: [(i8, i8); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

fn get_score(num_tiles: u32, tile_value: u32) -> u32 {
    match num_tiles {
        1 => 0,
        _ => {
            let multiplier = min(1 + num_tiles / 5, 3);
            tile_value * 5 * num_tiles * multiplier
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Move {
    // overload add operator
    pos: (usize, usize),
}

#[allow(dead_code)]
impl Move {
    fn is_valid(&self) -> bool {
        self.pos.0 > 0 && self.pos.0 < BOARD_SIZE &&
        self.pos.1 > 0 && self.pos.1 < BOARD_SIZE
    }
}

#[derive(PartialEq, Eq, Hash)]
struct BoardHash {
    board_hash: [u128; 2],
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Board {
    board: [[u8; BOARD_SIZE]; BOARD_SIZE],
    score: u32,
    mve: Move,
}

#[derive(PartialEq, Eq, Hash)]
struct Board2Hash {
    board_hash: [u128; 2],
}

impl Board {
    fn new() -> Board {
        Board { board: [[0; BOARD_SIZE]; BOARD_SIZE], score: 0, mve: Move { pos: (0, 0) } }
        // Board { board: [[0; 8]; 8], moves: Vec::new(), score: 0 }
    }
    
    fn collapse(&mut self) {
        // collapse vertically
        for i in 0..BOARD_SIZE {
            let col = &mut self.board[i];
            let mut left = BOARD_SIZE as i8 - 2;
            let mut right = BOARD_SIZE as i8 - 1;
            while left >= 0 && right > 0 {
                match (col[left as usize], col[right as usize]) {
                    (0, 0) => { left -= 1; },
                    (0, _) => { left -= 1; right -= 1; },
                    (_, 0) => {
                        let tmp = col[right as usize];
                        col[right as usize] = col[left as usize];
                        col[left as usize] = tmp;
                        left -= 1;
                        right -= 1;
                    },
                    _ => { left -= 2; right -= 2; },
                }
            }
        }
        // collapse horizontally
        let mut left = 0;
        let mut right = 1;
        while left < BOARD_SIZE-1 && right < BOARD_SIZE {
            match (u64::from_ne_bytes(self.board[left]), u64::from_ne_bytes(self.board[right])) {
                (0, 0) => { right += 1; },
                (0, _) => {
                    let tmp = self.board[left];
                    self.board[left] = self.board[right];
                    self.board[right] = tmp;
                    left += 1;
                    right += 1;
                },
                (_, 0) => { left += 1; right += 1; },
                _ => { left += 2; right += 2; },
            }
        }
    }

    fn get_valid_moves(&self, valid_moves: &mut [Move]) -> usize {
        // let mut valid_moves = Vec::new();

        let b = self.board.clone(); // TODO: local clone vs use reference
        let mut valid = [[0; 8]; 8];

        for x in 0..BOARD_SIZE-2 {
            for y in 0..BOARD_SIZE {
                if b[x][y] == 0 { continue; }
                if b[x][y] == b[x+1][y] {
                    valid[x][y] = 1;
                    valid[x+1][y] = 1;
                }
            }
        }
        for y in 0..BOARD_SIZE {
            if b[6][y] == 0 { continue; }
            if b[6][y] == b[7][y] {
                valid[6][y] = 1;
                valid[7][y] = 1;
            }
        }

        for y in 0..BOARD_SIZE-2 {
            for x in 0..BOARD_SIZE {
                if b[x][y] == 0 { continue; }
                if b[x][y] == b[x][y+1] {
                    valid[x][y] = 1;
                    valid[x][y+1] = 1;
                }
            } 
        }

        for x in 0..BOARD_SIZE {
            if b[x][6] == 0 { continue; }
            if b[x][6] == b[x][7] {
                valid[x][6] = 1;
                valid[x][7] = 1;
            }
        }

        // cull vertically stacked similar numbers
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE-1 {
                if b[x][y] == b[x][y+1] {
                    valid[x][y+1] = 0;
                }
            }
        }

        let mut move_count = 0;
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if valid[x][y] == 1 {
                    valid_moves[move_count] = Move { pos: (x, y) };
                    move_count += 1;
                }
            }
        }
        move_count
    }

    fn is_cleared(&self) -> bool {
        return self.board[0][BOARD_SIZE-2] == 0 && self.board[1][BOARD_SIZE-1] == 0;
    }

    fn get_tiles_remaining(&self) -> u32 {
        let mut count: u32 = 0;
        for i in 0 .. BOARD_SIZE {
            for j in 0 .. BOARD_SIZE {
                if self.board[j][i] > 0 {
                    count += 1;
                }
            }
        }
        count
    }

    fn get_bonus(&self) -> u32 {
        let n = self.get_tiles_remaining();
        if n == 0 {
            println!("panic! {}", self);
            panic!();
        }
        unsafe {
            if n == 1 {
                return 500 * (LEVEL + 1);
            } else if n < 6 {
                return (250 - 50 * (n - 2)) * (LEVEL + 1);
            }
        }
        0
    }

    fn click(&self, m: Move) -> Board {
        let (x, y) = m.pos;
        let val = self.board[x][y];
        let mut visited = [[false; 8]; 8];
        visited[x][y] = true;
        let mut open_cells = vec![Move {pos: (x, y)}];
        let mut closed_cells: Vec<Move> = Vec::new();
        while open_cells.len() > 0 {
            let (x, y) = open_cells.pop().unwrap().pos;
            closed_cells.push(Move { pos: (x, y) } );
            for (dx, dy) in DELTAS.iter() {
                let (nx, ny) = (dx + x as i8, dy + y as i8);
                if nx >= 0 && nx < BOARD_SIZE as i8 && ny >= 0 && ny < BOARD_SIZE as i8 && !visited[nx as usize][ny as usize] {
                    visited[nx as usize][ny as usize] = true;
                    if self.board[nx as usize][ny as usize] == val {
                        open_cells.push(Move {pos: (nx as usize, ny as usize) } );
                    }
                }
            }
        }
        let score = get_score(closed_cells.len() as u32, val as u32);
        
        let mut new_board = Board {
            board: self.board.clone(),
            mve: m.clone(),
            score: score,
        };

        for m in closed_cells.iter() {
            new_board.board[m.pos.0 as usize][m.pos.1 as usize] = 0;
            new_board.board[x as usize][y as usize] = val + 1;
        }
        new_board.collapse();
        new_board
    }

    fn hash(&self) -> BoardHash {
            BoardHash { board_hash: [
                u128::from((u64::from_ne_bytes(self.board[0]) ^ u64::from_ne_bytes(self.board[0]) << 4) as u128) ^ ((u64::from_ne_bytes(self.board[1]) ^ u64::from_ne_bytes(self.board[2]) << 4) as u128) << 64,
                u128::from((u64::from_ne_bytes(self.board[1]) ^ u64::from_ne_bytes(self.board[2]) << 4) as u128) ^ ((u64::from_ne_bytes(self.board[3]) ^ u64::from_ne_bytes(self.board[4]) << 4) as u128) << 64,
            ]
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        println!();
        for y in 0 .. BOARD_SIZE {
            for x in 0 .. BOARD_SIZE {
                print!("{} ", self.board[x][y]);
            }
            println!();
        }
        Ok(())
    }
}

const STACK_SIZE: usize = 48;

#[derive(Clone, Copy)]
struct Frame {
    board: Board,
    moves: [Move; STACK_SIZE],
    num_moves: usize,
}

#[derive(Clone, Copy)]
struct Stack {
    frames: [Frame; STACK_SIZE],
    frame_count: usize,

}

fn solve(starting_board: Board) {
    let mut hashes: HashMap<BoardHash, u32> = HashMap::new();
    let mut stack = Stack {
        frames: [
            Frame {
                board: Board::new(),
                moves: [Move { pos: (0, 0) }; STACK_SIZE],
                num_moves: 0,
            }; STACK_SIZE
        ],
        frame_count: 0,
    };

    let mut best_score = 0u32;

    stack.frames[0].board = starting_board;
    stack.frames[0].num_moves = starting_board.get_valid_moves(&mut stack.frames[0].moves);
    stack.frame_count = 1;

    let t0 = Instant::now();

    let mut cache_hit_keep_count = 0;
    let mut cache_hit_drop_count = 0;

    while stack.frame_count > 0 {
        // let frame = stack.frames.get(stack.frame_count-1).unwrap(); // TODO: get_unchecked
        // TODO: iterate reverse move order to verify correctness
        match stack.frames[stack.frame_count-1].num_moves {
            0 => {
                // no more moves for current board
                let total = stack.frames[stack.frame_count-1].board.score + stack.frames[stack.frame_count-1].board.get_bonus(); // TODO: combine these scores
                if total > best_score {
                    if stack.frames[stack.frame_count-1].board.is_cleared() {
                        print!("[P] ");
                    }
                    let rate = (cache_hit_keep_count + cache_hit_drop_count) as f32 / (Instant::now() - t0).as_secs_f32() * 0.000001;
                    print!("new best score: {}, k/d: {:.2}, rate: {:.2}M, moves: [", total, (cache_hit_keep_count as f32 / cache_hit_drop_count as f32), rate);
                    for i in 1..stack.frame_count {
                        print!("{:?}, ", stack.frames[i].board.mve.pos);
                    }
                    println!("]");
                    best_score = total;
                }
                // "pop" the stack
                stack.frame_count -= 1;
            },
            _ => {
                // if stack.frame_count < 4 {
                //     for _ in 0..stack.frame_count-1 {
                //         print!("-");
                //     }
                //     println!("progress: {}", stack.frames[stack.frame_count-1].num_moves);
                // }
                let m = stack.frames[stack.frame_count-1].moves.get(stack.frames[stack.frame_count-1].num_moves-1).unwrap(); // TODO: get_unchecked
                stack.frames[stack.frame_count-1].num_moves -= 1;
                stack.frames[stack.frame_count].board = stack.frames[stack.frame_count-1].board.click(*m);
                stack.frames[stack.frame_count].board.score += stack.frames[stack.frame_count-1].board.score;

                let h = stack.frames[stack.frame_count].board.hash();
                match hashes.get(&h) {
                    Some(val) if val > &stack.frames[stack.frame_count-1].board.score => { cache_hit_keep_count += 1; continue; },
                    _ => { hashes.insert(h, stack.frames[stack.frame_count-1].board.score); cache_hit_drop_count += 1;  },
                }

                stack.frames[stack.frame_count].num_moves = stack.frames[stack.frame_count].board.get_valid_moves(&mut stack.frames[stack.frame_count].moves);
                stack.frame_count += 1;
            },
        }
    }

    println!("time: {}", (Instant::now() - t0).as_millis());

}

static mut LEVEL: u32 = 1;

fn main() {

    let mut board = Board::new();

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    unsafe {
        LEVEL = args.remove(0).parse::<u32>().expect("Not a number");
    }

    // println!("{:?}", args);

    for y in 0 .. BOARD_SIZE {
        for x in 0 .. BOARD_SIZE {
            board.board[x][y] = args[y * BOARD_SIZE + x].parse::<u8>().expect("Not a number");
        }
    }

    println!("board: {}", board);

    board.collapse();

    println!("board: {}", board);
    println!("score: {}", board.score);
    println!("score: {}", board.click(Move {pos: (0, 0)}).score);
    println!("score: {}", board.click(Move {pos: (1, 1)}).score);
    println!("=============");

    // print!("Moves:\n[",);
    // for m in board.get_valid_moves().iter() {
    //     print!("({} {}) ", m.pos.0, m.pos.1);
    // }
    // println!("]");

    // let mut threads = vec![];

    // let mut primed_boards: Vec<Board> = Vec::new();
    // let mut primed_boards2: Vec<Board> = Vec::new();

    // primed_boards.push(board);
    // let mut priming_hashset: HashSet<BoardHash> = HashSet::new();

    // for i in 0..4 {
    //     println!("i: {}, num_boards: {}", i, primed_boards.len());
    //     for b in &primed_boards {
    //         for x in 0..BOARD_SIZE {
    //             for y in 0..BOARD_SIZE {
    //                 let (new_board, score) = b.click(Move { pos: (x, y) });
    //                 let hash = new_board.hash();
    //                 if !priming_hashset.contains(&hash) {
    //                      priming_hashset.insert(hash);
    //                      primed_boards2.push(new_board);
    //                 }
    //             }
    //         }
    //     }
    //     primed_boards.clear();
    //     std::mem::swap(&mut primed_boards2, &mut primed_boards);
    // }

    solve(board.clone());
}
