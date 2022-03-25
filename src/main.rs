#![allow(dead_code)]

use std::cmp::min;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::mem::MaybeUninit;
use std::hash::Hash;
use std::time::Instant;

// use bitintr::x86::bmi2;

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

fn flood(m: Move, val: u8, count: &mut u8, board: &mut [[u8; BOARD_SIZE]; BOARD_SIZE], visited: &mut [[bool; BOARD_SIZE]; BOARD_SIZE]) {
    let (x, y) = m.pos;
    if m.is_valid() {
        if !visited[x][y] && board[x][y] == val {
            board[x][y] = 0;
            visited[x][y] = true;
            *count += 1;
            flood(Move { pos: (x-1, y) }, val, count, board, visited);
            flood(Move { pos: (x+1, y) }, val, count, board, visited);
            flood(Move { pos: (x, y-1) }, val, count, board, visited);
            flood(Move { pos: (x, y+1) }, val, count, board, visited);
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
        self.pos.0 as i8 >= 0 && self.pos.0 < BOARD_SIZE &&
        self.pos.1 as i8 >= 0 && self.pos.1 < BOARD_SIZE
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
        // for i in 0..move_count/2 {
        //     let tmp = valid_moves[i];
        //     valid_moves[i] = valid_moves[move_count-1-i];
        //     valid_moves[move_count-1-i] = tmp;
        // }
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

    fn click3(&self, m: Move) -> Board {
        let mut board = self.board.clone();
        let (x, y) = m.pos;
        let val = board[x][y];
        let val = unsafe { *board.get_unchecked(x).get_unchecked(y) };

        
        // TODO: use PEXT/PCMPEQB
        let bool_board = board.map(|col| -> u8 {
            let mut new_col = 0u8;
            for i in 0..BOARD_SIZE {
                new_col |= if col[i] == val { 1 } else { 0 } << i;
            }
            new_col
        } );

        let mut flood_board = [0u8; BOARD_SIZE];
        flood_board[x] |= 1 << y;

        let mut changed = true;
        while changed {
            changed = false;
            // dilate columns TODO: pack bytes
            for i in 0..BOARD_SIZE {
                let dilated = flood_board[i] | (flood_board[i] << 1) | (flood_board[i] >> 1);
                let masked  = dilated & bool_board[i];
                let k = flood_board[i];
                if flood_board[i] != flood_board[i] | masked {
                    // update was made to the board
                    changed = true;
                    flood_board[i] |= masked;
                }
            }

            // dilate rows
            for i in 0..BOARD_SIZE {
                let dilated = match i {
                    0 => flood_board[0] | flood_board[i+1],
                    7 => flood_board[i-1] & flood_board[i],
                    _ => flood_board[i-1] | flood_board[i] | flood_board[i+1],
                };
                let masked = dilated & bool_board[i];
                let k = flood_board[i];
                if flood_board[i] != flood_board[i] | masked {
                    changed = true;
                    flood_board[i] |= masked;
                }
            }
        }
        let count = u64::count_ones(u64::from_ne_bytes(flood_board));

        // println!("({x} {y}) {val}");
        // println!("num iterations: {count}");
        // println!("{:^18}{:^18}{:^18}", "board", "bool", "flood");
        // for y in 0..BOARD_SIZE {
        //     for x in 0..BOARD_SIZE {
        //         print!("{} ", board[x][y]);
        //     }
        //     print!("  ");
        //     for x in 0..BOARD_SIZE {
        //         print!("{} ", (bool_board[x] & (1 << y)) >> y)
        //     }
        //     print!("  ");
        //     for x in 0..BOARD_SIZE {
        //         print!("{} ", (flood_board[x] & (1 << y)) >> y)
        //     }
        //     println!();
        // }
        // println!();

        // for col in flood_board {
        //     println!("{:08b}", col);
        // }
        // println!("");


        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                board[i][j] = board[i][j] & !(0xff * ((flood_board[i] & (1 << j)) >> j));
            }
        }

        board[x][y] = val + 1;
        let mut new_board = Board {
            board,
            score: get_score(count, val as u32),
            mve: m,
        };
        new_board.collapse();
        new_board
    }

    fn click2(&self, m: Move) -> Board {
        let (x, y) = m.pos;

        let val = self.board[x][y];
        let mut visited = [[false; BOARD_SIZE]; BOARD_SIZE];
        let mut board = self.board.clone();
        let mut count = 0;

        flood(m, val, &mut count, &mut board, &mut visited);

        board[x][y] = val + 1;

        let score = get_score(count as u32, val as u32);
        let mut new_board = Board { board, score, mve: m };
        new_board.collapse();
        new_board
    }

    fn click(&self, m: Move) -> Board {
        let (x, y) = m.pos;
        let val = self.board[x][y];
        let mut visited = [[false; 8]; 8];
        visited[x][y] = true;

        // no need to initialize 
        let mut closed_cells = unsafe {
            let data: [MaybeUninit<Move>; 32] = MaybeUninit::uninit().assume_init();
            std::mem::transmute::<_, [Move; 32]>(data)
        } ;
        let mut open_cells = unsafe {
            let data: [MaybeUninit<Move>; 32] = MaybeUninit::uninit().assume_init();
            std::mem::transmute::<_, [Move; 32]>(data)
        } ;

        let mut closed_count = 0;
        let mut open_count = 1;

        open_cells[0] = m;

        while open_count > 0 {
            let (x, y) = open_cells[open_count-1].pos;
            open_count -= 1;
            closed_cells[closed_count].pos = (x, y);
            closed_count += 1;
            for (dx, dy) in DELTAS.iter() {
                let (nx, ny) = (dx + x as i8, dy + y as i8);
                if nx >= 0 && nx < BOARD_SIZE as i8 && ny >= 0 && ny < BOARD_SIZE as i8 && !visited[nx as usize][ny as usize] {
                    visited[nx as usize][ny as usize] = true;
                    if self.board[nx as usize][ny as usize] == val {
                        open_cells[open_count].pos = (nx as usize, ny as usize);
                        open_count += 1;
                    }
                }
            }
        }
        
        let score = get_score(closed_count as u32, val as u32);
        
        let mut new_board = Board {
            board: self.board.clone(),
            mve: m.clone(),
            score: score,
        };

        for m in &mut closed_cells[0..closed_count] {
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
                let m = stack.frames[stack.frame_count-1].moves.get(stack.frames[stack.frame_count-1].num_moves-1).unwrap(); // TODO: get_unchecked
                stack.frames[stack.frame_count-1].num_moves -= 1;

                // stack.frames[stack.frame_count].board = stack.frames[stack.frame_count-1].board.click(*m);
                // stack.frames[stack.frame_count].board = stack.frames[stack.frame_count-1].board.click2(*m);
                stack.frames[stack.frame_count].board = stack.frames[stack.frame_count-1].board.click3(*m);
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

    // println!("score: {}", board.click3(Move {pos: (0, 3)}).score);
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
