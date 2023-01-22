mod dds;
mod deal;
mod hand;
mod payoff;
mod shape;
mod smartstack;
mod utils;

mod prelude {
    pub const SUITS: usize = 4;
    pub const SHAPE_COMBINATIONS: usize = 14usize.pow(4);
    pub const MAX_N_OF_BOARDS: u8 = 32;
    pub const RANKS: u8 = 13;
    pub const NUMBER_OF_HANDS: usize = 4;
    pub use crate::dds::*;
    pub use crate::deal::*;
    pub use crate::hand::*;
    pub use crate::payoff::*;
    pub use crate::shape::*;
    pub use crate::smartstack::*;
    pub use crate::utils::*;
    pub use bridge_deck::{Card, Cards, Suit};
    pub use colored::Colorize;
    pub use itertools::{any, Itertools};
    pub use rusty_dealer::*;
    pub use std::{
        array::IntoIter,
        collections::{HashMap, HashSet},
        error::Error,
        fmt,
        fs::{File, OpenOptions},
        hash::{Hash, Hasher},
        io::Write,
        str::FromStr,
    };
}

use prelude::*;

fn main() {
    let mut factory = ShapeFactory::new();
    factory.new_shape("3334").unwrap();
    factory.new_shape("(332)5").unwrap();
    factory.new_shape("2245").unwrap();
    let constraint = Constraints::BoundsAndPredeal(
        &|deal: &[Hand; 4], factory: &mut ShapeFactory| -> bool {
            deal[Seat::West as usize].hcp() > 11
                && deal[Seat::West as usize].hcp() < 18
                && factory.includes(&deal[Seat::East as usize])
                && deal[Seat::East as usize].hcp() > 11
                && deal[Seat::East as usize].hcp() < 15
                && deal[Seat::West as usize].spades().len() < 5
                && deal[Seat::West as usize].hearts().len() == 4
                && deal[Seat::West as usize].diamonds().len() < 5
        },
        [
            (Seat::West, None),
            (Seat::East, None),
            (Seat::North, None),
            (
                Seat::South,
                Some(Hand::from_str("S9S8S7S5S4S3DKDQD5D4HTH2CT").unwrap()),
            ),
        ],
    );
    //    let hearts = Contract::from_str("4HS", true).unwrap();
    //    let notrump = Contract::from_str("3NN", true).unwrap();
    //    let mut imps_payoff = Payoff::new(vec!["4♡S", "3NN"], imps);
    //    let mut raw_scores: HashMap<String, i32> = HashMap::new();
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("test.pbn")
        .unwrap();
    let mut found = 0;
    let goal = 10000;
    for _ in 0..10usize.pow(4) {
        if found > goal {
            break;
        }
        let deal = Deal::new_with_conditions(&constraint, &mut factory);
        found += 1;
        writeln!(&mut file, "{}", deal.as_pbn()).unwrap();
        //        let remainCards = populate(&deal);
        //        let hc_deal = crate::lib::deal {
        //            trump: 1,
        //            first: 3,
        //            currentTrickSuit: [0; 3],
        //            currentTrickRank: [0; 3],
        //            remainCards,
        //        };
        //        let nc_deal = crate::lib::deal {
        //            trump: 4,
        //            first: 1,
        //            currentTrickSuit: [0; 3],
        //            currentTrickRank: [0; 3],
        //            remainCards,
        //        };
        //        let mut futph = futureTricks {
        //            nodes: 0,
        //            cards: 0,
        //            suit: [0; 13],
        //            rank: [0; 13],
        //            equals: [0; 13],
        //            score: [0; 13],
        //        };
        //        let mut futpn = futureTricks {
        //            nodes: 0,
        //            cards: 0,
        //            suit: [0; 13],
        //            rank: [0; 13],
        //            equals: [0; 13],
        //            score: [0; 13],
        //        };
        //        unsafe {
        //            let ptrh: *mut futureTricks = &mut futph;
        //            let resh = SolveBoard(hc_deal, -1, 1, 0, ptrh, 0);
        //            let ptrn: *mut futureTricks = &mut futpn;
        //            let resn = SolveBoard(nc_deal, -1, 1, 0, ptrn, 0);
        //        }
        //        let hscore = hearts.score(13 - futph.score[0] as usize);
        //        raw_scores.insert(String::from("4♡S"), hscore);
        //        let nscore = notrump.score(13 - futpn.score[0] as usize);
        //        raw_scores.insert(String::from("3NN"), nscore);
        //        imps_payoff.add_data(&raw_scores);
        //        found += 1;
    }
    // imps_payoff.report();
}
