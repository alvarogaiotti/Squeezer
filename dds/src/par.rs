// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::bindings::{
    ConvertToDealerTextFormat, ConvertToSidesTextFormat, DealerPar, DealerParBin, Par, SidesPar,
};
use crate::ddserror::DDSError;
use crate::deal::DdsHandEncoding;
use crate::tables::{DdTableResults, ParResults, Populated, VulnerabilityEncoding};
use crate::utils::if_no_fault_return;

pub trait ParCalculator {
    fn par(
        &self,
        tablep: &mut DdTableResults<Populated>,
        presp: &mut ParResults,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<(), DDSError> {
        let result = unsafe {
            Par(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                std::ptr::from_mut::<ParResults>(presp),
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, ());
    }

    fn calc_side_par(
        tablep: &mut DdTableResults<Populated>,
        side_res: &mut ParResultsDealer,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<(), DDSError> {
        let result = unsafe {
            SidesPar(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                std::ptr::from_mut::<ParResultsDealer>(side_res),
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, ());
    }

    fn dealer_par(
        tablep: &mut DdTableResults<Populated>,
        presp: &mut ParResultsDealer,
        dealer: DdsHandEncoding,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<(), DDSError> {
        let result = unsafe {
            DealerPar(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                std::ptr::from_mut::<ParResultsDealer>(presp),
                dealer as i32,
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, ());
    }

    fn dealer_par_bin(
        tablep: &mut DdTableResults<Populated>,
        presp: &mut ParResultsMaster,
        dealer: DdsHandEncoding,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<(), DDSError> {
        let result = unsafe {
            DealerParBin(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                std::ptr::from_mut::<ParResultsMaster>(presp),
                dealer as i32,
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, ());
    }

    fn calc_side_par_bin(
        tablep: &mut DdTableResults<Populated>,
        side_res: &mut ParResultsDealer,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<(), DDSError> {
        let result = unsafe {
            SidesPar(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                std::ptr::from_mut::<ParResultsDealer>(side_res),
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, ());
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ParResultsDealer {
    pub number: ::std::os::raw::c_int,
    pub score: ::std::os::raw::c_int,
    pub contracts: [[::std::os::raw::c_char; 10usize]; 10usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ParResultsMaster {
    pub score: ::std::os::raw::c_int,
    pub number: ::std::os::raw::c_int,
    pub contracts: [ContractType; 10usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ContractType {
    pub under_tricks: ::std::os::raw::c_int,
    pub over_tricks: ::std::os::raw::c_int,
    pub level: ::std::os::raw::c_int,
    pub denom: ::std::os::raw::c_int,
    pub seats: ::std::os::raw::c_int,
}

pub fn convert_to_dealer_text_format(pres: &mut ParResultsMaster) -> Result<String, DDSError> {
    let mut resp = String::with_capacity(100);
    let result = unsafe {
        ConvertToDealerTextFormat(
            std::ptr::from_mut::<ParResultsMaster>(pres),
            resp.as_mut_ptr() as *mut i8,
        )
    };
    if_no_fault_return!(result, resp);
}

pub fn convert_to_sides_text_format(
    pres: &mut ParResultsMaster,
    resp: &mut ParTextResults,
) -> Result<(), DDSError> {
    let result = unsafe {
        ConvertToSidesTextFormat(
            std::ptr::from_mut::<ParResultsMaster>(pres),
            std::ptr::from_mut::<ParTextResults>(resp),
        )
    };
    if_no_fault_return!(result, ());
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ParTextResults {
    pub par_text: [[::std::os::raw::c_char; 128usize]; 2usize],
    pub equal: bool,
}

#[cfg(test)]
#[allow(deref_nullptr)]
mod test {
    use super::*;

    #[test]
    fn bindgen_test_layout_par_text_results() {
        assert_eq!(
            ::std::mem::size_of::<ParTextResults>(),
            257usize,
            concat!("Size of: ", stringify!(ParTextResults))
        );
        assert_eq!(
            ::std::mem::align_of::<ParTextResults>(),
            1usize,
            concat!("Alignment of ", stringify!(ParTextResults))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ParTextResults>())).par_text as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ParTextResults),
                "::",
                stringify!(ParText)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ParTextResults>())).equal as *const _ as usize },
            256usize,
            concat!(
                "Offset of field: ",
                stringify!(ParTextResults),
                "::",
                stringify!(equal)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_contract_type() {
        assert_eq!(
            ::std::mem::size_of::<ContractType>(),
            20usize,
            concat!("Size of: ", stringify!(ContractType))
        );
        assert_eq!(
            ::std::mem::align_of::<ContractType>(),
            4usize,
            concat!("Alignment of ", stringify!(ContractType))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ContractType>())).under_tricks as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ContractType),
                "::",
                stringify!(under_tricks)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ContractType>())).over_tricks as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ContractType),
                "::",
                stringify!(over_tricks)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ContractType>())).level as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(ContractType),
                "::",
                stringify!(level)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ContractType>())).denom as *const _ as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(ContractType),
                "::",
                stringify!(denom)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ContractType>())).seats as *const _ as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(ContractType),
                "::",
                stringify!(seats)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_par_results_master() {
        assert_eq!(
            ::std::mem::size_of::<ParResultsMaster>(),
            208usize,
            concat!("Size of: ", stringify!(ParResultsMaster))
        );
        assert_eq!(
            ::std::mem::align_of::<ParResultsMaster>(),
            4usize,
            concat!("Alignment of ", stringify!(ParResultsMaster))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ParResultsMaster>())).score as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ParResultsMaster),
                "::",
                stringify!(score)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ParResultsMaster>())).number as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ParResultsMaster),
                "::",
                stringify!(number)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ParResultsMaster>())).contracts as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(ParResultsMaster),
                "::",
                stringify!(contracts)
            )
        );
    }
    #[test]
    fn bindgen_test_layout_par_results_dealer() {
        assert_eq!(
            ::std::mem::size_of::<ParResultsDealer>(),
            108usize,
            concat!("Size of: ", stringify!(ParResultsDealer))
        );
        assert_eq!(
            ::std::mem::align_of::<ParResultsDealer>(),
            4usize,
            concat!("Alignment of ", stringify!(ParResultsDealer))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ParResultsDealer>())).number as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ParResultsDealer),
                "::",
                stringify!(number)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ParResultsDealer>())).score as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ParResultsDealer),
                "::",
                stringify!(score)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ParResultsDealer>())).contracts as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(ParResultsDealer),
                "::",
                stringify!(contracts)
            )
        );
    }
}
