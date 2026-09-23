// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::*;

/// Builds the Polish Club 1♣ opening constraint: weak balanced 11-14,
/// unbalanced with clubs 11-17, or any strong 18+ hand.
///
/// # Errors
///
/// Returns [`ConstraintError`] if an internal pattern is malformed. The
/// patterns are fixed tables covered by tests, so this signals a crate bug,
/// not caller input.
pub fn polish_club_hand_constraint() -> Result<HandConstraint, ConstraintError> {
    let weak1n = HandConstraintBuilder::new()
        .add_shape("(4432)")?
        .add_shape("(4333)")?
        .add_shape("(4414)")?
        .remove_shape("4441")?
        .add_shape("(5332)")?
        .remove_shape("(5x)(2+2+)")?
        .with_range(11, 14)
        .build()?;
    let unbal_with_clubs = HandConstraintBuilder::new()
        .with_longest(Suit::Clubs)?
        .remove_shape("(4x)x5")?
        .remove_shape("(5-5-5-)6+")?
        .with_range(11, 17)
        .build()?;
    let strong_any = HandConstraintBuilder::new()
        .all_shapes()
        .with_range(18, 37)
        .build()?;
    HandConstraint::any(vec![weak1n, strong_any, unbal_with_clubs])
}

/// Legacy Polish Club descriptor kept for compatibility.
///
/// Prefer [`polish_club_hand_constraint`]: same bridge meaning, but behind
/// the deepened [`HandConstraint`] interface with a typed error.
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn polish_club_hand_descriptor() -> HandDescriptor {
    let weak1n = HandTypeBuilder::new()
        .add_shape("(4432)")
        .unwrap()
        .add_shape("(4333)")
        .unwrap()
        .add_shape("(4414)")
        .unwrap()
        .remove_shape("4441")
        .unwrap()
        .add_shape("(5332)")
        .unwrap()
        .remove_shape("(5x)(2+2+)")
        .unwrap()
        .with_range(11, 14)
        .build();
    let unbal_with_clubs = HandTypeBuilder::new()
        .with_longest(Suit::Clubs)
        .remove_shape("(4x)x5")
        .unwrap()
        .remove_shape("(5-5-5-)6+")
        .unwrap()
        .with_range(11, 17)
        .build();
    let strong_any = HandTypeBuilder::new()
        .all_shapes()
        .with_range(18, 37)
        .build();
    HandDescriptor::new(vec![weak1n, strong_any, unbal_with_clubs])
    /* let hand = hands[seat as usize];
    hand_type.check(&hand) && 10 < hand.hcp() && hand.hcp() < 15
        || hand.clubs().len() == hand.into_iter().map(|x| x.len()).max().unwrap()
            && !Shapes::new().add_shape("(5xx)5").(&hand, "(5xx)5")
        || factory.is_not_in(&hand, "(144)4") && 14 < hand.hcp()
        || hand.hcp() > 17 */
}

const W2_EVALUATOR: Evaluator = Evaluator::new(&[2, 2, 1, 1, 1]);
const CONTROLS_EVALUATOR: Evaluator = Evaluator::new(&[2, 1]);
const ZAR_EVALUATOR: Evaluator = Evaluator::new(&[6, 4, 2, 1]);

/// Weak-two opening detector: 6-card major, 5-10 HCP, weak controls.
///
/// # Panics
///
/// Panics only if a fixed shape pattern stops parsing (a crate bug: the
/// patterns are literals covered by tests).
#[must_use]
pub fn weak2_hand_descriptor(hand: Hand) -> bool {
    let wk2 = HandConstraintBuilder::new()
        .add_shape("(63-)4-4-")
        .expect("fixed weak-two shape pattern parses")
        .with_range(5, 10)
        .build()
        .expect("fixed weak-two constraint is non-empty");
    wk2.check(hand)
        && ((hand.slen() == 6
            && W2_EVALUATOR.evaluate(hand.spades()) > 3
            && CONTROLS_EVALUATOR.evaluate(hand.cards()) < 4)
            || (hand.hlen() == 6
                && W2_EVALUATOR.evaluate(hand.hearts()) > 3
                && CONTROLS_EVALUATOR.evaluate(hand.cards()) < 4))
}

#[must_use]
fn devaluate_short_honors(hand: Hand) -> u8 {
    let mut malus: u8 = 0;
    malus += hand
        .into_iter()
        .map(|suit| match suit.len() {
            0..=1 => suit.kings().len() + suit.queens().len() + suit.jacks().len(),
            2 => suit.queens().len() + suit.jacks().len(),
            _ => 0u8,
        })
        .sum::<u8>();
    malus
}

/// Sorts four suit holdings by length in ascending order using a sorting network.
fn sort_suit_lengths(suits: &mut [Cards; 4]) {
    fn cmp_swap(suits: &mut [Cards; 4], i: usize, j: usize) {
        if suits[i].len() > suits[j].len() {
            suits.swap(i, j);
        }
    }
    cmp_swap(suits, 0, 1);
    cmp_swap(suits, 2, 3);
    cmp_swap(suits, 0, 2);
    cmp_swap(suits, 1, 3);
    cmp_swap(suits, 1, 2);
}

#[must_use]
fn evaluate_lenght_and_concentration(hand: Hand) -> u8 {
    let mut suits = [hand.spades(), hand.hearts(), hand.diamonds(), hand.clubs()];
    sort_suit_lengths(&mut suits);
    let [shortest, second_shortest, second_longest, longest] = suits;
    let sum_long = longest.len() + second_longest.len();
    let diff_long_short = longest.len() - shortest.len();
    let mut points = sum_long + diff_long_short;
    let weak_concentration = 10 < hand.hcp()
        && hand.hcp() < 15
        && longest.high_card_points() + second_longest.high_card_points() >= hand.hcp() - 1;
    let strong_concentration = 14 < hand.hcp()
        && longest.high_card_points()
            + second_longest.high_card_points()
            + second_shortest.high_card_points()
            >= hand.hcp() - 1;
    if weak_concentration || strong_concentration {
        points += 1;
    }

    if points == 25 && hand.spades().len() > 3 {
        points += 1;
    }
    points
}
#[must_use]
pub fn zar_points(hand: Hand) -> u8 {
    ZAR_EVALUATOR.evaluate(hand.cards()) + evaluate_lenght_and_concentration(hand)
        - devaluate_short_honors(hand)
}

/// Dealer for the Lavazza 3NT opening: long minor, 26-32 Zar points.
///
/// # Panics
///
/// Panics only if a fixed shape pattern stops parsing (a crate bug: the
/// patterns are literals covered by tests).
#[must_use]
pub fn dealer_of_lavazza_3nt_opening(seat: Option<Seat>) -> impl Dealer {
    let hand_type = HandConstraintBuilder::new()
        .add_shape("(8x)xx")
        .expect("fixed Lavazza shape pattern parses")
        .add_shape("(7x)xx")
        .expect("fixed Lavazza shape pattern parses")
        .add_shape("(9x)xx")
        .expect("fixed Lavazza shape pattern parses")
        .build()
        .expect("fixed Lavazza constraint is non-empty");
    let builder = DealerBuilder::new().with_function(Box::new(move |deal: &Deal| {
        if let Some(seat) = seat {
            hand_type.check(deal[seat as usize])
                && (26..33).contains(&zar_points(deal[seat as usize]))
        } else {
            deal.iter()
                .any(|hand| hand_type.check(*hand) && (26..33).contains(&zar_points(*hand)))
        }
    }));
    builder.build().unwrap()
}
