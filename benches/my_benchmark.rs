use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

#[allow(unused_imports)]
use safe_arch::*;
use squeezer::{Card, Cards, Seat, Suit};

fn winner_no_simd(
    previous_card: Card,
    card: Card,
    actual_player: Seat,
    winner: Seat,
    trump: Suit,
) -> (Seat, Card) {
    if previous_card.suit() != card.suit() {
        if card.suit() == trump {
            (actual_player, card)
        } else {
            (winner, previous_card)
        }
    } else if previous_card.rank() > card.rank() {
        (winner, previous_card)
    } else {
        (actual_player, card)
    }
}
fn wrapping_winner(cards: &mut [Card], trump: Suit) {
    let mut seat = Seat::North;
    let ptr = &mut seat;
    for trick in cards.chunks_exact(4) {
        trick.iter().copied().reduce(|winner, card| {
            let next_player = ptr.next();
            let winner = winner_no_simd(winner, card, *ptr, next_player, trump);
            *ptr = next_player;
            winner.1
        });
    }
}

fn wrapping_winner_lut(cards: &mut [Card], trump: Suit) {
    let mut seat = Seat::North;
    for trick in cards.chunks_exact(4) {
        let (winner, _) = find_trick_winner_lut(trick, trump, seat);
        seat = winner;
    }
}
fn find_trick_winner_lut(trick: &[Card], trump: Suit, lead_seat: Seat) -> (Seat, Card) {
    // Pre-compute card values for faster comparison
    assert!(trick.len() == 4);
    let trump_u8 = trump as u8;
    let lead_suit = trick[0].suit() as u8;

    let mut best_idx = 0;
    let mut best_value = card_value(trick[0], trump_u8, lead_suit);

    for (i, &card) in trick.iter().enumerate().skip(1) {
        let value = card_value(card, trump_u8, lead_suit);
        if value > best_value {
            best_idx = i;
            best_value = value;
        }
    }

    let winner_seat = match (lead_seat as u8 + best_idx as u8) % 4 {
        0 => Seat::North,
        1 => Seat::East,
        2 => Seat::South,
        3 => Seat::West,
        _ => unreachable!(),
    };

    (winner_seat, trick[best_idx])
}

#[inline]
fn card_value(card: Card, trump: u8, lead_suit: u8) -> u16 {
    let suit = card.suit() as u8;
    let rank = card.rank();

    if suit == trump {
        // Trump cards: highest priority (256+) + rank
        256 + rank as u16
    } else if suit == lead_suit {
        // Lead suit cards: medium priority (128+) + rank
        128 + rank as u16
    } else {
        // Off-suit cards: lowest priority, just rank
        rank as u16
    }
}

#[cfg(not(target_feature = "avx2"))]
fn simd_trick_win_trump(cards: &mut [Card], trump: Suit) {
    wrapping_winner(cards, trump)
}

#[cfg(target_feature = "avx2")]
fn simd_trick_win_trump(cards: &mut [Card], trump: Suit) {
    let len = input.len();
    let new_len = len / 4;
    //let reinterpreted_slice = unsafe {
    //    core::slice::from_raw_parts_mut(input[..new_len * 4].as_mut_ptr() as *mut u32, new_len)
    //};
    let trump_value = trump as u8 * 32;
    let mask = m128i::from([trump_value; 16]);
    let input = unsafe { core::slice::from_raw_parts_mut(input.as_mut_ptr().cast::<u8>(), 32) };

    for x in 1..3 {
        let simd_cards = m128i::from(
            std::convert::TryInto::<[u8; 16]>::try_into(&input[(16 * (x - 1))..(16 * x)]).unwrap(),
        );
        let card_to_mask = cmp_eq_mask_i8_m128i(simd_cards, mask);
        let to_set = bitand_m128i(card_to_mask, mask);
        let card_updated = bitor_m128i(simd_cards, to_set);
        let stuff: [u8; 16] = card_updated.into();
        let card_iterator = unsafe { std::mem::transmute::<[u8; 16], [Card; 16]>(stuff) };
        for trick in card_iterator.chunks_exact(4) {
            let winner = trick
                .iter()
                .enumerate()
                .max_by(|(index, card), (index2, card2)| {
                    (card.rank() + card.suit() as u8 * 16)
                        .cmp(&(card2.rank() + card2.suit() as u8 * 16))
                });
        }
    }
}

#[allow(const_item_mutation)]
fn criterion_benchmark(c: &mut Criterion) {
    let mut cards1 = Cards::ALL.pick(32).unwrap().into_iter();
    let mut cards2 = Cards::ALL.pick(32).unwrap().into_iter();
    let mut cards3 = Cards::ALL.pick(32).unwrap().into_iter();
    let array1: [Card; 32] = std::array::from_fn(|_| cards1.next().unwrap());
    let array2: [Card; 32] = std::array::from_fn(|_| cards2.next().unwrap());
    let array3: [Card; 32] = std::array::from_fn(|_| cards3.next().unwrap());
    let mut group = c.benchmark_group("Compare winner fn");
    for cards in [array1, array2, array3] {
        group.bench_with_input(
            BenchmarkId::new("simd", format!("{:?}", &cards)),
            &cards,
            |b, cards| {
                b.iter_batched(
                    || *cards,
                    |mut cards| simd_trick_win_trump(&mut cards, Suit::Spades),
                    criterion::BatchSize::SmallInput,
                )
            },
        );
        group.bench_with_input(
            BenchmarkId::new("no simd", format!("{:?}", &cards)),
            &cards,
            |b, cards| {
                b.iter_batched(
                    || *cards,
                    |mut cards| wrapping_winner(&mut cards, Suit::Spades),
                    criterion::BatchSize::SmallInput,
                )
            },
        );
        group.bench_with_input(
            BenchmarkId::new("no_simd_lut", format!("{:?}", &cards)),
            &cards,
            |b, cards| {
                b.iter_batched(
                    || *cards,
                    |mut cards| wrapping_winner_lut(&mut cards, Suit::Spades),
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
