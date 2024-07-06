use std::marker::PhantomData;

use crate::*;

pub trait DdTableCalculator {
    /// Function to calculate a double dummy table for the given deal
    /// We start specific but the aim is to generalize tha interface
    fn calculate_complete_table(
        &self,
        table_deal: DdTableDeal,
        tablep: &mut DdTableResults<NotPopulated>,
    ) -> Result<&mut DdTableResults<Populated>, DDSError>;
    fn calculate_complete_table_pbn(
        &self,
        table_deal_pbn: DdTableDealPbn,
        tablep: &mut DdTableResults<NotPopulated>,
    ) -> Result<&mut DdTableResults<Populated>, DDSError>;
    fn calculate_all_complete_tables(
        &self,
        table_deals: DdTableDeals,
        vulnerability: VulnerabilityEnc,
        trump_filter: TrumpFilter,
        resp: &mut DdTablesRes<NotPopulated>,
        presp: &mut AllParResults,
    ) -> Result<&mut DdTablesRes<Populated>, DDSError>;
    fn calculate_all_complete_tables_pbn(
        &self,
        table_deals_pbn: DdTableDealsPbn,
        vulnerability: VulnerabilityEnc,
        trump_filter: TrumpFilter,
        resp: &mut DdTablesRes<NotPopulated>,
        presp: &mut AllParResults,
    ) -> Result<&mut DdTablesRes<Populated>, DDSError>;
}

pub enum VulnerabilityEnc {
    NoPar = -1,
    None = 0,
    Both = 1,
    NS = 2,
    EW = 3,
}

pub type TrumpFilter = [c_int; 5];

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DdTableDeal {
    pub cards: [[::std::os::raw::c_uint; 4usize]; 4usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DdTableDeals {
    pub noOfTables: ::std::os::raw::c_int,
    pub deals: [DdTableDeal; 200usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DdTableDealPbn {
    pub cards: [::std::os::raw::c_char; 80usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DdTableDealsPbn {
    pub noOfTables: ::std::os::raw::c_int,
    pub deals: [DdTableDealPbn; 200usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DdTableResults<T: TablePopulated> {
    pub resTable: [[::std::os::raw::c_int; 4usize]; 5usize],
    state: PhantomData<T>,
}
pub trait TablePopulated: populated_private::SealedPopulated {}
mod populated_private {
    pub trait SealedPopulated {}
}

pub struct NotPopulated;
pub struct Populated;

impl TablePopulated for NotPopulated {}
impl TablePopulated for Populated {}
impl populated_private::SealedPopulated for NotPopulated {}
impl populated_private::SealedPopulated for Populated {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DdTablesRes<T: TablePopulated> {
    pub noOfBoards: ::std::os::raw::c_int,
    pub results: [DdTableResults<T>; 200usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ParResults {
    pub parScore: [[::std::os::raw::c_char; 16usize]; 2usize],
    pub parContractsString: [[::std::os::raw::c_char; 128usize]; 2usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AllParResults {
    pub presults: [ParResults; 40usize],
}

#[cfg(test)]
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
            unsafe { &(*(::std::ptr::null::<DdTableDeals>())).noOfTables as *const _ as usize },
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
            unsafe { &(*(::std::ptr::null::<DdTableDealsPbn>())).noOfTables as *const _ as usize },
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
                &(*(::std::ptr::null::<DdTableResults<NotPopulated>>())).resTable as *const _
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
                &(*(::std::ptr::null::<DdTablesRes<NotPopulated>>())).noOfBoards as *const _
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
            unsafe { &(*(::std::ptr::null::<AllParResults>())).presults as *const _ as usize },
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
            unsafe { &(*(::std::ptr::null::<ParResults>())).parScore as *const _ as usize },
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
                &(*(::std::ptr::null::<ParResults>())).parContractsString as *const _ as usize
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
}
