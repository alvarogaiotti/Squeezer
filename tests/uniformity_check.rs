use squeezer::{Cards, FastSampler, UniformSampler};
use std::collections::HashMap;

#[test]
fn uniformity_test() {
    let mut rng = rand::rng();
    let mut counts_a = HashMap::new();
    let mut counts_b = HashMap::new();
    for _ in 0..100_000_000 {
        let mut deck = Cards::ALL;
        if let Some(hand) = deck.pick_rng::<_, FastSampler>(&mut rng, 13) {
            *counts_a.entry(hand).or_insert(0) += 1;
        }
        let mut deck = Cards::ALL;
        if let Some(hand) = deck.pick_rng::<_, UniformSampler>(&mut rng, 13) {
            *counts_b.entry(hand).or_insert(0) += 1;
        }
    }
    let unique_keys: std::collections::HashSet<_> =
        counts_a.keys().chain(counts_b.keys()).copied().collect();

    let mut chi2 = 0.0;
    let mut total_bins = 0;

    for key in unique_keys {
        let a = *counts_a.get(&key).unwrap_or(&0) as f64;
        let b = *counts_b.get(&key).unwrap_or(&0) as f64;
        let total = a + b;
        if total == 0.0 {
            continue;
        }
        let expected = total / 2.0;
        chi2 += ((a - expected).powi(2) + (b - expected).powi(2)) / expected;
        total_bins += 1;
    }

    println!("Chi-squared: {chi2:.4}");
    println!("Bins compared: {total_bins}");
    println!("Per-bin average χ²: {:.6}", chi2 / total_bins as f64);

    if chi2 > total_bins as f64 * 1.5 {
        println!("⚠️  Distributions appear statistically different.");
    } else {
        println!("✅  No strong evidence of distributional difference.");
    }
}
