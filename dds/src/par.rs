// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::context::{DealerPar, Par};
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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ParTextResultHalfBuffer(
    #[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
    pub  [std::os::raw::c_char; 128usize],
);

#[cfg(test)]
#[allow(deref_nullptr, clippy::ref_as_ptr)]
mod test {
    use super::*;

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
            unsafe { &raw const (*(::std::ptr::null::<ParResultsDealer>())).number as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ParResultsDealer),
                "::",
                stringify!(number)
            )
        );
        assert_eq!(
            unsafe { &raw const (*(::std::ptr::null::<ParResultsDealer>())).score as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(ParResultsDealer),
                "::",
                stringify!(score)
            )
        );
        assert_eq!(
            unsafe { &raw const (*(::std::ptr::null::<ParResultsDealer>())).contracts as usize },
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
