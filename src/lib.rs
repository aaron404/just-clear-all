#![feature(portable_simd)]
#![feature(stdsimd)]

use core::fmt;
use std::collections::HashMap;
use std::time::Instant;
use std::{mem, cmp};
use std::simd;
use std::simd::ToBitMask;

use std::arch::x86_64;

use arrayvec::ArrayVec;

pub const BOARD_SIZE: usize = 8;
const DELTAS: [(i8, i8); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

const LMASK: u64 = 0x7f7f7f7f7f7f7f7f;
const RMASK: u64 = 0xfefefefefefefefe;
static mut LEVEL: u32 = 0;
const STACK_SIZE: usize = 64;

mod masks;

const SHUFFLE_MASKS_LOW: [u64; 256] = [
    0x8080808080808080, 0x0080808080808080, 0x0180808080808080, 0x0100808080808080, 0x0280808080808080, 0x0200808080808080, 0x0201808080808080, 0x0201008080808080, // 8
    0x0380808080808080, 0x0300808080808080, 0x0301808080808080, 0x0301008080808080, 0x0302808080808080, 0x0302008080808080, 0x0302018080808080, 0x0302010080808080, // 16
    0x0480808080808080, 0x0400808080808080, 0x0401808080808080, 0x0401008080808080, 0x0402808080808080, 0x0402008080808080, 0x0402018080808080, 0x0402010080808080, // 24
    0x0403808080808080, 0x0403008080808080, 0x0403018080808080, 0x0403010080808080, 0x0403028080808080, 0x0403020080808080, 0x0403020180808080, 0x0403020100808080, // 32
    0x0580808080808080, 0x0500808080808080, 0x0501808080808080, 0x0501008080808080, 0x0502808080808080, 0x0502008080808080, 0x0502018080808080, 0x0502010080808080, // 40
    0x0503808080808080, 0x0503008080808080, 0x0503018080808080, 0x0503010080808080, 0x0503028080808080, 0x0503020080808080, 0x0503020180808080, 0x0503020100808080, // 48
    0x0504808080808080, 0x0504008080808080, 0x0504018080808080, 0x0504010080808080, 0x0504028080808080, 0x0504020080808080, 0x0504020180808080, 0x0504020100808080, // 56
    0x0504038080808080, 0x0504030080808080, 0x0504030180808080, 0x0504030100808080, 0x0504030280808080, 0x0504030200808080, 0x0504030201808080, 0x0504030201008080, // 64
    0x0680808080808080, 0x0600808080808080, 0x0601808080808080, 0x0601008080808080, 0x0602808080808080, 0x0602008080808080, 0x0602018080808080, 0x0602010080808080, // 72
    0x0603808080808080, 0x0603008080808080, 0x0603018080808080, 0x0603010080808080, 0x0603028080808080, 0x0603020080808080, 0x0603020180808080, 0x0603020100808080, // 80
    0x0604808080808080, 0x0604008080808080, 0x0604018080808080, 0x0604010080808080, 0x0604028080808080, 0x0604020080808080, 0x0604020180808080, 0x0604020100808080, // 88
    0x0604038080808080, 0x0604030080808080, 0x0604030180808080, 0x0604030100808080, 0x0604030280808080, 0x0604030200808080, 0x0604030201808080, 0x0604030201008080, // 96
    0x0605808080808080, 0x0605008080808080, 0x0605018080808080, 0x0605010080808080, 0x0605028080808080, 0x0605020080808080, 0x0605020180808080, 0x0605020100808080, // 104
    0x0605038080808080, 0x0605030080808080, 0x0605030180808080, 0x0605030100808080, 0x0605030280808080, 0x0605030200808080, 0x0605030201808080, 0x0605030201008080, // 112
    0x0605048080808080, 0x0605040080808080, 0x0605040180808080, 0x0605040100808080, 0x0605040280808080, 0x0605040200808080, 0x0605040201808080, 0x0605040201008080, // 120
    0x0605040380808080, 0x0605040300808080, 0x0605040301808080, 0x0605040301008080, 0x0605040302808080, 0x0605040302008080, 0x0605040302018080, 0x0605040302010080, // 128
    0x0780808080808080, 0x0700808080808080, 0x0701808080808080, 0x0701008080808080, 0x0702808080808080, 0x0702008080808080, 0x0702018080808080, 0x0702010080808080, // 136
    0x0703808080808080, 0x0703008080808080, 0x0703018080808080, 0x0703010080808080, 0x0703028080808080, 0x0703020080808080, 0x0703020180808080, 0x0703020100808080, // 144
    0x0704808080808080, 0x0704008080808080, 0x0704018080808080, 0x0704010080808080, 0x0704028080808080, 0x0704020080808080, 0x0704020180808080, 0x0704020100808080, // 152
    0x0704038080808080, 0x0704030080808080, 0x0704030180808080, 0x0704030100808080, 0x0704030280808080, 0x0704030200808080, 0x0704030201808080, 0x0704030201008080, // 160
    0x0705808080808080, 0x0705008080808080, 0x0705018080808080, 0x0705010080808080, 0x0705028080808080, 0x0705020080808080, 0x0705020180808080, 0x0705020100808080, // 168
    0x0705038080808080, 0x0705030080808080, 0x0705030180808080, 0x0705030100808080, 0x0705030280808080, 0x0705030200808080, 0x0705030201808080, 0x0705030201008080, // 176
    0x0705048080808080, 0x0705040080808080, 0x0705040180808080, 0x0705040100808080, 0x0705040280808080, 0x0705040200808080, 0x0705040201808080, 0x0705040201008080, // 184
    0x0705040380808080, 0x0705040300808080, 0x0705040301808080, 0x0705040301008080, 0x0705040302808080, 0x0705040302008080, 0x0705040302018080, 0x0705040302010080, // 192
    0x0706808080808080, 0x0706008080808080, 0x0706018080808080, 0x0706010080808080, 0x0706028080808080, 0x0706020080808080, 0x0706020180808080, 0x0706020100808080, // 200
    0x0706038080808080, 0x0706030080808080, 0x0706030180808080, 0x0706030100808080, 0x0706030280808080, 0x0706030200808080, 0x0706030201808080, 0x0706030201008080, // 208
    0x0706048080808080, 0x0706040080808080, 0x0706040180808080, 0x0706040100808080, 0x0706040280808080, 0x0706040200808080, 0x0706040201808080, 0x0706040201008080, // 216
    0x0706040380808080, 0x0706040300808080, 0x0706040301808080, 0x0706040301008080, 0x0706040302808080, 0x0706040302008080, 0x0706040302018080, 0x0706040302010080, // 224
    0x0706058080808080, 0x0706050080808080, 0x0706050180808080, 0x0706050100808080, 0x0706050280808080, 0x0706050200808080, 0x0706050201808080, 0x0706050201008080, // 232
    0x0706050380808080, 0x0706050300808080, 0x0706050301808080, 0x0706050301008080, 0x0706050302808080, 0x0706050302008080, 0x0706050302018080, 0x0706050302010080, // 240
    0x0706050480808080, 0x0706050400808080, 0x0706050401808080, 0x0706050401008080, 0x0706050402808080, 0x0706050402008080, 0x0706050402018080, 0x0706050402010080, // 248
    0x0706050403808080, 0x0706050403008080, 0x0706050403018080, 0x0706050403010080, 0x0706050403028080, 0x0706050403020080, 0x0706050403020180, 0x0706050403020100, // 256
];

const SHUFFLE_MASKS_HIGH: [u64; 256] = [
    0x8888888888888888, 0x0888888888888888, 0x0988888888888888, 0x0908888888888888, 0x0a88888888888888, 0x0a08888888888888, 0x0a09888888888888, 0x0a09088888888888, // 8
    0x0b88888888888888, 0x0b08888888888888, 0x0b09888888888888, 0x0b09088888888888, 0x0b0a888888888888, 0x0b0a088888888888, 0x0b0a098888888888, 0x0b0a090888888888, // 16
    0x0c88888888888888, 0x0c08888888888888, 0x0c09888888888888, 0x0c09088888888888, 0x0c0a888888888888, 0x0c0a088888888888, 0x0c0a098888888888, 0x0c0a090888888888, // 24
    0x0c0b888888888888, 0x0c0b088888888888, 0x0c0b098888888888, 0x0c0b090888888888, 0x0c0b0a8888888888, 0x0c0b0a0888888888, 0x0c0b0a0988888888, 0x0c0b0a0908888888, // 32
    0x0d88888888888888, 0x0d08888888888888, 0x0d09888888888888, 0x0d09088888888888, 0x0d0a888888888888, 0x0d0a088888888888, 0x0d0a098888888888, 0x0d0a090888888888, // 40
    0x0d0b888888888888, 0x0d0b088888888888, 0x0d0b098888888888, 0x0d0b090888888888, 0x0d0b0a8888888888, 0x0d0b0a0888888888, 0x0d0b0a0988888888, 0x0d0b0a0908888888, // 48
    0x0d0c888888888888, 0x0d0c088888888888, 0x0d0c098888888888, 0x0d0c090888888888, 0x0d0c0a8888888888, 0x0d0c0a0888888888, 0x0d0c0a0988888888, 0x0d0c0a0908888888, // 56
    0x0d0c0b8888888888, 0x0d0c0b0888888888, 0x0d0c0b0988888888, 0x0d0c0b0908888888, 0x0d0c0b0a88888888, 0x0d0c0b0a08888888, 0x0d0c0b0a09888888, 0x0d0c0b0a09088888, // 64
    0x0e88888888888888, 0x0e08888888888888, 0x0e09888888888888, 0x0e09088888888888, 0x0e0a888888888888, 0x0e0a088888888888, 0x0e0a098888888888, 0x0e0a090888888888, // 72
    0x0e0b888888888888, 0x0e0b088888888888, 0x0e0b098888888888, 0x0e0b090888888888, 0x0e0b0a8888888888, 0x0e0b0a0888888888, 0x0e0b0a0988888888, 0x0e0b0a0908888888, // 80
    0x0e0c888888888888, 0x0e0c088888888888, 0x0e0c098888888888, 0x0e0c090888888888, 0x0e0c0a8888888888, 0x0e0c0a0888888888, 0x0e0c0a0988888888, 0x0e0c0a0908888888, // 88
    0x0e0c0b8888888888, 0x0e0c0b0888888888, 0x0e0c0b0988888888, 0x0e0c0b0908888888, 0x0e0c0b0a88888888, 0x0e0c0b0a08888888, 0x0e0c0b0a09888888, 0x0e0c0b0a09088888, // 96
    0x0e0d888888888888, 0x0e0d088888888888, 0x0e0d098888888888, 0x0e0d090888888888, 0x0e0d0a8888888888, 0x0e0d0a0888888888, 0x0e0d0a0988888888, 0x0e0d0a0908888888, // 104
    0x0e0d0b8888888888, 0x0e0d0b0888888888, 0x0e0d0b0988888888, 0x0e0d0b0908888888, 0x0e0d0b0a88888888, 0x0e0d0b0a08888888, 0x0e0d0b0a09888888, 0x0e0d0b0a09088888, // 112
    0x0e0d0c8888888888, 0x0e0d0c0888888888, 0x0e0d0c0988888888, 0x0e0d0c0908888888, 0x0e0d0c0a88888888, 0x0e0d0c0a08888888, 0x0e0d0c0a09888888, 0x0e0d0c0a09088888, // 120
    0x0e0d0c0b88888888, 0x0e0d0c0b08888888, 0x0e0d0c0b09888888, 0x0e0d0c0b09088888, 0x0e0d0c0b0a888888, 0x0e0d0c0b0a088888, 0x0e0d0c0b0a098888, 0x0e0d0c0b0a090888, // 128
    0x0f88888888888888, 0x0f08888888888888, 0x0f09888888888888, 0x0f09088888888888, 0x0f0a888888888888, 0x0f0a088888888888, 0x0f0a098888888888, 0x0f0a090888888888, // 136
    0x0f0b888888888888, 0x0f0b088888888888, 0x0f0b098888888888, 0x0f0b090888888888, 0x0f0b0a8888888888, 0x0f0b0a0888888888, 0x0f0b0a0988888888, 0x0f0b0a0908888888, // 144
    0x0f0c888888888888, 0x0f0c088888888888, 0x0f0c098888888888, 0x0f0c090888888888, 0x0f0c0a8888888888, 0x0f0c0a0888888888, 0x0f0c0a0988888888, 0x0f0c0a0908888888, // 152
    0x0f0c0b8888888888, 0x0f0c0b0888888888, 0x0f0c0b0988888888, 0x0f0c0b0908888888, 0x0f0c0b0a88888888, 0x0f0c0b0a08888888, 0x0f0c0b0a09888888, 0x0f0c0b0a09088888, // 160
    0x0f0d888888888888, 0x0f0d088888888888, 0x0f0d098888888888, 0x0f0d090888888888, 0x0f0d0a8888888888, 0x0f0d0a0888888888, 0x0f0d0a0988888888, 0x0f0d0a0908888888, // 168
    0x0f0d0b8888888888, 0x0f0d0b0888888888, 0x0f0d0b0988888888, 0x0f0d0b0908888888, 0x0f0d0b0a88888888, 0x0f0d0b0a08888888, 0x0f0d0b0a09888888, 0x0f0d0b0a09088888, // 176
    0x0f0d0c8888888888, 0x0f0d0c0888888888, 0x0f0d0c0988888888, 0x0f0d0c0908888888, 0x0f0d0c0a88888888, 0x0f0d0c0a08888888, 0x0f0d0c0a09888888, 0x0f0d0c0a09088888, // 184
    0x0f0d0c0b88888888, 0x0f0d0c0b08888888, 0x0f0d0c0b09888888, 0x0f0d0c0b09088888, 0x0f0d0c0b0a888888, 0x0f0d0c0b0a088888, 0x0f0d0c0b0a098888, 0x0f0d0c0b0a090888, // 192
    0x0f0e888888888888, 0x0f0e088888888888, 0x0f0e098888888888, 0x0f0e090888888888, 0x0f0e0a8888888888, 0x0f0e0a0888888888, 0x0f0e0a0988888888, 0x0f0e0a0908888888, // 200
    0x0f0e0b8888888888, 0x0f0e0b0888888888, 0x0f0e0b0988888888, 0x0f0e0b0908888888, 0x0f0e0b0a88888888, 0x0f0e0b0a08888888, 0x0f0e0b0a09888888, 0x0f0e0b0a09088888, // 208
    0x0f0e0c8888888888, 0x0f0e0c0888888888, 0x0f0e0c0988888888, 0x0f0e0c0908888888, 0x0f0e0c0a88888888, 0x0f0e0c0a08888888, 0x0f0e0c0a09888888, 0x0f0e0c0a09088888, // 216
    0x0f0e0c0b88888888, 0x0f0e0c0b08888888, 0x0f0e0c0b09888888, 0x0f0e0c0b09088888, 0x0f0e0c0b0a888888, 0x0f0e0c0b0a088888, 0x0f0e0c0b0a098888, 0x0f0e0c0b0a090888, // 224
    0x0f0e0d8888888888, 0x0f0e0d0888888888, 0x0f0e0d0988888888, 0x0f0e0d0908888888, 0x0f0e0d0a88888888, 0x0f0e0d0a08888888, 0x0f0e0d0a09888888, 0x0f0e0d0a09088888, // 232
    0x0f0e0d0b88888888, 0x0f0e0d0b08888888, 0x0f0e0d0b09888888, 0x0f0e0d0b09088888, 0x0f0e0d0b0a888888, 0x0f0e0d0b0a088888, 0x0f0e0d0b0a098888, 0x0f0e0d0b0a090888, // 240
    0x0f0e0d0c88888888, 0x0f0e0d0c08888888, 0x0f0e0d0c09888888, 0x0f0e0d0c09088888, 0x0f0e0d0c0a888888, 0x0f0e0d0c0a088888, 0x0f0e0d0c0a098888, 0x0f0e0d0c0a090888, // 248
    0x0f0e0d0c0b888888, 0x0f0e0d0c0b088888, 0x0f0e0d0c0b098888, 0x0f0e0d0c0b090888, 0x0f0e0d0c0b0a8888, 0x0f0e0d0c0b0a0888, 0x0f0e0d0c0b0a0988, 0x0f0e0d0c0b0a0908, // 256
];

fn get_score(num_tiles: u32, tile_value: u8) -> u32 {
    match num_tiles {
        1 => 0,
        _ => {
            let multiplier = cmp::min(1 + num_tiles / 5, 3);
            tile_value as u32 * 5 * num_tiles * multiplier
        }
    }
}

pub fn set_level(level: u32) {
    unsafe { LEVEL = level; }
}

#[derive(Clone)]
#[repr(align(64))]
pub struct Board {
    pub board: [[u8; BOARD_SIZE]; BOARD_SIZE],
    score: u32,
}

#[derive(PartialEq, Eq, Hash)]
struct BoardHash {
    board_hash: [u128; 2],
}

impl Board {
    pub fn new() -> Board {
        Board {
            board: [[0; BOARD_SIZE]; BOARD_SIZE],
            score: 0,
        }
    }

    pub fn from_rows(rows: [u32; BOARD_SIZE]) -> Board {
        let mut board = Board::new();
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let shift = 4 * x;
                board.board[BOARD_SIZE - x - 1][y] = ((rows[y] & (0xf << shift)) >> shift) as u8;
            }
        }
        board
    }

    fn is_cleared(&self) -> bool {
        self.board[0][BOARD_SIZE-2] == 0 && self.board[1][BOARD_SIZE-1] == 0
    }

    fn get_tiles_remaining(&self) -> u8 {
        let mut count = 0;
        for i in 0 .. BOARD_SIZE {
            for j in 0 .. BOARD_SIZE {
                if self.board[j][i] > 0 {
                    count += 1;
                }
            }
        }
        count
    }
    
    // TODO: lookup table
    fn get_bonus(&self) -> u32 {
        let n = self.get_tiles_remaining() as u32;
        assert!(n > 0);
        unsafe {
            if n == 1 {
                return 500 * (LEVEL + 1);
            } else if n < 6 {
                return (250 - 50 * (n - 2)) * (LEVEL + 1);
            }
        }
        0
    }


    pub fn get_valid_moves(&self, valid_moves: &mut ArrayVec<(usize, usize), STACK_SIZE>) {
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

        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                if valid[x][y] == 1 {
                    valid_moves.push((x, y));
                }
            }
        }
    }

    pub fn get_valid_moves_simd(&self) -> u64 {
        for window in self.board.windows(2) {
            let (_, left, _) = window[0].as_simd::<8>();
            let (_, right, _) = window[1].as_simd::<8>();

            // let a = left.
        }

        0
    }

    pub fn collapse_scalar(&mut self) {
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

    #[target_feature(enable = "avx2")]
    pub unsafe fn collapse_v_simd(&mut self) {
        let board = simd::u8x64::from_array(mem::transmute(self.board));
        let bitmask = board.lanes_ne(simd::u8x64::splat(0)).to_bitmask();

        let bytes: [u8; 8] = bitmask.to_ne_bytes(); // mem::transmute(bitmask);
        let shuffle_mask_lo = [
            SHUFFLE_MASKS_LOW[bytes[0] as usize],
            SHUFFLE_MASKS_HIGH[bytes[1] as usize],
            SHUFFLE_MASKS_LOW[bytes[2] as usize],
            SHUFFLE_MASKS_HIGH[bytes[3] as usize],
        ];
        let shuffle_mask_hi = [
            SHUFFLE_MASKS_LOW[bytes[4] as usize],
            SHUFFLE_MASKS_HIGH[bytes[5] as usize],
            SHUFFLE_MASKS_LOW[bytes[6] as usize],
            SHUFFLE_MASKS_HIGH[bytes[7] as usize],
        ];

        let shuffle_mask_lo = x86_64::_mm256_lddqu_si256(mem::transmute(shuffle_mask_lo.as_ptr()));
        let board_lo = x86_64::_mm256_lddqu_si256(mem::transmute(&self.board[0]));
        let shuffled_lo = x86_64::_mm256_shuffle_epi8(board_lo, shuffle_mask_lo);
        let shuffle_mask_hi = x86_64::_mm256_lddqu_si256(mem::transmute(shuffle_mask_hi.as_ptr()));
        let board_hi = x86_64::_mm256_lddqu_si256(mem::transmute(&self.board[4]));
        let shuffled_hi = x86_64::_mm256_shuffle_epi8(board_hi, shuffle_mask_hi);

        x86_64::_mm256_storeu_si256(mem::transmute(&self.board[0]), shuffled_lo);
        x86_64::_mm256_storeu_si256(mem::transmute(&self.board[4]), shuffled_hi);
    }

    pub fn collapse_simd(&mut self) {
        unsafe { self.collapse_v_simd(); }
        self.collapse_h();
    }

    pub fn collapse_simd2(&mut self) {
        unsafe { self.collapse_v_simd2(); }
        self.collapse_h();
    }

    #[target_feature(enable = "avx2")]
    unsafe fn collapse_v_simd2(&mut self) {
        // requires avx512 x86_64::_mm_maskz_compress_epi8(k, a)
        let board_ll = x86_64::_mm_lddqu_si128(mem::transmute(&self.board[0]));
        let board_lh = x86_64::_mm_lddqu_si128(mem::transmute(&self.board[2]));
        let board_hl = x86_64::_mm_lddqu_si128(mem::transmute(&self.board[4]));
        let board_hh = x86_64::_mm_lddqu_si128(mem::transmute(&self.board[6]));

        let zeros = x86_64::_mm_setzero_si128();

        let masked_ll = x86_64::_mm_cmpeq_epi8(board_ll, zeros);
        let masked_lh = x86_64::_mm_cmpeq_epi8(board_lh, zeros);
        let masked_hl = x86_64::_mm_cmpeq_epi8(board_hl, zeros);
        let masked_hh = x86_64::_mm_cmpeq_epi8(board_hh, zeros);

        let bits_ll = x86_64::_mm_movemask_epi8(masked_ll) as usize;
        let bits_lh = x86_64::_mm_movemask_epi8(masked_lh) as usize;
        let bits_hl = x86_64::_mm_movemask_epi8(masked_hl) as usize;
        let bits_hh = x86_64::_mm_movemask_epi8(masked_hh) as usize;
    
        let mask_ll = x86_64::_mm_lddqu_si128(mem::transmute(&masks::SHUFFLE_MASKS_128[bits_ll]));
        let mask_lh = x86_64::_mm_lddqu_si128(mem::transmute(&masks::SHUFFLE_MASKS_128[bits_lh]));
        let mask_hl = x86_64::_mm_lddqu_si128(mem::transmute(&masks::SHUFFLE_MASKS_128[bits_hl]));
        let mask_hh = x86_64::_mm_lddqu_si128(mem::transmute(&masks::SHUFFLE_MASKS_128[bits_hh]));

        let shuffled_ll = x86_64::_mm_shuffle_epi8(board_ll, mask_ll);
        let shuffled_lh = x86_64::_mm_shuffle_epi8(board_lh, mask_lh);
        let shuffled_hl = x86_64::_mm_shuffle_epi8(board_hl, mask_hl);
        let shuffled_hh = x86_64::_mm_shuffle_epi8(board_hh, mask_hh);

        // let mut mask = 0i32;
        // mask |= x86_64::_mm_movemask_epi8(x86_64::_mm_cmpeq_epi8(shuffled_ll, zeros));
        // mask |= x86_64::_mm_movemask_epi8(x86_64::_mm_cmpeq_epi8(shuffled_lh, zeros)) << 2;
        // mask |= x86_64::_mm_movemask_epi8(x86_64::_mm_cmpeq_epi8(shuffled_hl, zeros)) << 4;
        // mask |= x86_64::_mm_movemask_epi8(x86_64::_mm_cmpeq_epi8(shuffled_hh, zeros)) << 6;
        // mask = mask & 0x55555555;
        // mask = mask | (mask >> 7);
        // mask = mask & 0xff;

        x86_64::_mm_storeu_si128(mem::transmute(&mut self.board[0]), shuffled_ll);
        x86_64::_mm_storeu_si128(mem::transmute(&mut self.board[2]), shuffled_lh);
        x86_64::_mm_storeu_si128(mem::transmute(&mut self.board[4]), shuffled_hl);
        x86_64::_mm_storeu_si128(mem::transmute(&mut self.board[6]), shuffled_hh);

        //x86_64::mm_cmp_epi8
        //let shuffle_mask_lo = x86_64::_mm256_lddqu_si256(mem::transmute(shuffle_mask_lo.as_ptr()));
    }

    fn collapse_h(&mut self) {
        let mut left = 0;
        let mut right = 1;
        while left < BOARD_SIZE-1 && right < BOARD_SIZE {
            match (self.board[left][7], self.board[right][7]) {
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

    #[target_feature(enable = "avx2")]
    pub unsafe fn click2(&self, pos: (usize, usize)) -> Board {
        let (x, y) = pos;
        let val = self.board[x][y];

        

        Board::new()
    }

    pub fn click(&self, m: &(usize, usize)) -> Board {
        let (x, y) = *m;
        let val = self.board[x][y];
        let mut visited = [[false; 8]; 8];
        visited[x][y] = true;

        let mut new_board = self.board.clone();

        let mut open_cells = ArrayVec::<(usize, usize), 64>::new();
        let mut closed_count = 0u32;

        open_cells.push(*m);

        while open_cells.len() > 0 {
            let (x, y) = unsafe { open_cells.pop().unwrap_unchecked() };
            closed_count += 1;
            new_board[x][y] = 0;
            for (dx, dy) in DELTAS.iter() {
                let (nx, ny) = (dx + x as i8, dy + y as i8);
                if nx >= 0 && nx < BOARD_SIZE as i8 && ny >= 0 && ny < BOARD_SIZE as i8 && !visited[nx as usize][ny as usize] {
                    visited[nx as usize][ny as usize] = true;
                    if self.board[nx as usize][ny as usize] == val {
                        open_cells.push((nx as usize, ny as usize));
                    }
                }
            }
        }
        new_board[x][y] = val + 1;
        
        let score = get_score(closed_count, val as u8) + self.score;
        
        Board {
            board: new_board,
            score: score,
        }
    }


    #[target_feature(enable = "avx2")]
    pub unsafe fn click3(&self, pos: (usize, usize)) -> Board {
        let (x, y) = pos;
        let val = self.board[x][y];

        let brd = simd::u8x64::from_array(std::mem::transmute(self.board));
        let bools = brd.lanes_eq(simd::u8x64::splat(val)).to_bitmask();
        let mut flood = 1u64 << (x * BOARD_SIZE + y);

        let mut done = false;
        while !done {
            let dilated = flood | (flood << 8) | (flood >> 8) | ((flood & LMASK) << 1) | ((flood & RMASK) >> 1);
            let masked = dilated & bools;
            if masked == flood {
                done = true;
            } else {
                flood = masked;
            }

        }

        let count = flood.count_ones();

        let bitmask = !std::simd::mask8x64::from_bitmask(flood);

        let mul = brd * (bitmask.to_int() * std::simd::i8x64::splat(-1)).cast::<u8>();

        let mut result: [[u8; 8]; 8] = std::mem::transmute(mul.to_array());
        result[x][y] = val + 1;

        Board {
            board: result,
            score: get_score(count, val),
        }
    }

    pub fn solve2(&self) {
        
    }

    pub fn solve(&self) {
        let mut stack = ArrayVec::<Frame, STACK_SIZE>::new();

        let mut frame = Frame {
            board: self.clone(),
            moves: ArrayVec::new(),
        };
        frame.board.get_valid_moves(&mut frame.moves);
        stack.push( frame);

        let mut best_score = 0;
        let mut hashes: HashMap<BoardHash, u32> = HashMap::new();
        let mut cache_hit_keep_count = 0u64;
        let mut cache_hit_replace_count = 0u64;
        let mut cache_hit_insert_count = 0u64;

        println!("Solving Board:{self}");
        let t0 = Instant::now();

        while stack.len() > 0 {

            // print stack
            // println!("stack: ");
            // for f in stack.iter() {
            //     for m in f.moves.iter() {
            //         print!("{:?}, ",  m);
            //     }
            //     println!("");
            // }

            let frame = stack.last_mut().unwrap();

            // println!("{}", frame.board);
            // println!("{:?}", frame.moves);
            match frame.moves.len() {
                0 => {

                    // no moves left for this frame, calculate score and pop it
                    let score = frame.board.score + frame.board.get_bonus();
                    if score > best_score {
                        best_score = score;
                        let elapsed = (Instant::now() - t0).as_millis();
                        let count = cache_hit_keep_count + cache_hit_replace_count + cache_hit_insert_count;
                        let rate = count as f32 / (Instant::now() - t0).as_secs_f32() * 0.000001;
                        print!("[{:02}:{:02}:{:02}.{:03}] ", elapsed / 3600000, (elapsed / 60000) % 60, (elapsed / 1000) % 60, elapsed % 1000);
                        if frame.board.is_cleared() {
                            print!("[P] ");
                        }
                        print!("{score} ");
                        print!("[{cache_hit_insert_count}/{cache_hit_replace_count}/{cache_hit_keep_count}][{rate}]");
                        print!(": [");
                        for frame in stack.iter() {
                            match frame.moves.last() {
                                Some(x) => { print!("{:?}, ", x) },
                                None => {},
                            }
                        }
                        println!("]");
                    }
                   
                    stack.pop();
                    if stack.len() > 0 {
                        stack.last_mut().unwrap().moves.pop();
                    }
                    // panic!();
                },
                _ => {
                    let m = frame.moves.last().unwrap();
                    // println!("{:?}", m);
                    let mut board = frame.board.click(&m);
                    unsafe { board.collapse_simd2(); }

                    
                    let h = board.hash();
                    match hashes.get(&h) {
                        Some(val) if *val > board.score => { cache_hit_keep_count += 1; frame.moves.pop(); continue; },
                        Some(_) => { cache_hit_replace_count += 1 }
                        _ => { hashes.insert(h, board.score); cache_hit_insert_count += 1;  },
                    }
                    // cache_hit_insert_count += 1;

                    let mut moves = ArrayVec::new();
                    board.get_valid_moves(&mut moves);
                    let new_frame = Frame {
                        board: board,
                        moves: moves,
                    };
                    stack.push(new_frame);
                }
            }
        }
    }

    fn hash(&self) -> BoardHash {
        BoardHash { board_hash: [
            // unsafe { mem::transmute( *(&self.board[0])) },
            // unsafe { mem::transmute( *(&self.board[4])) },
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

impl std::hash::Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        
    }
}

struct Frame {
    board: Board,
    moves: ArrayVec<(usize, usize), STACK_SIZE>,
}