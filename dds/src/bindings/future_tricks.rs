use core::ffi::c_int;

use super::{ddsffi::futureTricks, RawDDS};

#[derive(Debug, RawDDS)]
pub struct FutureTricks(#[raw] pub futureTricks);

impl FutureTricks {
    pub fn new() -> Self {
        Self(futureTricks::default())
    }
    pub fn score(&self) -> &[c_int; 13] {
        &self.0.score
    }
}

impl Default for futureTricks {
    fn default() -> Self {
        futureTricks {
            nodes: 0,
            cards: 0,
            suit: [0; 13],
            rank: [0; 13],
            equals: [0; 13],
            score: [0; 13],
        }
    }
}
