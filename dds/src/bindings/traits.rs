use crate::{DDSHandEncoding, DDSSuitEncoding};

pub trait RawDDS {
    type Raw;

    fn get_raw(&self) -> Self::Raw;
}

/// Models a side: either North-South or East-West

pub trait AsDDSContract {
    fn as_dds_contract(&self) -> (i32, i32);
}

pub trait ContractScorer {
    fn score(&self, tricks: u8) -> i32;
}

pub trait AsDDSCard {
    fn as_card(&self) -> (i32, i32);
}

pub trait AsDDSPlayTrace<I, C>
where
    I: IntoIterator,
    I::Item: std::borrow::Borrow<C>,
    C: AsDDSCard,
{
    fn as_play_trace(&self) -> I;
}
