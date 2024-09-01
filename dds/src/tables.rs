// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use core::ffi::c_int;
use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use crate::{
    bindings::{
        ddsffi::{DDS_HANDS, DDS_STRAINS, DDS_SUITS, MAXNOOFTABLES},
        CalcAllTables, CalcAllTablesPBN, CalcDDtable, CalcDDtablePBN,
    },
    ddserror::DDSError,
    deal::DdsSuit,
    doubledummy::DoubleDummySolver,
    utils::if_no_fault_return,
};

/// Trait that abstracts the ability to solve a doubledummy table, or par table,
/// like the one you see in scores after a tournament.
pub trait DdTableCalculator {
    /// Calculates all the table for a given deal, returning the maximum result for every contract
    /// played by every player. Disregards information about seat, e.g. if both the pairs can make
    /// 1NT, this function will return 1NT as makable for both.
    /// Use [`DdTableCalculator::calculate_all_complete_tables`] for parallel calculation of more
    /// deals, as starting from a couple of dozens deals will get slow using this function.
    /// # Errors
    /// Should error only if the underlying solver errors out.
    fn calculate_complete_table<T>(
        &self,
        table_deal: &T,
    ) -> Result<DdTableResults<Populated>, DDSError>
    where
        for<'a> &'a T: Into<DdTableDeal>;

    /// Calculates all the table for a given deal in PBN format (or a deal which can be converted
    /// in PBN format), returning the maximum result for every contract played by every player.
    /// Disregards information about seat, e.g. if both the pairs can make 1NT, this function will
    /// return 1NT as makable for both.
    /// Use [`DdTableCalculator::calculate_all_complete_tables_pbn`] for parallel calculation of more
    /// deals, as starting from a couple of dozens deals will get slow using this function.
    /// # Errors
    /// Should error only if the underlying solver errors out.
    fn calculate_complete_table_pbn<P>(
        &self,
        table_deal_pbn: &P,
    ) -> Result<DdTableResults<Populated>, DDSError>
    where
        for<'a> &'a P: Into<DdTableDealPbn>;
    /// Same as [`DdTableCalculator::calculate_complete_table`], but parallel: use this function if
    /// you have more than a dozen deals.
    /// Calculates all the table for a given deal, returning the maximum result for every contract
    /// played by every player. Disregards information about seat, e.g. if both the pairs can make
    /// 1NT, this function will return 1NT as makable for both.
    /// # Errors
    /// Should error only if the underlying solver errors out.
    fn calculate_all_complete_tables<T>(
        &self,
        table_deals: &[T],
        mode: ParCalcMode,
        trump_filter: TrumpFilter,
    ) -> Result<DdTablesRes<Populated>, DDSError>
    where
        for<'a> &'a T: Into<DdTableDeal>;
    /// Same as [`DdTableCalculator::calculate_complete_table_pbn`], but parallel: use this
    /// function if you have more than a couple of dozens deals.
    /// Calculates all the table for a given deal in PBN format (or a deal which can be converted
    /// in PBN format), returning the maximum result for every contract played by every player.
    /// Disregards information about seat, e.g. if both the pairs can make 1NT, this function will
    /// return 1NT as makable for both.
    /// # Errors
    /// Should error only if the underlying solver errors out.
    fn calculate_all_complete_tables_pbn<P>(
        &self,
        table_deals_pbn: &[P],
        mode: ParCalcMode,
        trump_filter: TrumpFilter,
    ) -> Result<DdTablesRes<Populated>, DDSError>
    where
        for<'a> &'a P: Into<DdTableDealPbn>;
}

