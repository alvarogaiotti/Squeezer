use rusty_dealer_macros::*;
use std::ffi::c_int;

mod analyseplay;
mod ddserror;
mod ddsffi;
mod deal;
mod future_tricks;
mod solver;
mod traits;
mod utils;
pub use analyseplay::*;
pub use ddserror::DDSError;
pub use ddsffi::*;
pub use deal::*;
pub use solver::*;
pub use traits::*;
pub use utils::*;

pub struct DoubleDummySolver {}

pub struct DDSCalc {}

impl DoubleDummySolver {
    pub fn solver() -> DDSSolver {
        DDSSolver {}
    }

    pub fn play_analyzer() -> impl PlayAnalyzer {
        DDSPlayAnalyzer {}
    }

    pub fn calculator() -> DDSCalc {
        DDSCalc {}
    }

    fn set_max_threads(user_threads: ThreadIndex) {
        unsafe { ddsffi::SetMaxThreads(user_threads.into()) }
    }

    fn set_resources(max_memory_mb: i32, max_threads: ThreadIndex) {
        unsafe { ddsffi::SetResources(max_memory_mb, max_threads.into()) }
    }
}

fn dd_score<D: AsDDSDeal, C: AsDDSContract + ContractScorer>(
    deal: &D,
    contract: &C,
) -> Result<i32, Box<dyn std::error::Error>> {
    let solver = DoubleDummySolver::solver();
    let tricks = solver.dd_tricks(deal, contract)?;
    Ok(contract.score(tricks))
}

#[cfg(test)]
mod test {
    use super::BridgeSolver;
    use std::cell::OnceCell;

    const DEAL: OnceCell<DealMock> = OnceCell::new();

    #[derive(Debug, Clone)]
    struct DealMock {
        hands: [[usize; 4]; 4],
    }

    impl IntoIterator for DealMock {
        type Item = [usize; 4];
        type IntoIter = std::array::IntoIter<[usize; 4], 4>;
        fn into_iter(self) -> Self::IntoIter {
            self.hands.into_iter()
        }
    }

    impl super::deal::AsDDSDeal for DealMock {
        fn as_dds_deal(&self) -> super::deal::DDSDealRepr {
            let mut remain_cards = [[0; 4]; 4];
            for (seat, hand) in self.clone().into_iter().enumerate() {
                for (index, suit) in hand.into_iter().enumerate() {
                    remain_cards[seat][index] = (suit >> (16 * index)) as u32;
                }
            }
            crate::DDSDealRepr::new(remain_cards)
        }
    }

    struct ContractMock {}

    impl crate::ContractScorer for ContractMock {
        fn score(&self, tricks: u8) -> i32 {
            0
        }
    }

    impl crate::AsDDSContract for ContractMock {
        fn as_dds_contract(&self) -> (i32, i32) {
            (2, 2)
        }
    }

    fn initialize_test() -> DealMock {
        DEAL.get_or_init(|| DealMock {
            hands: [
                [1580, 3145728, 71468255805440, 5215168368495034368],
                [26624, 1233649664, 171798691840, 3459890413727383552],
                [80, 608436224, 9431748182016, 364791569817010176],
                [4480, 301989888, 59648505806848, 182395784908505088],
            ],
        })
        .clone()
    }

    #[test]
    fn test_linkage() {
        let deal = initialize_test();
        let contract = ContractMock {};
        let solver = crate::bindings::DoubleDummySolver::solver();
        println!("{}", solver.dd_tricks(&deal, &contract).unwrap());
        println!("{}", crate::bindings::dd_score(&deal, &contract).unwrap());
    }
}
