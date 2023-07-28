use crate::RawDDS;

use super::{
    ddsffi::SolveBoard,
    deal::{AsDDSDeal, DDSDealBuilder, DDSDealConstructionError},
    future_tricks::FutureTricks,
    AsDDSContract,
};

pub trait BridgeSolver<E: std::error::Error> {
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(&self, deal: &D, contract: &C) -> Result<u8, E>;
}
pub struct DDSSolver {}

impl BridgeSolver<DDSDealConstructionError> for DDSSolver {
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, DDSDealConstructionError> {
        let (trump, first) = contract.as_dds_contract();
        let c_deal = DDSDealBuilder::new()
            .trump(trump.try_into()?)
            .first(first.try_into()?)
            .remain_cards(deal.as_dds_deal())
            .build()?;
        let future_tricks = FutureTricks::new();
        let futp = &mut future_tricks.get_raw();
        unsafe { SolveBoard(c_deal.get_raw(), -1, 1, 1, futp, 0) };
        Ok(13 - future_tricks.score()[0] as u8)
    }
}