impl DdTableCalculator for DoubleDummySolver {
    fn calculate_complete_table<T>(
        &self,
        table_deal: &T,
    ) -> Result<DdTableResults<Populated>, DDSError>
    where
        for<'a> &'a T: Into<DdTableDeal>,
    {
        let mut tablep = DdTableResults::new();
        let result = unsafe {
            CalcDDtable(
                table_deal.into(),
                std::ptr::from_mut::<DdTableResults<NotPopulated>>(&mut tablep),
            )
        };
        if_no_fault_return!(result, tablep.populated());
    }
    fn calculate_complete_table_pbn<P>(
        &self,
        table_deal_pbn: &P,
    ) -> Result<DdTableResults<Populated>, DDSError>
    where
        for<'a> &'a P: Into<DdTableDealPbn>,
    {
        let mut tablep = DdTableResults::new();
        let result = unsafe {
            CalcDDtablePBN(
                table_deal_pbn.into(),
                std::ptr::from_mut::<DdTableResults<NotPopulated>>(&mut tablep),
            )
        };
        if_no_fault_return!(result, tablep.populated());
    }
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    fn calculate_all_complete_tables<T>(
        &self,
        table_deals: &[T],
        mode: ParCalcMode,
        mut trump_filter: TrumpFilter,
    ) -> Result<DdTablesRes<Populated>, DDSError>
    where
        for<'a> &'a T: Into<DdTableDeal>,
    {
        let mut resp = DdTablesRes::new(table_deals.len() as i32);

        let mut dealsp = DdTableDeals::new(table_deals);
        let mut presp = AllParResults::new();

        let result = unsafe {
            CalcAllTables(
                std::ptr::from_mut::<DdTableDeals>(&mut dealsp),
                mode as i32,
                trump_filter.as_mut_ptr(),
                std::ptr::from_mut::<DdTablesRes<NotPopulated>>(&mut resp),
                std::ptr::from_mut::<AllParResults>(&mut presp),
            )
        };

        if_no_fault_return!(result, resp.populated());
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn calculate_all_complete_tables_pbn<P>(
        &self,
        table_deals_pbn: &[P],
        mode: ParCalcMode,
        mut trump_filter: TrumpFilter,
    ) -> Result<DdTablesRes<Populated>, DDSError>
    where
        for<'a> &'a P: Into<DdTableDealPbn>,
    {
        let mut resp = DdTablesRes::new(table_deals_pbn.len() as i32);

        let mut presp = AllParResults::new();
        let mut dealsp = DdTableDealsPbn::new(table_deals_pbn);
        let result = unsafe {
            CalcAllTablesPBN(
                std::ptr::from_mut::<DdTableDealsPbn>(&mut dealsp),
                mode as i32,
                trump_filter.as_mut_ptr(),
                std::ptr::from_mut::<DdTablesRes<NotPopulated>>(&mut resp),
                std::ptr::from_mut::<AllParResults>(&mut presp),
            )
        };
        if_no_fault_return!(result, resp.populated());
    }
}

#[repr(i32)]
/// Par Calculation Mode, gives info on the vulnerability.
/// See DDS docs for informations.
pub enum ParCalcMode {
    NoPar = -1,
    None = 0,
    Both = 1,
    Ns = 2,
    Ew = 3,
}

#[repr(i32)]
/// How DDS encodes vulnerability.
pub enum VulnerabilityEncoding {
    None = 0,
    Both = 1,
    Ns = 2,
    Ew = 3,
}

/// Filter which decides which strain should we analyze.
/// The order of the ints is based on [`DdsSuitEncoding`] encoding.
/// 0 mean we DO NOT FILTER the suit out, other mean we filter.
/// So if the filter is `[0, 0, -1, 2, 3]` we'll be analyzing
/// [`DdsSuitEncoding::Spades`] and [`DdsSuitEncoding::Hearts`].
pub type TrumpFilter = [c_int; 5];

impl Index<DdsSuit> for TrumpFilter {
    type Output = c_int;
    #[inline]
    fn index(&self, index: DdsSuit) -> &Self::Output {
        self.index(index as usize)
    }
}

impl IndexMut<DdsSuit> for TrumpFilter {
    #[inline]
    fn index_mut(&mut self, index: DdsSuit) -> &mut Self::Output {
        self.index_mut(index as usize)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// This struct contains the distribution of the cards, with a particular encoding
/// First index is encoded in [`crate::deal::DdsHandEncoding`], second index is encoded in
/// [`DdsSuitEncoding`]. The way we store the fields is a bit set of the rank the hand holds in a
/// particular suit so, if North has AKQ of Spades, then:
///
/// ```
/// use dds::tables::DdTableDeal;
/// let mut table = DdTableDeal::new();
/// table[0][0] = 0b011100000000000
///               // SA|SK|SQ
/// ```
pub struct DdTableDeal {
    pub cards: [[::std::os::raw::c_uint; DDS_SUITS as usize]; DDS_HANDS as usize],
}

impl Index<usize> for DdTableDeal {
    type Output = [std::os::raw::c_uint; DDS_SUITS as usize];
    fn index(&self, index: usize) -> &Self::Output {
        self.cards.index(index)
    }
}
impl IndexMut<usize> for DdTableDeal {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.cards.index_mut(index)
    }
}

impl Index<DdsSuit> for DdTableDeal {
    type Output = [std::os::raw::c_uint; DDS_SUITS as usize];
    fn index(&self, index: DdsSuit) -> &Self::Output {
        self.cards.index(index as usize)
    }
}
impl IndexMut<DdsSuit> for DdTableDeal {
    fn index_mut(&mut self, index: DdsSuit) -> &mut Self::Output {
        self.cards.index_mut(index as usize)
    }
}

impl DdTableDeal {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self { cards: [[0; 4]; 4] }
    }
}

impl Default for DdTableDeal {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// A collection of [`DdTableDeal`]s, contained in a fixed array of 200 elements.
/// We can provide less, since we keep a counter with the number of deals loaded.
pub struct DdTableDeals {
    pub no_of_tables: ::std::os::raw::c_int,
    pub deals: [DdTableDeal; (MAXNOOFTABLES * DDS_STRAINS) as usize],
}

impl DdTableDeals {
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new<T>(deals: &[T]) -> Self
    where
        for<'a> &'a T: Into<DdTableDeal>,
    {
        let mut deals_vec: Vec<DdTableDeal> = deals
            .iter()
            .take(200)
            .map(std::convert::Into::into)
            .collect();
        let len = deals_vec.len();

        deals_vec.resize(200, DdTableDeal::new());

        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        Self {
            no_of_tables: len as i32,
            deals: deals_vec.try_into().unwrap(),
        }
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// A bridge deal represented as an array of chars.
/// Pbn are basically strings.
pub struct DdTableDealPbn {
    pub cards: [::std::os::raw::c_char; 80usize],
}

impl DdTableDealPbn {
    #[must_use]
    pub fn new() -> Self {
        Self { cards: [56; 80] }
    }
}

impl Default for DdTableDealPbn {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// A collection of [`DdTableDealPbn`]s, contained in a fixed array of 200 elements.
/// We can provide less, since we keep a counter with the number of deals loaded.
pub struct DdTableDealsPbn {
    pub no_of_tables: ::std::os::raw::c_int,
    pub deals: [DdTableDealPbn; (MAXNOOFTABLES * DDS_STRAINS) as usize],
}

impl DdTableDealsPbn {
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new<T>(deals: &[T]) -> Self
    where
        for<'a> &'a T: Into<DdTableDealPbn>,
    {
        let mut deals_vec: Vec<DdTableDealPbn> = deals
            .iter()
            .take(200)
            .map(std::convert::Into::into)
            .collect();
        let len = deals_vec.len();

        deals_vec.resize(200, DdTableDealPbn::new());

        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        Self {
            no_of_tables: len as i32,
            deals: deals_vec.try_into().unwrap(),
        }
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// A struct that keeps the result of the double dummy result for a deal, basically saving the
/// result for every suit for every player. The generic you see is for keeping track of the state
/// of the table.
pub struct DdTableResults<T: TablePopulated> {
    pub res_table: [[::std::os::raw::c_int; DDS_HANDS as usize]; DDS_STRAINS as usize],
    state: PhantomData<T>,
}

impl DdTableResults<NotPopulated> {
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            res_table: [[0; 4]; 5],
            state: PhantomData,
        }
    }

    #[must_use]
    const fn populated(self) -> DdTableResults<Populated> {
        DdTableResults {
            res_table: self.res_table,
            state: PhantomData,
        }
    }
}

impl Default for DdTableResults<NotPopulated> {
    fn default() -> Self {
        Self::new()
    }
}
pub trait TablePopulated: populated_private::SealedPopulated {}
mod populated_private {
    pub trait SealedPopulated {}
}

#[derive(Debug, Copy, Clone)]
/// Marker struct for representing a not yet populated [`DdTableResults`]
/// This is used as a [`PhantomData`] for marking.
pub struct NotPopulated;
/// Marker struct for representing a populated [`DdTableResults`].
/// This is used as a [`PhantomData`] for marking.
#[derive(Debug, Copy, Clone)]
pub struct Populated;

impl TablePopulated for NotPopulated {}
impl TablePopulated for Populated {}
impl populated_private::SealedPopulated for NotPopulated {}
impl populated_private::SealedPopulated for Populated {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// A collection of [`DdTableResults`] for multiple deals.
pub struct DdTablesRes<T: TablePopulated> {
    pub no_of_boards: ::std::os::raw::c_int,
    pub results: [DdTableResults<T>; (MAXNOOFTABLES * DDS_STRAINS) as usize],
}

impl DdTablesRes<NotPopulated> {
    #[must_use]
    pub(crate) const fn new(no_of_boards: i32) -> Self {
        Self {
            no_of_boards,
            results: [DdTableResults::new(); 200],
        }
    }

    #[must_use]
    const fn populated(self) -> DdTablesRes<Populated> {
        unsafe { std::mem::transmute::<Self, DdTablesRes<Populated>>(self) }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// This is a struct used by DDS to represent basically the string of the par score.
pub struct ParResults {
    pub par_score: [[::std::os::raw::c_char; 16usize]; 2usize],
    pub par_contracts_string: [[::std::os::raw::c_char; 128usize]; 2usize],
}

impl ParResults {
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            par_score: [[20; 16]; 2],
            par_contracts_string: [[20; 128]; 2],
        }
    }
}

impl Default for ParResults {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// A DDS struct containing multile [`ParResults`].
pub struct AllParResults {
    pub par_results: [ParResults; MAXNOOFTABLES as usize],
}

impl AllParResults {
    #[must_use]
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            par_results: [ParResults::new(); MAXNOOFTABLES as usize],
        }
    }
}

impl Default for AllParResults {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(deref_nullptr, non_snake_case, clippy::ref_as_ptr)]
mod test {
    use super::*;
    #[test]
    fn bindgen_test_layout_ddTableDeal() {
        assert_eq!(
            ::std::mem::size_of::<DdTableDeal>(),
            64usize,
            concat!("Size of: ", stringify!(ddTableDeal))
        );
        assert_eq!(
            ::std::mem::align_of::<DdTableDeal>(),
            4usize,
            concat!("Alignment of ", stringify!(ddTableDeal))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdTableDeal>())).cards as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ddTableDeal),
                "::",
                stringify!(cards)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_ddTableDeals() {
        assert_eq!(
            ::std::mem::size_of::<DdTableDeals>(),
            12804usize,
            concat!("Size of: ", stringify!(ddTableDeals))
        );
        assert_eq!(
            ::std::mem::align_of::<DdTableDeals>(),
            4usize,
            concat!("Alignment of ", stringify!(ddTableDeals))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdTableDeals>())).no_of_tables as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ddTableDeals),
                "::",
                stringify!(noOfTables)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdTableDeals>())).deals as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ddTableDeals),
                "::",
                stringify!(deals)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_ddTableDealPBN() {
        assert_eq!(
            ::std::mem::size_of::<DdTableDealPbn>(),
            80usize,
            concat!("Size of: ", stringify!(ddTableDealPBN))
        );
        assert_eq!(
            ::std::mem::align_of::<DdTableDealPbn>(),
            1usize,
            concat!("Alignment of ", stringify!(ddTableDealPBN))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdTableDealPbn>())).cards as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ddTableDealPBN),
                "::",
                stringify!(cards)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_ddTableDealsPBN() {
        assert_eq!(
            ::std::mem::size_of::<DdTableDealsPbn>(),
            16004usize,
            concat!("Size of: ", stringify!(ddTableDealsPBN))
        );
        assert_eq!(
            ::std::mem::align_of::<DdTableDealsPbn>(),
            4usize,
            concat!("Alignment of ", stringify!(ddTableDealsPBN))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<DdTableDealsPbn>())).no_of_tables as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ddTableDealsPBN),
                "::",
                stringify!(noOfTables)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DdTableDealsPbn>())).deals as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ddTableDealsPBN),
                "::",
                stringify!(deals)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_ddTableResults() {
        assert_eq!(
            ::std::mem::size_of::<DdTableResults<NotPopulated>>(),
            80usize,
            concat!("Size of: ", stringify!(ddTableResults))
        );
        assert_eq!(
            ::std::mem::align_of::<DdTableResults<NotPopulated>>(),
            4usize,
            concat!("Alignment of ", stringify!(ddTableResults))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<DdTableResults<NotPopulated>>())).res_table as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ddTableResults),
                "::",
                stringify!(resTable)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_ddTablesRes() {
        assert_eq!(
            ::std::mem::size_of::<DdTablesRes<NotPopulated>>(),
            16004usize,
            concat!("Size of: ", stringify!(ddTablesRes))
        );
        assert_eq!(
            ::std::mem::align_of::<DdTablesRes<NotPopulated>>(),
            4usize,
            concat!("Alignment of ", stringify!(ddTablesRes))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<DdTablesRes<NotPopulated>>())).no_of_boards as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ddTablesRes),
                "::",
                stringify!(noOfBoards)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<DdTablesRes<NotPopulated>>())).results as *const _ as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ddTablesRes),
                "::",
                stringify!(results)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_allParResults() {
        assert_eq!(
            ::std::mem::size_of::<AllParResults>(),
            11520usize,
            concat!("Size of: ", stringify!(allParResults))
        );
        assert_eq!(
            ::std::mem::align_of::<AllParResults>(),
            1usize,
            concat!("Alignment of ", stringify!(allParResults))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<AllParResults>())).par_results as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(allParResults),
                "::",
                stringify!(presults)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_parResults() {
        assert_eq!(
            ::std::mem::size_of::<ParResults>(),
            288usize,
            concat!("Size of: ", stringify!(parResults))
        );
        assert_eq!(
            ::std::mem::align_of::<ParResults>(),
            1usize,
            concat!("Alignment of ", stringify!(parResults))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ParResults>())).par_score as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(parResults),
                "::",
                stringify!(parScore)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<ParResults>())).par_contracts_string as *const _ as usize
            },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(parResults),
                "::",
                stringify!(parContractsString)
            )
        );
    }

    impl From<&[[u32; 4]; 4]> for DdTableDeal {
        fn from(val: &[[u32; 4]; 4]) -> Self {
            DdTableDeal { cards: *val }
        }
    }

    const HOLDINGS: [[[u32; 4]; 4]; 3] = [
        [
            // Spades
            [
                // North
                1 << 12 | 1 << 11 | 1 << 6,
                // East
                1 << 8 | 1 << 7 | 1 << 3,
                // South
                1 << 13 | 1 << 5,
                // South
                1 << 14 | 1 << 10 | 1 << 9 | 1 << 4 | 1 << 2,
            ],
            // Hearts
            [
                1 << 13 | 1 << 6 | 1 << 5 | 1 << 2,
                1 << 11 | 1 << 9 | 1 << 7,
                1 << 10 | 1 << 8 | 1 << 3,
                1 << 14 | 1 << 12 | 1 << 4,
            ],
            // Diamonds
            [
                1 << 11 | 1 << 8 | 1 << 5,
                1 << 14 | 1 << 10 | 1 << 7 | 1 << 6 | 1 << 4,
                1 << 13 | 1 << 12 | 1 << 9,
                1 << 3 | 1 << 2,
            ],
            // Clubs
            [
                1 << 10 | 1 << 9 | 1 << 8,
                1 << 12 | 1 << 4,
                1 << 14 | 1 << 7 | 1 << 6 | 1 << 5 | 1 << 2,
                1 << 13 | 1 << 11 | 1 << 3,
            ],
        ],
        [
            [
                1 << 14 | 1 << 13 | 1 << 9 | 1 << 6,
                1 << 12 | 1 << 11 | 1 << 10 | 1 << 5 | 1 << 4 | 1 << 3 | 1 << 2,
                0,
                1 << 8 | 1 << 7,
            ],
            [
                1 << 13 | 1 << 12 | 1 << 8,
                1 << 10,
                1 << 11 | 1 << 9 | 1 << 7 | 1 << 5 | 1 << 4 | 1 << 3,
                1 << 14 | 1 << 6 | 1 << 2,
            ],
            [
                1 << 14 | 1 << 9 | 1 << 8,
                1 << 6,
                1 << 13 | 1 << 7 | 1 << 5 | 1 << 3 | 1 << 2,
                1 << 12 | 1 << 11 | 1 << 10 | 1 << 4,
            ],
            [
                1 << 13 | 1 << 6 | 1 << 3,
                1 << 12 | 1 << 11 | 1 << 8 | 1 << 2,
                1 << 9 | 1 << 4,
                1 << 14 | 1 << 10 | 1 << 7 | 1 << 5,
            ],
        ],
        [
            [
                1 << 7 | 1 << 3,
                1 << 12 | 1 << 10 | 1 << 6,
                1 << 5,
                1 << 14 | 1 << 13 | 1 << 11 | 1 << 9 | 1 << 8 | 1 << 4 | 1 << 2,
            ],
            [
                1 << 12 | 1 << 11 | 1 << 10,
                1 << 8 | 1 << 7 | 1 << 6,
                1 << 14 | 1 << 9 | 1 << 5 | 1 << 4 | 1 << 3 | 1 << 2,
                1 << 13,
            ],
            [
                1 << 14 | 1 << 12 | 1 << 5 | 1 << 4,
                1 << 13 | 1 << 11 | 1 << 9,
                1 << 7 | 1 << 6 | 1 << 3 | 1 << 2,
                1 << 10 | 1 << 8,
            ],
            [
                1 << 10 | 1 << 7 | 1 << 5 | 1 << 2,
                1 << 14 | 1 << 12 | 1 << 8 | 1 << 4,
                1 << 13 | 1 << 6,
                1 << 11 | 1 << 9 | 1 << 3,
            ],
        ],
    ];

    const DDTABLE: [[i32; 20]; 3] = [
        [5, 8, 5, 8, 6, 6, 6, 6, 5, 7, 5, 7, 7, 5, 7, 5, 6, 6, 6, 6],
        [4, 9, 4, 9, 10, 2, 10, 2, 8, 3, 8, 3, 6, 7, 6, 7, 9, 3, 9, 3],
        [3, 10, 3, 10, 9, 4, 9, 4, 8, 4, 8, 4, 3, 9, 3, 9, 4, 8, 4, 8],
    ];

    fn check_table(table: &DdTableResults<Populated>, hand_no: usize) {
        for strain in 0..5 {
            for player in 0..4 {
                assert_eq!(
                    table.res_table[strain][player],
                    DDTABLE[hand_no][4 * strain + player]
                );
            }
        }
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_calculate_table_unprotected_worrs() {
        // Remember to run all this test in one thread, otherwise they'll SEGFAULT
        let mut table_deal = [[0; 4]; 4];
        let solver = DoubleDummySolver::new();
        for deal in 0..3 {
            for h in 0..4 {
                for s in 0..4 {
                    table_deal[h][s] = HOLDINGS[deal][s][h];
                }
            }
            let table = solver.calculate_complete_table(&table_deal).unwrap();
            check_table(&table, deal);
        }
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_CalcDDTable_unprotected_works() {
        // Remember to run all this test in one thread, otherwise they'll SEGFAULT
        let mut table_deal = DdTableDeal::new();
        for deal in 0..3 {
            for h in 0..4 {
                for s in 0..4 {
                    table_deal.cards[h][s] = HOLDINGS[deal][s][h];
                }
            }
            let mut table = DdTableResults::new();
            let result =
                unsafe { CalcDDtable(table_deal, &mut table as *mut DdTableResults<NotPopulated>) };
            let table = unsafe {
                std::mem::transmute::<DdTableResults<NotPopulated>, DdTableResults<Populated>>(
                    table,
                )
            };
            assert_eq!(crate::bindings::ddsffi::RETURN_NO_FAULT, result);
            check_table(&table, deal);
        }
    }

    #[test]
    fn test_calculate_all_table_unprotected() {
        // Remember to run all this test in one thread, otherwise they'll SEGFAULT
        let mut table_deal = [[[0; 4]; 4]; 3];

        for deal in 0..3 {
            for h in 0..4 {
                for s in 0..4 {
                    table_deal[deal][h][s] = HOLDINGS[deal][s][h];
                }
            }
        }
        let mut table_deal = DdTableDeals::new(&table_deal);
        let mut table = DdTablesRes::new(3);
        let mut par_results = AllParResults::new();
        let result = unsafe {
            CalcAllTables(
                (&mut table_deal) as *mut DdTableDeals,
                ParCalcMode::None as i32,
                &mut [0, 0, 0, 0, 0] as *mut i32,
                &mut table as *mut DdTablesRes<NotPopulated>,
                &mut par_results as *mut AllParResults,
            )
        };
        let table = unsafe {
            std::mem::transmute::<DdTablesRes<NotPopulated>, DdTablesRes<Populated>>(table)
        };
        assert_eq!(crate::bindings::ddsffi::RETURN_NO_FAULT, result);
        check_table(&table.results[0], 0);
        check_table(&table.results[1], 1);
        check_table(&table.results[2], 2);
    }
}
