// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

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
    I::Item: core::borrow::Borrow<C>,
    C: AsDDSCard,
{
    fn as_play_trace(&self) -> I;
}
