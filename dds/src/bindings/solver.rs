use crate::RawDDS;

use super::{
    ddsffi::SolveBoard,
    deal::{AsDDSDeal, DDSDealBuilder, DDSDealConstructionError},
    future_tricks::FutureTricks,
    AsDDSContract,
};

pub trait BridgeSolver {
    type Error;
    #[must_use]
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, Self::Error>;
}
pub(super) struct DDSSolver {}

impl BridgeSolver for DDSSolver {
    type Error = DDSDealConstructionError;
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, Self::Error> {
        let (trump, first) = contract.as_dds_contract();
        let c_deal = DDSDealBuilder::new()
            .trump(trump.try_into()?)
            .first(first.try_into()?)
            .remain_cards(deal.as_dds_deal())
            .build()?;
        let mut future_tricks = FutureTricks::new();
        let futp = &mut future_tricks.get_raw();
        unsafe { SolveBoard(c_deal.get_raw(), -1, 1, 1, futp, 0) };
        Ok(13 - future_tricks.score()[0] as u8)
    }
}
