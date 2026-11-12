//! The card-set operations only have bits for the first 64 packed offsets.
//!
//! `Cards` packs one card per bit of a `u64`, so a card whose offset reaches
//! 64 has no bit at all. `Card::JOKER` lives at `u8::MAX`: every set operation
//! treats it as absent instead of shifting by 255.

use squeezer_cards::{Card, Cards, Suit};

#[test]
fn joker_is_absent_from_every_card_set() {
    // Inserting the joker leaves the set untouched, so `insert` stays total
    // instead of panicking on the out-of-range shift.
    assert_eq!(Cards::EMPTY.insert(Card::JOKER), Cards::EMPTY);
    assert_eq!(Cards::ALL.insert(Card::JOKER), Cards::ALL);

    // Removing a card that cannot be present is the same no-op.
    assert_eq!(Cards::ALL.remove(Card::JOKER), Cards::ALL);
    assert_eq!(Cards::EMPTY.remove(Card::JOKER), Cards::EMPTY);

    // And membership is always false, never a set bit 255.
    assert!(!Cards::ALL.has_card(Card::JOKER));
    assert!(!Cards::EMPTY.has_card(Card::JOKER));
}

#[test]
fn the_conversions_also_treat_the_joker_as_absent() {
    // `From<Card>` and `From<&Vec<Card>>` route through the same bit
    // operation, so they must agree with `insert`.
    assert_eq!(Cards::from(Card::JOKER), Cards::EMPTY);
    assert_eq!(Cards::from(Card::SA), Cards::EMPTY.insert(Card::SA));

    let joker_only = Cards::from(&vec![Card::JOKER]);
    assert!(joker_only.is_empty());

    let with_joker = Cards::from(&vec![Card::JOKER, Card::SA, Card::HK]);
    let without_joker = Cards::from(&vec![Card::SA, Card::HK]);
    assert_eq!(with_joker, without_joker);
}

#[test]
fn every_real_card_still_round_trips_through_the_set() {
    // The joker guard must not cost the deck any of its 52 cards: each one
    // inserts, is found, and removes.
    for suit in Suit::ALL {
        for rank in 2..=14 {
            let card = Card::new(*suit, rank);
            let set = Cards::EMPTY.insert(card);
            assert_eq!(set.len(), 1, "{card:?} should occupy one bit");
            assert!(set.has_card(card), "{card:?} should be found");
            assert_eq!(set.remove(card), Cards::EMPTY);
        }
    }
    assert_eq!(Cards::new_deck().len(), 52);
}

#[test]
fn a_card_whose_offset_left_the_set_is_also_absent() {
    // The guard is "offset below 64", not "offset is JOKER": a rank that
    // spills out of its suit nibble (16 of clubs packs to offset 64) must not
    // shift by 64 either.
    let spilled = Card::new(Suit::Clubs, 16);
    assert_eq!(spilled.as_u8(), 64);
    assert_eq!(Cards::ALL.insert(spilled), Cards::ALL);
    assert!(!Cards::ALL.has_card(spilled));
    assert_eq!(Cards::ALL.remove(spilled), Cards::ALL);
}
