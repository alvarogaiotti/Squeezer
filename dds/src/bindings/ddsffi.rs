#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(deref_nullptr)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::unseparated_literal_suffix)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]
#![allow(clippy::restriction)]

pub const R_OK: u32 = 4;
pub const W_OK: u32 = 2;
pub const X_OK: u32 = 1;
pub const F_OK: u32 = 0;
pub const SEEK_SET: u32 = 0;
pub const SEEK_CUR: u32 = 1;
pub const SEEK_END: u32 = 2;
pub const SEEK_DATA: u32 = 3;
pub const SEEK_HOLE: u32 = 4;
pub const L_SET: u32 = 0;
pub const L_INCR: u32 = 1;
pub const L_XTND: u32 = 2;
pub const DDS_VERSION: u32 = 20900;
pub const DDS_HANDS: u32 = 4;
pub const DDS_SUITS: u32 = 4;
pub const DDS_STRAINS: u32 = 5;
pub const MAXNOOFBOARDS: u32 = 200;
pub const MAXNOOFTABLES: u32 = 40;
pub const RETURN_NO_FAULT: u32 = 1;
pub const TEXT_NO_FAULT: &[u8; 8usize] = b"Success\0";
pub const RETURN_UNKNOWN_FAULT: i32 = -1;
pub const TEXT_UNKNOWN_FAULT: &[u8; 14usize] = b"General error\0";
pub const RETURN_ZERO_CARDS: i32 = -2;
pub const TEXT_ZERO_CARDS: &[u8; 11usize] = b"Zero cards\0";
pub const RETURN_TARGET_TOO_HIGH: i32 = -3;
pub const TEXT_TARGET_TOO_HIGH: &[u8; 32usize] = b"Target exceeds number of tricks\0";
pub const RETURN_DUPLICATE_CARDS: i32 = -4;
pub const TEXT_DUPLICATE_CARDS: &[u8; 17usize] = b"Cards duplicated\0";
pub const RETURN_TARGET_WRONG_LO: i32 = -5;
pub const TEXT_TARGET_WRONG_LO: &[u8; 23usize] = b"Target is less than -1\0";
pub const RETURN_TARGET_WRONG_HI: i32 = -7;
pub const TEXT_TARGET_WRONG_HI: &[u8; 25usize] = b"Target is higher than 13\0";
pub const RETURN_SOLNS_WRONG_LO: i32 = -8;
pub const TEXT_SOLNS_WRONG_LO: &[u8; 35usize] = b"Solutions parameter is less than 1\0";
pub const RETURN_SOLNS_WRONG_HI: i32 = -9;
pub const TEXT_SOLNS_WRONG_HI: &[u8; 37usize] = b"Solutions parameter is higher than 3\0";
pub const RETURN_TOO_MANY_CARDS: i32 = -10;
pub const TEXT_TOO_MANY_CARDS: &[u8; 15usize] = b"Too many cards\0";
pub const RETURN_SUIT_OR_RANK: i32 = -12;
pub const TEXT_SUIT_OR_RANK: &[u8; 52usize] =
    b"currentTrickSuit or currentTrickRank has wrong data\0";
