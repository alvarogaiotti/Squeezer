// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use std::sync::OnceLock;

use dds::deal::{DdsHandEncoding, DdsSuit};

#[allow(dead_code)]
pub static DEAL: OnceLock<DealMock> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct DealMock {
    pub hands: [[usize; 4]; 4],
}

impl IntoIterator for DealMock {
    type Item = [usize; 4];
    type IntoIter = std::array::IntoIter<[usize; 4], 4>;
    fn into_iter(self) -> Self::IntoIter {
        self.hands.into_iter()
    }
}

impl dds::deal::AsDDSDeal for DealMock {
    fn to_dds_deal(&self) -> dds::deal::DDSDealRepr {
        let mut remain_cards = [[0; 4]; 4];
        for (seat, hand) in self.clone().into_iter().enumerate() {
            for (index, suit) in hand.into_iter().enumerate() {
                remain_cards[seat][index] = (suit >> (16 * index)) as u32;
            }
        }
        remain_cards.into()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ContractMock {}

impl dds::traits::ContractScorer for ContractMock {
    fn score(&self, _tricks: u8) -> i32 {
        0
    }
}

impl dds::traits::AsDDSContract for ContractMock {
    fn as_dds_contract(&self) -> (DdsSuit, DdsHandEncoding) {
        (DdsSuit::Diamonds, DdsHandEncoding::West)
    }
}

pub fn initialize_test() -> DealMock {
    /*
    *       ♠K93
           ♡JT9862
           ♢9
           ♣K73

    ♠T4           ♠AQJ
    ♡Q            ♡75
    ♢KQT543       ♢AJ2
    ♣QT85         ♣J9642

           ♠87652
           ♡AK43
           ♢876
           ♣A
    */

    DealMock {
        hands: [
            [8712, 256114688, 2199023255552, 2344123606046343168],
            [22528, 10485760, 79182017069056, 744219838422974464],
            [484, 1612185600, 1924145348608, 4611686018427387904],
            [1040, 268435456, 57415122812928, 1522216674051227648],
        ],
    }
}
