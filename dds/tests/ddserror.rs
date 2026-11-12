// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Return-code coverage for [`DdsErrorKind`].
//!
//! The numeric codes and their text are DDS 3.1's ABI (`vendor/dds/library/src/api/dll.h`):
//! the wrapper must name each code rather than folding it into `UnknownFault`.

use dds::ddserror::{DdsError, DdsErrorKind};

#[test]
fn too_many_boards_has_its_own_error_kind() {
    // `RETURN_TOO_MANY_BOARDS` is -101; DDS's text is "Too many Boards requested".
    let error = DdsError::from(-101);

    assert!(matches!(error.kind, DdsErrorKind::TooManyBoards));
    assert_eq!(error.kind.to_string(), "too many boards requested");
}

#[test]
fn thread_missing_has_its_own_error_kind() {
    // `RETURN_THREAD_MISSING` is -104; DDS's text is
    // "Multi-threading system not present".
    let error = DdsError::from(-104);

    assert!(matches!(error.kind, DdsErrorKind::ThreadMissing));
    assert_eq!(
        error.kind.to_string(),
        "the multi-threading system is not present"
    );
}
