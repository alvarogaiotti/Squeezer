use super::{AsDDSContract, AsDDSDeal, DDSDeal};
use crate::bindings::ddsffi::SolveBoard;
pub trait BridgeSolver {
    #[must_use]
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(&self, deal: &D, contract: &C) -> u8;
}
pub(super) struct DDSSolver {}

impl BridgeSolver for DDSSolver {
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(&self, deal: &D, contract: &C) -> u8 {
        let (trump, first) = contract.as_dds_contract();
        let c_deal = ::deal {
            trump: trump as c_int,
            first: first as c_int,
            currentTrickSuit: [0; 3],
            currentTrickRank: [0; 3],
            remainCards: deal.as_dds_deal(),
        };
        let mut future_tricks = empty_fut();
        let futp: *mut futureTricks = &mut future_tricks;
        unsafe { SolveBoard(c_deal, -1, 1, 1, futp, 0) };
        13 - future_tricks.score[0] as u8
    }
}
