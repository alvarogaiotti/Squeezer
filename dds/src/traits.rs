// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::deal::{DdsHandEncoding, DdsRank, DdsSuit};

pub trait IntoRawDDS {
    type Raw;

    fn into_raw(self) -> Self::Raw;
}

pub trait RawDDSRef<'a> {
    type Raw;

    fn get_raw(&'a self) -> Self::Raw;
}

pub trait RawDDSRefMut<'a> {
    type RawMut;

    fn get_raw_mut(&'a mut self) -> Self::RawMut;
}

pub trait AsDDSContract {
    /// This function returns (strain, leader) for the contract
    fn as_dds_contract(&self) -> (DdsSuit, DdsHandEncoding);
}

pub trait ContractScorer {
    fn score(&self, tricks: u8) -> i32;
}

pub trait AsDDSCard {
    fn as_card(&self) -> (DdsRank, DdsSuit);
}

pub trait AsDDSPlayTrace<I, C>
where
    I: IntoIterator,
    I::Item: core::borrow::Borrow<C>,
    C: AsDDSCard,
{
    fn as_play_trace(&self) -> I;
}
