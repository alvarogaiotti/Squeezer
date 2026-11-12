//! `HcpRange`'s three constructor spellings must agree.
//!
//! The normalisation rule itself is documented on [`HcpRange`]; these tests
//! pin that all three spellings land on it for ordinary, inverted and
//! over-cap bounds.

use squeezer_model::HcpRange;

#[test]
fn every_constructor_normalises_inverted_bounds_the_same_way() {
    struct Case {
        input: (u8, u8),
        expected: (u8, u8),
    }
    let cases = [
        Case {
            input: (12, 15),
            expected: (12, 15), // ordinary window
        },
        Case {
            input: (5, 3),
            expected: (3, 3), // inverted: narrows to the upper bound
        },
        Case {
            input: (15, 12),
            expected: (12, 12), // inverted: still narrows, never swaps
        },
        Case {
            input: (40, 45),
            expected: (37, 37), // both above the cap
        },
        Case {
            input: (40, 5),
            expected: (5, 5), // cap applies before the inversion is resolved
        },
        Case {
            input: (0, 45),
            expected: (0, 37), // only the upper bound exceeds the cap
        },
    ];

    for Case { input, expected } in cases {
        let (input_min, input_max) = input;
        let from_new = HcpRange::new(input_min, input_max);
        let from_pair = HcpRange::from(input);
        let from_range = HcpRange::from_range(input_min..=input_max);

        assert_eq!(
            (from_new.range_min(), from_new.range_max()),
            expected,
            "HcpRange::new({input_min}, {input_max})"
        );
        assert_eq!(
            from_pair, from_new,
            "HcpRange::from(({input_min}, {input_max})) disagrees with new"
        );
        assert_eq!(
            from_range, from_new,
            "HcpRange::from_range({input_min}..={input_max}) disagrees with new"
        );
    }
}

#[test]
fn from_range_keeps_its_half_open_and_unbounded_reading() {
    // An exclusive end means "stop one short", as before.
    assert_eq!(
        HcpRange::from_range(12..15),
        HcpRange::new(12, 14),
        "12..15 is the inclusive 12..=14 window"
    );

    // Unbounded ends take the edges of the representable range.
    assert_eq!(
        HcpRange::from_range::<std::ops::RangeFull, u8>(..),
        HcpRange::new(0, 37)
    );
    assert_eq!(HcpRange::from_range(12..), HcpRange::new(12, 37));
    assert_eq!(HcpRange::from_range(..=10), HcpRange::new(0, 10));
}

#[test]
fn a_normalised_range_reports_membership_at_its_new_edges() {
    // The collapsed window is a real window, not just a pair of fields.
    let collapsed = HcpRange::from((5, 3));
    assert!(collapsed.contains(3));
    assert!(!collapsed.contains(4));
    assert!(!collapsed.contains(2));
    assert_eq!(collapsed.as_range(), 3..=3);

    let capped = HcpRange::new(40, 45);
    assert!(capped.contains(37));
    assert!(!capped.contains(36));
}
