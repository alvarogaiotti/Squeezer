// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::bindings::{
    ConvertToDealerTextFormat, ConvertToSidesTextFormat, DealerPar, DealerParBin, Par, SidesPar,
    SidesParBin,
};
use crate::ddserror::DdsError;
use crate::deal::DdsHandEncoding;
use crate::tables::{DdTableResults, ParResults, Populated, VulnerabilityEncoding};
use crate::utils::if_no_fault_return;

/// Trait representing the ability to do par calculations for deals.
pub trait ParCalculator {
    /// Standard simple par for a deal. Doesn't use dealer information, so both sides might have a 1NT par.
    ///
    /// # Errors
    ///
    /// Errors when DDS returns an error
    fn par(
        &self,
        tablep: &mut DdTableResults<Populated>,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<ParResults, DdsError> {
        let mut presp = ParResults::new();
        let result = unsafe {
            Par(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                std::ptr::from_mut::<ParResults>(&mut presp),
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, presp);
    }

    /// Calculate par for a specific side.
    ///
    /// # Errors
    ///
    /// Errors when DDS returns an error
    fn side_par(
        tablep: &mut DdTableResults<Populated>,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<[ParResultsDealer; 2], DdsError> {
        let mut side_res = [ParResultsDealer::new(), ParResultsDealer::new()];
        let result = unsafe {
            SidesPar(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                side_res.as_mut_ptr(),
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, side_res);
    }

    /// Calculate par based on dealer information.
    /// Basically if both sides can make 1NT, only the dealer side will be reported as it can declare 1NT first.
    ///
    /// # Errors
    ///
    /// Errors when DDS returns an error
    fn dealer_par(
        tablep: &mut DdTableResults<Populated>,
        dealer: DdsHandEncoding,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<ParResultsDealer, DdsError> {
        let mut par = ParResultsDealer::new();
        let result = unsafe {
            DealerPar(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                std::ptr::from_mut::<ParResultsDealer>(&mut par),
                dealer as i32,
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, par);
    }

    /// Same of [`ParCalculator::dealer_par()`], but gives back results in binary format. Easier if you have to use the information obtained from par calculation
    /// instead of simply displaying it to the end user.
    ///
    /// # Errors
    ///
    /// Errors when DDS returns an error
    fn dealer_par_bin(
        tablep: &mut DdTableResults<Populated>,
        dealer: DdsHandEncoding,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<ParResultsMaster, DdsError> {
        let mut presp = ParResultsMaster::new();
        let result = unsafe {
            DealerParBin(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                std::ptr::from_mut::<ParResultsMaster>(&mut presp),
                dealer as i32,
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, presp);
    }

    /// Same of [`ParCalculator::side_par()`], but gives back results in binary format. Easier if you have to use the information obtained from par calculation
    /// instead of simply displaying it to the end user.
    ///
    /// # Errors
    ///
    /// Errors when DDS returns an error
    fn side_par_bin(
        tablep: &mut DdTableResults<Populated>,
        vulnerable: VulnerabilityEncoding,
    ) -> Result<ParResultsMaster, DdsError> {
        let mut side_res = ParResultsMaster::new();
        let result = unsafe {
            SidesParBin(
                std::ptr::from_mut::<DdTableResults<Populated>>(tablep),
                std::ptr::from_mut::<ParResultsMaster>(&mut side_res),
                vulnerable as i32,
            )
        };
        if_no_fault_return!(result, side_res);
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
/// Struct used by DDS for storing side oriented par results.
pub struct ParResultsDealer {
    pub number: ::std::os::raw::c_int,
    pub score: ::std::os::raw::c_int,
    pub contracts: [[::std::os::raw::c_char; 10usize]; 10usize],
}

impl ParResultsDealer {
    const fn new() -> Self {
        Self {
            number: 0,
            score: 0,
            contracts: [[0; 10]; 10],
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct ParResultsMaster {
    pub score: ::std::os::raw::c_int,
    pub number: ::std::os::raw::c_int,
    pub contracts: [ContractType; 10usize],
}

impl ParResultsMaster {
    const fn new() -> Self {
        Self {
            score: 0,
            number: 0,
            contracts: [ContractType::new(); 10],
        }
    }
}

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct ContractType {
    pub under_tricks: ::std::os::raw::c_int,
    pub over_tricks: ::std::os::raw::c_int,
    pub level: ::std::os::raw::c_int,
    pub denom: ::std::os::raw::c_int,
    pub seats: ::std::os::raw::c_int,
}

impl ContractType {
    const fn new() -> Self {
        Self {
            under_tricks: -1,
            over_tricks: -1,
            level: -1,
            denom: -1,
            seats: -1,
        }
    }
}

/// Utility function provided by DDS to convert a [`ParResultsMaster`] to a string.
///
/// # Errors
///
/// Errors when DDS returns an error
pub fn convert_to_dealer_text_format(pres: &mut ParResultsMaster) -> Result<String, DdsError> {
    let mut resp = String::with_capacity(100);
    let result = unsafe {
        ConvertToDealerTextFormat(
            std::ptr::from_mut::<ParResultsMaster>(pres),
            resp.as_mut_ptr().cast::<i8>(),
        )
    };
    if_no_fault_return!(result, resp);
}

/// Utility function provided by DDS to convert a [`ParResultsMaster`] to a [`ParTextResults`].
///
/// # Errors
///
/// Errors when DDS returns an error
pub fn convert_to_sides_text_format(
    pres: &mut ParResultsMaster,
    resp: &mut ParTextResults,
) -> Result<(), DdsError> {
    let result = unsafe {
        ConvertToSidesTextFormat(
            std::ptr::from_mut::<ParResultsMaster>(pres),
            std::ptr::from_mut::<ParTextResults>(resp),
        )
    };
    if_no_fault_return!(result, ());
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ParTextResultHalfBuffer(
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    pub  [std::os::raw::c_char; 128usize],
);

#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct ParTextResults {
    pub par_text: [ParTextResultHalfBuffer; 2usize],
    pub equal: bool,
}

#[cfg(test)]
#[allow(deref_nullptr, clippy::ref_as_ptr)]
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