pub const RETURN_PLAYED_CARD: i32 = -13;
pub const TEXT_PLAYED_CARD: &[u8; 35usize] = b"Played card also remains in a hand\0";
pub const RETURN_CARD_COUNT: i32 = -14;
pub const TEXT_CARD_COUNT: &[u8; 42usize] = b"Wrong number of remaining cards in a hand\0";
pub const RETURN_THREAD_INDEX: i32 = -15;
pub const TEXT_THREAD_INDEX: &[u8; 33usize] = b"Thread index is not 0 .. maximum\0";
pub const RETURN_MODE_WRONG_LO: i32 = -16;
pub const TEXT_MODE_WRONG_LO: &[u8; 30usize] = b"Mode parameter is less than 0\0";
pub const RETURN_MODE_WRONG_HI: i32 = -17;
pub const TEXT_MODE_WRONG_HI: &[u8; 32usize] = b"Mode parameter is higher than 2\0";
pub const RETURN_TRUMP_WRONG: i32 = -18;
pub const TEXT_TRUMP_WRONG: &[u8; 23usize] = b"Trump is not in 0 .. 4\0";
pub const RETURN_FIRST_WRONG: i32 = -19;
pub const TEXT_FIRST_WRONG: &[u8; 23usize] = b"First is not in 0 .. 2\0";
pub const RETURN_PLAY_FAULT: i32 = -98;
pub const TEXT_PLAY_FAULT: &[u8; 24usize] = b"AnalysePlay input error\0";
pub const RETURN_PBN_FAULT: i32 = -99;
pub const TEXT_PBN_FAULT: &[u8; 17usize] = b"PBN string error\0";
pub const RETURN_TOO_MANY_BOARDS: i32 = -101;
pub const TEXT_TOO_MANY_BOARDS: &[u8; 26usize] = b"Too many boards requested\0";
pub const RETURN_THREAD_CREATE: i32 = -102;
pub const TEXT_THREAD_CREATE: &[u8; 25usize] = b"Could not create threads\0";
pub const RETURN_THREAD_WAIT: i32 = -103;
pub const TEXT_THREAD_WAIT: &[u8; 43usize] = b"Something failed waiting for thread to end\0";
pub const RETURN_THREAD_MISSING: i32 = -104;
pub const TEXT_THREAD_MISSING: &[u8; 35usize] = b"Multi-threading system not present\0";
pub const RETURN_NO_SUIT: i32 = -201;
pub const TEXT_NO_SUIT: &[u8; 42usize] = b"Denomination filter vector has no entries\0";
pub const RETURN_TOO_MANY_TABLES: i32 = -202;
pub const TEXT_TOO_MANY_TABLES: &[u8; 29usize] = b"Too many DD tables requested\0";
pub const RETURN_CHUNK_SIZE: i32 = -301;
pub const TEXT_CHUNK_SIZE: &[u8; 26usize] = b"Chunk size is less than 1\0";
pub const THREADMEM_SMALL_MAX_MB: u32 = 30;
pub const THREADMEM_SMALL_DEF_MB: u32 = 20;
pub const THREADMEM_LARGE_MAX_MB: u32 = 160;
pub const THREADMEM_LARGE_DEF_MB: u32 = 95;
pub const MAXNODE: u32 = 1;
pub const MINNODE: u32 = 0;
pub const SIMILARDEALLIMIT: u32 = 5;
pub const SIMILARMAXWINNODES: u32 = 700_000;
pub const DDS_NOTRUMP: u32 = 4;
pub type size_t = ::std::os::raw::c_ulong;
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct futureTricks {
    pub nodes: ::std::os::raw::c_int,
    pub cards: ::std::os::raw::c_int,
    pub suit: [::std::os::raw::c_int; 13usize],
    pub rank: [::std::os::raw::c_int; 13usize],
    pub equals: [::std::os::raw::c_int; 13usize],
    pub score: [::std::os::raw::c_int; 13usize],
}
#[test]
fn bindgen_test_layout_futureTricks() {
    assert_eq!(
        ::std::mem::size_of::<futureTricks>(),
        216usize,
        concat!("Size of: ", stringify!(futureTricks))
    );
    assert_eq!(
        ::std::mem::align_of::<futureTricks>(),
        4usize,
        concat!("Alignment of ", stringify!(futureTricks))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).nodes as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(nodes)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).cards as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(cards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).suit as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).rank as *const _ as usize },
        60usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).equals as *const _ as usize },
        112usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(equals)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<futureTricks>())).score as *const _ as usize },
        164usize,
        concat!(
            "Offset of field: ",
            stringify!(futureTricks),
            "::",
            stringify!(score)
        )
    );
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct solvedBoards {
    pub noOfBoards: ::std::os::raw::c_int,
    pub solvedBoard: [futureTricks; 200usize],
}
#[test]
fn bindgen_test_layout_solvedBoards() {
    assert_eq!(
        ::std::mem::size_of::<solvedBoards>(),
        43204usize,
        concat!("Size of: ", stringify!(solvedBoards))
    );
    assert_eq!(
        ::std::mem::align_of::<solvedBoards>(),
        4usize,
        concat!("Alignment of ", stringify!(solvedBoards))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedBoards>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedBoards),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedBoards>())).solvedBoard as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedBoards),
            "::",
            stringify!(solvedBoard)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableDeal {
    pub cards: [[::std::os::raw::c_uint; 4usize]; 4usize],
}
#[test]
fn bindgen_test_layout_ddTableDeal() {
    assert_eq!(
        ::std::mem::size_of::<ddTableDeal>(),
        64usize,
        concat!("Size of: ", stringify!(ddTableDeal))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableDeal>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTableDeal))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDeal>())).cards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDeal),
            "::",
            stringify!(cards)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableDeals {
    pub noOfTables: ::std::os::raw::c_int,
    pub deals: [ddTableDeal; 200usize],
}
#[test]
fn bindgen_test_layout_ddTableDeals() {
    assert_eq!(
        ::std::mem::size_of::<ddTableDeals>(),
        12804usize,
        concat!("Size of: ", stringify!(ddTableDeals))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableDeals>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTableDeals))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDeals>())).noOfTables as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDeals),
            "::",
            stringify!(noOfTables)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDeals>())).deals as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDeals),
            "::",
            stringify!(deals)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableDealPBN {
    pub cards: [::std::os::raw::c_char; 80usize],
}
#[test]
fn bindgen_test_layout_ddTableDealPBN() {
    assert_eq!(
        ::std::mem::size_of::<ddTableDealPBN>(),
        80usize,
        concat!("Size of: ", stringify!(ddTableDealPBN))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableDealPBN>(),
        1usize,
        concat!("Alignment of ", stringify!(ddTableDealPBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDealPBN>())).cards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDealPBN),
            "::",
            stringify!(cards)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableDealsPBN {
    pub noOfTables: ::std::os::raw::c_int,
    pub deals: [ddTableDealPBN; 200usize],
}
#[test]
fn bindgen_test_layout_ddTableDealsPBN() {
    assert_eq!(
        ::std::mem::size_of::<ddTableDealsPBN>(),
        16004usize,
        concat!("Size of: ", stringify!(ddTableDealsPBN))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableDealsPBN>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTableDealsPBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDealsPBN>())).noOfTables as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDealsPBN),
            "::",
            stringify!(noOfTables)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableDealsPBN>())).deals as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableDealsPBN),
            "::",
            stringify!(deals)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTableResults {
    pub resTable: [[::std::os::raw::c_int; 4usize]; 5usize],
}
#[test]
fn bindgen_test_layout_ddTableResults() {
    assert_eq!(
        ::std::mem::size_of::<ddTableResults>(),
        80usize,
        concat!("Size of: ", stringify!(ddTableResults))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTableResults>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTableResults))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTableResults>())).resTable as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTableResults),
            "::",
            stringify!(resTable)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ddTablesRes {
    pub noOfBoards: ::std::os::raw::c_int,
    pub results: [ddTableResults; 200usize],
}
#[test]
fn bindgen_test_layout_ddTablesRes() {
    assert_eq!(
        ::std::mem::size_of::<ddTablesRes>(),
        16004usize,
        concat!("Size of: ", stringify!(ddTablesRes))
    );
    assert_eq!(
        ::std::mem::align_of::<ddTablesRes>(),
        4usize,
        concat!("Alignment of ", stringify!(ddTablesRes))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTablesRes>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTablesRes),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<ddTablesRes>())).results as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(ddTablesRes),
            "::",
            stringify!(results)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parResults {
    pub parScore: [[::std::os::raw::c_char; 16usize]; 2usize],
    pub parContractsString: [[::std::os::raw::c_char; 128usize]; 2usize],
}
#[test]
fn bindgen_test_layout_parResults() {
    assert_eq!(
        ::std::mem::size_of::<parResults>(),
        288usize,
        concat!("Size of: ", stringify!(parResults))
    );
    assert_eq!(
        ::std::mem::align_of::<parResults>(),
        1usize,
        concat!("Alignment of ", stringify!(parResults))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResults>())).parScore as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(parResults),
            "::",
            stringify!(parScore)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResults>())).parContractsString as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(parResults),
            "::",
            stringify!(parContractsString)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct allParResults {
    pub presults: [parResults; 40usize],
}
#[test]
fn bindgen_test_layout_allParResults() {
    assert_eq!(
        ::std::mem::size_of::<allParResults>(),
        11520usize,
        concat!("Size of: ", stringify!(allParResults))
    );
    assert_eq!(
        ::std::mem::align_of::<allParResults>(),
        1usize,
        concat!("Alignment of ", stringify!(allParResults))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<allParResults>())).presults as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(allParResults),
            "::",
            stringify!(presults)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parResultsDealer {
    pub number: ::std::os::raw::c_int,
    pub score: ::std::os::raw::c_int,
    pub contracts: [[::std::os::raw::c_char; 10usize]; 10usize],
}
#[test]
fn bindgen_test_layout_parResultsDealer() {
    assert_eq!(
        ::std::mem::size_of::<parResultsDealer>(),
        108usize,
        concat!("Size of: ", stringify!(parResultsDealer))
    );
    assert_eq!(
        ::std::mem::align_of::<parResultsDealer>(),
        4usize,
        concat!("Alignment of ", stringify!(parResultsDealer))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsDealer>())).number as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsDealer),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsDealer>())).score as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsDealer),
            "::",
            stringify!(score)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsDealer>())).contracts as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsDealer),
            "::",
            stringify!(contracts)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct contractType {
    pub underTricks: ::std::os::raw::c_int,
    pub overTricks: ::std::os::raw::c_int,
    pub level: ::std::os::raw::c_int,
    pub denom: ::std::os::raw::c_int,
    pub seats: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_contractType() {
    assert_eq!(
        ::std::mem::size_of::<contractType>(),
        20usize,
        concat!("Size of: ", stringify!(contractType))
    );
    assert_eq!(
        ::std::mem::align_of::<contractType>(),
        4usize,
        concat!("Alignment of ", stringify!(contractType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).underTricks as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(underTricks)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).overTricks as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(overTricks)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).level as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(level)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).denom as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(denom)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<contractType>())).seats as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(contractType),
            "::",
            stringify!(seats)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parResultsMaster {
    pub score: ::std::os::raw::c_int,
    pub number: ::std::os::raw::c_int,
    pub contracts: [contractType; 10usize],
}
#[test]
fn bindgen_test_layout_parResultsMaster() {
    assert_eq!(
        ::std::mem::size_of::<parResultsMaster>(),
        208usize,
        concat!("Size of: ", stringify!(parResultsMaster))
    );
    assert_eq!(
        ::std::mem::align_of::<parResultsMaster>(),
        4usize,
        concat!("Alignment of ", stringify!(parResultsMaster))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsMaster>())).score as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsMaster),
            "::",
            stringify!(score)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsMaster>())).number as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsMaster),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parResultsMaster>())).contracts as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(parResultsMaster),
            "::",
            stringify!(contracts)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct parTextResults {
    pub parText: [[::std::os::raw::c_char; 128usize]; 2usize],
    pub equal: bool,
}
#[test]
fn bindgen_test_layout_parTextResults() {
    assert_eq!(
        ::std::mem::size_of::<parTextResults>(),
        257usize,
        concat!("Size of: ", stringify!(parTextResults))
    );
    assert_eq!(
        ::std::mem::align_of::<parTextResults>(),
        1usize,
        concat!("Alignment of ", stringify!(parTextResults))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parTextResults>())).parText as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(parTextResults),
            "::",
            stringify!(parText)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<parTextResults>())).equal as *const _ as usize },
        256usize,
        concat!(
            "Offset of field: ",
            stringify!(parTextResults),
            "::",
            stringify!(equal)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct playTraceBin {
    pub number: ::std::os::raw::c_int,
    pub suit: [::std::os::raw::c_int; 52usize],
    pub rank: [::std::os::raw::c_int; 52usize],
}
#[test]
fn bindgen_test_layout_playTraceBin() {
    assert_eq!(
        ::std::mem::size_of::<playTraceBin>(),
        420usize,
        concat!("Size of: ", stringify!(playTraceBin))
    );
    assert_eq!(
        ::std::mem::align_of::<playTraceBin>(),
        4usize,
        concat!("Alignment of ", stringify!(playTraceBin))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTraceBin>())).number as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(playTraceBin),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTraceBin>())).suit as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(playTraceBin),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTraceBin>())).rank as *const _ as usize },
        212usize,
        concat!(
            "Offset of field: ",
            stringify!(playTraceBin),
            "::",
            stringify!(rank)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct playTracePBN {
    pub number: ::std::os::raw::c_int,
    pub cards: [::std::os::raw::c_char; 106usize],
}
#[test]
fn bindgen_test_layout_playTracePBN() {
    assert_eq!(
        ::std::mem::size_of::<playTracePBN>(),
        112usize,
        concat!("Size of: ", stringify!(playTracePBN))
    );
    assert_eq!(
        ::std::mem::align_of::<playTracePBN>(),
        4usize,
        concat!("Alignment of ", stringify!(playTracePBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracePBN>())).number as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracePBN),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracePBN>())).cards as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracePBN),
            "::",
            stringify!(cards)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct solvedPlay {
    pub number: ::std::os::raw::c_int,
    pub tricks: [::std::os::raw::c_int; 53usize],
}
#[test]
fn bindgen_test_layout_solvedPlay() {
    assert_eq!(
        ::std::mem::size_of::<solvedPlay>(),
        216usize,
        concat!("Size of: ", stringify!(solvedPlay))
    );
    assert_eq!(
        ::std::mem::align_of::<solvedPlay>(),
        4usize,
        concat!("Alignment of ", stringify!(solvedPlay))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedPlay>())).number as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedPlay),
            "::",
            stringify!(number)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedPlay>())).tricks as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedPlay),
            "::",
            stringify!(tricks)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct playTracesBin {
    pub noOfBoards: ::std::os::raw::c_int,
    pub plays: [playTraceBin; 200usize],
}
#[test]
fn bindgen_test_layout_playTracesBin() {
    assert_eq!(
        ::std::mem::size_of::<playTracesBin>(),
        84004usize,
        concat!("Size of: ", stringify!(playTracesBin))
    );
    assert_eq!(
        ::std::mem::align_of::<playTracesBin>(),
        4usize,
        concat!("Alignment of ", stringify!(playTracesBin))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracesBin>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracesBin),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracesBin>())).plays as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracesBin),
            "::",
            stringify!(plays)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct playTracesPBN {
    pub noOfBoards: ::std::os::raw::c_int,
    pub plays: [playTracePBN; 200usize],
}
#[test]
fn bindgen_test_layout_playTracesPBN() {
    assert_eq!(
        ::std::mem::size_of::<playTracesPBN>(),
        22404usize,
        concat!("Size of: ", stringify!(playTracesPBN))
    );
    assert_eq!(
        ::std::mem::align_of::<playTracesPBN>(),
        4usize,
        concat!("Alignment of ", stringify!(playTracesPBN))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracesPBN>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracesPBN),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<playTracesPBN>())).plays as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(playTracesPBN),
            "::",
            stringify!(plays)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct solvedPlays {
    pub noOfBoards: ::std::os::raw::c_int,
    pub solved: [solvedPlay; 200usize],
}
#[test]
fn bindgen_test_layout_solvedPlays() {
    assert_eq!(
        ::std::mem::size_of::<solvedPlays>(),
        43204usize,
        concat!("Size of: ", stringify!(solvedPlays))
    );
    assert_eq!(
        ::std::mem::align_of::<solvedPlays>(),
        4usize,
        concat!("Alignment of ", stringify!(solvedPlays))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedPlays>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedPlays),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<solvedPlays>())).solved as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(solvedPlays),
            "::",
            stringify!(solved)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DDSInfo {
    pub major: ::std::os::raw::c_int,
    pub minor: ::std::os::raw::c_int,
    pub patch: ::std::os::raw::c_int,
    pub versionString: [::std::os::raw::c_char; 10usize],
    pub system: ::std::os::raw::c_int,
    pub numBits: ::std::os::raw::c_int,
    pub compiler: ::std::os::raw::c_int,
    pub constructor: ::std::os::raw::c_int,
    pub numCores: ::std::os::raw::c_int,
    pub threading: ::std::os::raw::c_int,
    pub noOfThreads: ::std::os::raw::c_int,
    pub threadSizes: [::std::os::raw::c_char; 128usize],
    pub systemString: [::std::os::raw::c_char; 1024usize],
}
#[test]
fn bindgen_test_layout_DDSInfo() {
    assert_eq!(
        ::std::mem::size_of::<DDSInfo>(),
        1204usize,
        concat!("Size of: ", stringify!(DDSInfo))
    );
    assert_eq!(
        ::std::mem::align_of::<DDSInfo>(),
        4usize,
        concat!("Alignment of ", stringify!(DDSInfo))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).major as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(major)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).minor as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(minor)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).patch as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(patch)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).versionString as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(versionString)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).system as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(system)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).numBits as *const _ as usize },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(numBits)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).compiler as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(compiler)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).constructor as *const _ as usize },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(constructor)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).numCores as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(numCores)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).threading as *const _ as usize },
        44usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(threading)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).noOfThreads as *const _ as usize },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(noOfThreads)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).threadSizes as *const _ as usize },
        52usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(threadSizes)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<DDSInfo>())).systemString as *const _ as usize },
        180usize,
        concat!(
            "Offset of field: ",
            stringify!(DDSInfo),
            "::",
            stringify!(systemString)
        )
    );
}
extern "C" {
    pub fn SetMaxThreads(userThreads: ::std::os::raw::c_int);
}
extern "C" {
    pub fn SetThreading(code: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SetResources(maxMemoryMB: ::std::os::raw::c_int, maxThreads: ::std::os::raw::c_int);
}
extern "C" {
    pub fn FreeMemory();
}
extern "C" {
    pub fn SolveBoard(
        dl: deal,
        target: ::std::os::raw::c_int,
        solutions: ::std::os::raw::c_int,
        mode: ::std::os::raw::c_int,
        futp: *mut futureTricks,
        threadIndex: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveBoardPBN(
        dlpbn: dealPBN,
        target: ::std::os::raw::c_int,
        solutions: ::std::os::raw::c_int,
        mode: ::std::os::raw::c_int,
        futp: *mut futureTricks,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcDDtable(
        tableDeal: ddTableDeal,
        tablep: *mut ddTableResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcDDtablePBN(
        tableDealPBN: ddTableDealPBN,
        tablep: *mut ddTableResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcAllTables(
        dealsp: *mut ddTableDeals,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut ddTablesRes,
        presp: *mut allParResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcAllTablesPBN(
        dealsp: *mut ddTableDealsPBN,
        mode: ::std::os::raw::c_int,
        trumpFilter: *mut ::std::os::raw::c_int,
        resp: *mut ddTablesRes,
        presp: *mut allParResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllBoards(bop: *mut boardsPBN, solvedp: *mut solvedBoards)
        -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllBoardsBin(bop: *mut boards, solvedp: *mut solvedBoards)
        -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllChunks(
        bop: *mut boardsPBN,
        solvedp: *mut solvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllChunksBin(
        bop: *mut boards,
        solvedp: *mut solvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SolveAllChunksPBN(
        bop: *mut boardsPBN,
        solvedp: *mut solvedBoards,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Par(
        tablep: *mut ddTableResults,
        presp: *mut parResults,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcPar(
        tableDeal: ddTableDeal,
        vulnerable: ::std::os::raw::c_int,
        tablep: *mut ddTableResults,
        presp: *mut parResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn CalcParPBN(
        tableDealPBN: ddTableDealPBN,
        tablep: *mut ddTableResults,
        vulnerable: ::std::os::raw::c_int,
        presp: *mut parResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SidesPar(
        tablep: *mut ddTableResults,
        sidesRes: *mut parResultsDealer,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DealerPar(
        tablep: *mut ddTableResults,
        presp: *mut parResultsDealer,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn DealerParBin(
        tablep: *mut ddTableResults,
        presp: *mut parResultsMaster,
        dealer: ::std::os::raw::c_int,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn SidesParBin(
        tablep: *mut ddTableResults,
        sidesRes: *mut parResultsMaster,
        vulnerable: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ConvertToDealerTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn ConvertToSidesTextFormat(
        pres: *mut parResultsMaster,
        resp: *mut parTextResults,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalysePlayBin(
        dl: deal,
        play: playTraceBin,
        solved: *mut solvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalysePlayPBN(
        dlPBN: dealPBN,
        playPBN: playTracePBN,
        solvedp: *mut solvedPlay,
        thrId: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalyseAllPlaysBin(
        bop: *mut boards,
        plp: *mut playTracesBin,
        solvedp: *mut solvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn AnalyseAllPlaysPBN(
        bopPBN: *mut boardsPBN,
        plpPBN: *mut playTracesPBN,
        solvedp: *mut solvedPlays,
        chunkSize: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn GetDDSInfo(info: *mut DDSInfo);
}
extern "C" {
    pub fn ErrorMessage(code: ::std::os::raw::c_int, line: *mut ::std::os::raw::c_char);
}
extern "C" {
    pub static mut lho: [::std::os::raw::c_int; 4usize];
}
extern "C" {
    pub static mut rho: [::std::os::raw::c_int; 4usize];
}
extern "C" {
    pub static mut partner: [::std::os::raw::c_int; 4usize];
}
extern "C" {
    pub static mut bitMapRank: [::std::os::raw::c_ushort; 16usize];
}
extern "C" {
    pub static mut cardRank: [::std::os::raw::c_uchar; 16usize];
}
extern "C" {
    pub static mut cardSuit: [::std::os::raw::c_uchar; 5usize];
}
extern "C" {
    pub static mut cardHand: [::std::os::raw::c_uchar; 4usize];
}
extern "C" {
    pub static mut highestRank: [::std::os::raw::c_int; 8192usize];
}
extern "C" {
    pub static mut lowestRank: [::std::os::raw::c_int; 8192usize];
}
extern "C" {
    pub static mut counttable: [::std::os::raw::c_int; 8192usize];
}
extern "C" {
    pub static mut relRank: [[::std::os::raw::c_char; 15usize]; 8192usize];
}
extern "C" {
    pub static mut winRanks: [[::std::os::raw::c_ushort; 14usize]; 8192usize];
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct moveGroupType {
    pub lastGroup: ::std::os::raw::c_int,
    pub rank: [::std::os::raw::c_int; 7usize],
    pub sequence: [::std::os::raw::c_int; 7usize],
    pub fullseq: [::std::os::raw::c_int; 7usize],
    pub gap: [::std::os::raw::c_int; 7usize],
}
#[test]
fn bindgen_test_layout_moveGroupType() {
    assert_eq!(
        ::std::mem::size_of::<moveGroupType>(),
        116usize,
        concat!("Size of: ", stringify!(moveGroupType))
    );
    assert_eq!(
        ::std::mem::align_of::<moveGroupType>(),
        4usize,
        concat!("Alignment of ", stringify!(moveGroupType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).lastGroup as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(lastGroup)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).rank as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).sequence as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(sequence)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).fullseq as *const _ as usize },
        60usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(fullseq)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveGroupType>())).gap as *const _ as usize },
        88usize,
        concat!(
            "Offset of field: ",
            stringify!(moveGroupType),
            "::",
            stringify!(gap)
        )
    );
}
extern "C" {
    pub static mut groupData: [moveGroupType; 8192usize];
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct moveType {
    pub suit: ::std::os::raw::c_int,
    pub rank: ::std::os::raw::c_int,
    pub sequence: ::std::os::raw::c_int,
    pub weight: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_moveType() {
    assert_eq!(
        ::std::mem::size_of::<moveType>(),
        16usize,
        concat!("Size of: ", stringify!(moveType))
    );
    assert_eq!(
        ::std::mem::align_of::<moveType>(),
        4usize,
        concat!("Alignment of ", stringify!(moveType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveType>())).suit as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(moveType),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveType>())).rank as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(moveType),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveType>())).sequence as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(moveType),
            "::",
            stringify!(sequence)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<moveType>())).weight as *const _ as usize },
        12usize,
        concat!(
            "Offset of field: ",
            stringify!(moveType),
            "::",
            stringify!(weight)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct movePlyType {
    pub move_: [moveType; 14usize],
    pub current: ::std::os::raw::c_int,
    pub last: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_movePlyType() {
    assert_eq!(
        ::std::mem::size_of::<movePlyType>(),
        232usize,
        concat!("Size of: ", stringify!(movePlyType))
    );
    assert_eq!(
        ::std::mem::align_of::<movePlyType>(),
        4usize,
        concat!("Alignment of ", stringify!(movePlyType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<movePlyType>())).move_ as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(movePlyType),
            "::",
            stringify!(move_)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<movePlyType>())).current as *const _ as usize },
        224usize,
        concat!(
            "Offset of field: ",
            stringify!(movePlyType),
            "::",
            stringify!(current)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<movePlyType>())).last as *const _ as usize },
        228usize,
        concat!(
            "Offset of field: ",
            stringify!(movePlyType),
            "::",
            stringify!(last)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct highCardType {
    pub rank: ::std::os::raw::c_int,
    pub hand: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_highCardType() {
    assert_eq!(
        ::std::mem::size_of::<highCardType>(),
        8usize,
        concat!("Size of: ", stringify!(highCardType))
    );
    assert_eq!(
        ::std::mem::align_of::<highCardType>(),
        4usize,
        concat!("Alignment of ", stringify!(highCardType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<highCardType>())).rank as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(highCardType),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<highCardType>())).hand as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(highCardType),
            "::",
            stringify!(hand)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pos {
    pub rankInSuit: [[::std::os::raw::c_ushort; 4usize]; 4usize],
    pub aggr: [::std::os::raw::c_ushort; 4usize],
    pub length: [[::std::os::raw::c_uchar; 4usize]; 4usize],
    pub handDist: [::std::os::raw::c_int; 4usize],
    pub winRanks: [[::std::os::raw::c_ushort; 4usize]; 50usize],
    pub first: [::std::os::raw::c_int; 50usize],
    pub move_: [moveType; 50usize],
    pub handRelFirst: ::std::os::raw::c_int,
    pub tricksMAX: ::std::os::raw::c_int,
    pub winner: [highCardType; 4usize],
    pub secondBest: [highCardType; 4usize],
}
#[test]
fn bindgen_test_layout_pos() {
    assert_eq!(
        ::std::mem::size_of::<pos>(),
        1544usize,
        concat!("Size of: ", stringify!(pos))
    );
    assert_eq!(
        ::std::mem::align_of::<pos>(),
        4usize,
        concat!("Alignment of ", stringify!(pos))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).rankInSuit as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(rankInSuit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).aggr as *const _ as usize },
        32usize,
        concat!("Offset of field: ", stringify!(pos), "::", stringify!(aggr))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).length as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(length)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).handDist as *const _ as usize },
        56usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(handDist)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).winRanks as *const _ as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(winRanks)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).first as *const _ as usize },
        472usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(first)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).move_ as *const _ as usize },
        672usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(move_)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).handRelFirst as *const _ as usize },
        1472usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(handRelFirst)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).tricksMAX as *const _ as usize },
        1476usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(tricksMAX)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).winner as *const _ as usize },
        1480usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(winner)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<pos>())).secondBest as *const _ as usize },
        1512usize,
        concat!(
            "Offset of field: ",
            stringify!(pos),
            "::",
            stringify!(secondBest)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct evalType {
    pub tricks: ::std::os::raw::c_int,
    pub winRanks: [::std::os::raw::c_ushort; 4usize],
}
#[test]
fn bindgen_test_layout_evalType() {
    assert_eq!(
        ::std::mem::size_of::<evalType>(),
        12usize,
        concat!("Size of: ", stringify!(evalType))
    );
    assert_eq!(
        ::std::mem::align_of::<evalType>(),
        4usize,
        concat!("Alignment of ", stringify!(evalType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<evalType>())).tricks as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(evalType),
            "::",
            stringify!(tricks)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<evalType>())).winRanks as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(evalType),
            "::",
            stringify!(winRanks)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct card {
    pub suit: ::std::os::raw::c_int,
    pub rank: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_card() {
    assert_eq!(
        ::std::mem::size_of::<card>(),
        8usize,
        concat!("Size of: ", stringify!(card))
    );
    assert_eq!(
        ::std::mem::align_of::<card>(),
        4usize,
        concat!("Alignment of ", stringify!(card))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<card>())).suit as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(card),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<card>())).rank as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(card),
            "::",
            stringify!(rank)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct extCard {
    pub suit: ::std::os::raw::c_int,
    pub rank: ::std::os::raw::c_int,
    pub sequence: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_extCard() {
    assert_eq!(
        ::std::mem::size_of::<extCard>(),
        12usize,
        concat!("Size of: ", stringify!(extCard))
    );
    assert_eq!(
        ::std::mem::align_of::<extCard>(),
        4usize,
        concat!("Alignment of ", stringify!(extCard))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<extCard>())).suit as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(extCard),
            "::",
            stringify!(suit)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<extCard>())).rank as *const _ as usize },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(extCard),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<extCard>())).sequence as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(extCard),
            "::",
            stringify!(sequence)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct absRankType {
    pub rank: ::std::os::raw::c_char,
    pub hand: ::std::os::raw::c_schar,
}
#[test]
fn bindgen_test_layout_absRankType() {
    assert_eq!(
        ::std::mem::size_of::<absRankType>(),
        2usize,
        concat!("Size of: ", stringify!(absRankType))
    );
    assert_eq!(
        ::std::mem::align_of::<absRankType>(),
        1usize,
        concat!("Alignment of ", stringify!(absRankType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<absRankType>())).rank as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(absRankType),
            "::",
            stringify!(rank)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<absRankType>())).hand as *const _ as usize },
        1usize,
        concat!(
            "Offset of field: ",
            stringify!(absRankType),
            "::",
            stringify!(hand)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct relRanksType {
    pub absRank: [[absRankType; 4usize]; 15usize],
}
#[test]
fn bindgen_test_layout_relRanksType() {
    assert_eq!(
        ::std::mem::size_of::<relRanksType>(),
        120usize,
        concat!("Size of: ", stringify!(relRanksType))
    );
    assert_eq!(
        ::std::mem::align_of::<relRanksType>(),
        1usize,
        concat!("Alignment of ", stringify!(relRanksType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<relRanksType>())).absRank as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(relRanksType),
            "::",
            stringify!(absRank)
        )
    );
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct paramType {
    pub noOfBoards: ::std::os::raw::c_int,
    pub bop: *mut boards,
    pub solvedp: *mut solvedBoards,
    pub error: ::std::os::raw::c_int,
}
#[test]
fn bindgen_test_layout_paramType() {
    assert_eq!(
        ::std::mem::size_of::<paramType>(),
        32usize,
        concat!("Size of: ", stringify!(paramType))
    );
    assert_eq!(
        ::std::mem::align_of::<paramType>(),
        8usize,
        concat!("Alignment of ", stringify!(paramType))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<paramType>())).noOfBoards as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(paramType),
            "::",
            stringify!(noOfBoards)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<paramType>())).bop as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(paramType),
            "::",
            stringify!(bop)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<paramType>())).solvedp as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(paramType),
            "::",
            stringify!(solvedp)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<paramType>())).error as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(paramType),
            "::",
            stringify!(error)
        )
    );
}
