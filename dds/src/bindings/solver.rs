use super::{
    ddsffi::SolveBoard,
    deal::{AsDDSDeal, DDSDealBuilder, DDSDealConstructionError},
    future_tricks::FutureTricks,
    AsDDSContract, Mode, Solutions, Target, ThreadIndex,
};
use crate::{DDSError, RawDDS};
use std::num::NonZeroI32;

pub trait BridgeSolver {
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, Box<dyn std::error::Error>>;
}
pub struct DDSSolver {}

impl BridgeSolver for DDSSolver {
    fn dd_tricks<D: AsDDSDeal, C: AsDDSContract>(
        &self,
        deal: &D,
        contract: &C,
    ) -> Result<u8, Box<dyn std::error::Error>> {
        let (trump, first) = contract.as_dds_contract();
        let c_deal = DDSDealBuilder::new()
            .trump(trump.try_into()?)
            .first(first.try_into()?)
            .remain_cards(deal.as_dds_deal())
            .build()?;
        let future_tricks = FutureTricks::new();
        let futp = &mut future_tricks.get_raw();
        let result;
        unsafe {
            result = SolveBoard(
                c_deal.get_raw(),
                Target::MaxTricks.into(),
                Solutions::Best.into(),
                Mode::AutoSearchAlways.into(),
                futp,
                ThreadIndex::Auto.into(),
            )
        };
        if result != 1 {
            return Err(Box::new(DDSError::new(result)));
        }
        Ok(13 - future_tricks.score()[0] as u8)
    }
}
