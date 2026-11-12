// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Terminal Formatters
//!
//! This module provides formatters for displaying simulation results in the terminal.
//! These are completely separate from the core simulation logic, making it easy to
//! create custom output formats or skip terminal output entirely in a GUI context.
//!
//! The module is behind the `formatters` feature and is the only output path:
//! the former CSV export and chart-data helpers were deleted. The simulation
//! results it renders come from whichever solver backend the caller
//! configured.
//!
//! ## Example
//!
//! This example is `no_run`: it formats a real payoff matrix, so it needs the
//! native DDS backend and 2,000 solved deals. It is still compiled, so the API
//! is checked; see the `simulation` module documentation for an executable run
//! against a mock factory.
//!
//! ```no_run
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use squeezer_model::*;
//! use squeezer_sim::simulation::formatters::*;
//! use squeezer_sim::simulation::*;
//! use std::str::FromStr;
//!
//! let sim = ContractPayoffSimulation::builder()
//!     .num_deals(1000)
//!     .dealer(StandardDealer::new())
//!     .contracts(vec![
//!         Contract::from_str("3CN")?,
//!         Contract::from_str("3NN")?,
//!     ])
//!     .denomination(Denomination::Compared {
//!         baseline: Baseline::AverageOfOptions,
//!         scoring: Scoring::Imps,
//!     })
//!     .build()?;
//!
//! let results = sim.run()?;
//!
//! // Format for terminal output
//! let formatter = PayoffMatrixFormatter::new(&results.statistics);
//! formatter.print();
//!
//! // Or get as a string
//! let output = formatter.to_string();
//! # Ok(())
//! # }
//! ```

use crate::simulation::{LeadAnalysis, PayoffComparison, PayoffMatrix};
use colored::Colorize;
use std::fmt::{self, Write as FmtWrite};

// ============================================================================
// PAYOFF MATRIX FORMATTER
// ============================================================================

/// Formatter for payoff matrices (contract or card comparisons)
pub struct PayoffMatrixFormatter<'a, T: Clone + Eq + std::hash::Hash + fmt::Display> {
    matrix: &'a PayoffMatrix<T>,
    config: FormatterConfig,
}

impl<'a, T: Clone + Eq + std::hash::Hash + fmt::Display> PayoffMatrixFormatter<'a, T> {
    /// Create a new formatter with default configuration
    #[must_use]
    #[inline]
    pub fn new(matrix: &'a PayoffMatrix<T>) -> Self {
        Self {
            matrix,
            config: FormatterConfig::default(),
        }
    }

    /// Create with custom configuration
    #[must_use]
    #[inline]
    pub fn with_config(matrix: &'a PayoffMatrix<T>, config: FormatterConfig) -> Self {
        Self { matrix, config }
    }

    /// Print to stdout
    pub fn print(&self) {
        println!("{self}");
    }

    /// Print detailed comparison for two specific options
    pub fn print_detailed_comparison(&self, option_a: &T, option_b: &T) {
        println!("{}", self.detailed_comparison_string(option_a, option_b));
    }

    /// Get detailed comparison as string
    pub fn detailed_comparison_string(&self, option_a: &T, option_b: &T) -> String {
        let comparison = self
            .matrix
            .comparisons
            .iter()
            .find(|c| &c.option_a == option_a && &c.option_b == option_b);

        match comparison {
            Some(comp) => Self::format_detailed_comparison(comp),
            None => format!("No comparison found between {option_a} and {option_b}"),
        }
    }

    fn format_detailed_comparison(comp: &PayoffComparison<T>) -> String {
        let mut buffer = String::with_capacity(500);

        writeln!(
            buffer,
            "\n{}",
            format!(
                "Detailed Comparison: {} vs {}",
                comp.option_a, comp.option_b
            )
            .bold()
        )
        .unwrap();
        writeln!(buffer, "{}", "=".repeat(60)).unwrap();

        writeln!(buffer, "\n{}", "Statistics:".bold()).unwrap();
        writeln!(buffer, "  Mean difference:     {:>8.2}", comp.stats.mean).unwrap();
        writeln!(buffer, "  Standard deviation:  {:>8.2}", comp.stats.std_dev).unwrap();
        writeln!(
            buffer,
            "  Standard error:      {:>8.2}",
            comp.stats.std_error
        )
        .unwrap();
        writeln!(buffer, "  Median difference:   {:>8.2}", comp.stats.median).unwrap();

        writeln!(buffer, "\n{}", "Range:".bold()).unwrap();
        writeln!(buffer, "  Minimum:             {:>8}", comp.stats.min).unwrap();
        writeln!(buffer, "  Q1 (25th percentile):{:>8.2}", comp.stats.q1).unwrap();
        writeln!(buffer, "  Q3 (75th percentile):{:>8.2}", comp.stats.q3).unwrap();
        writeln!(buffer, "  Maximum:             {:>8}", comp.stats.max).unwrap();

        writeln!(buffer, "\n{}", "Win/Loss Record:".bold()).unwrap();

        #[allow(clippy::cast_precision_loss)]
        writeln!(
            buffer,
            "  {} better: {} deals ({:.1}%)",
            comp.option_a,
            comp.stats.a_better_count,
            (comp.stats.a_better_count as f64
                / (comp.stats.a_better_count + comp.stats.b_better_count + comp.stats.equal_count)
                    as f64)
                * 100.0
        )
        .unwrap();

        #[allow(clippy::cast_precision_loss)]
        writeln!(
            buffer,
            "  {} better: {} deals ({:.1}%)",
            comp.option_b,
            comp.stats.b_better_count,
            (comp.stats.b_better_count as f64
                / (comp.stats.a_better_count + comp.stats.b_better_count + comp.stats.equal_count)
                    as f64)
                * 100.0
        )
        .unwrap();

        #[allow(clippy::cast_precision_loss)]
        writeln!(
            buffer,
            "  Equal:       {} deals ({:.1}%)",
            comp.stats.equal_count,
            (comp.stats.equal_count as f64
                / (comp.stats.a_better_count + comp.stats.b_better_count + comp.stats.equal_count)
                    as f64)
                * 100.0
        )
        .unwrap();

        writeln!(buffer, "\n{}", "Conclusion:".bold()).unwrap();
        if comp.stats.is_a_significantly_better() {
            writeln!(buffer, "  {} is SIGNIFICANTLY BETTER", comp.option_a).unwrap();
        } else if comp.stats.is_b_significantly_better() {
            writeln!(buffer, "  {} is SIGNIFICANTLY BETTER", comp.option_b).unwrap();
        } else {
            writeln!(buffer, "  No significant difference detected").unwrap();
        }

        buffer
    }
}

impl<T: Clone + Eq + std::hash::Hash + fmt::Display> std::fmt::Display
    for PayoffMatrixFormatter<'_, T>
{
    #[inline]
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        // Title
        if self.config.show_title {
            writeln!(fmt, "\n{}", "PAYOFF MATRIX".bold())?;
            writeln!(fmt, "{}", "=".repeat(60))?;
        }

        // Rankings section
        if self.config.show_rankings {
            writeln!(fmt, "\n{}", "Rankings:".bold())?;
            for (idx, ranking) in self.matrix.rankings.iter().enumerate() {
                let rank = idx + 1;
                let marker = if rank == 1 { "★" } else { " " };
                writeln!(
                    fmt,
                    "  {} {}. {} (Avg: {:.2}, W: {}, L: {})",
                    marker,
                    rank,
                    ranking.option,
                    ranking.average_score,
                    ranking.wins,
                    ranking.losses
                )?;
            }
        }

        // Matrix section
        writeln!(fmt, "\n{}", "Pairwise Comparisons:".bold())?;
        writeln!(fmt, "{}", "-".repeat(60))?;

        // Header row
        write!(fmt, "{:12}", "")?;
        for option in &self.matrix.options {
            write!(fmt, "{:>12}", format!("{}", option))?;
        }
        writeln!(fmt)?;
        writeln!(fmt, "{}", "-".repeat(12 + self.matrix.options.len() * 12))?;

        // Precompute a dense lookup so each cell is O(1) instead of scanning
        // every comparison for every (row, column) pair. Build the option ->
        // index map once, then place each comparison directly.
        let n = self.matrix.options.len();
        let mut comparison_grid: Vec<Option<&PayoffComparison<T>>> = vec![None; n * n];
        let mut index_of = std::collections::HashMap::with_capacity(n * 2);
        for (idx, opt) in self.matrix.options.iter().enumerate() {
            // Options are unique; keep the first index on duplicates.
            index_of.entry(opt).or_insert(idx);
        }
        for comparison in &self.matrix.comparisons {
            if let (Some(&a), Some(&b)) = (
                index_of.get(&comparison.option_a),
                index_of.get(&comparison.option_b),
            ) {
                comparison_grid[a * n + b] = Some(comparison);
            }
        }

        // Data rows
        for (i, option_a) in self.matrix.options.iter().enumerate() {
            write!(fmt, "{:<12}", format!("{}", option_a))?;

            for (j, _) in self.matrix.options.iter().enumerate() {
                if i == j {
                    write!(fmt, "{:>12}", "-")?;
                } else {
                    let comparison = comparison_grid[i * n + j].unwrap();

                    let value = format!("{:>8.2}", comparison.stats.mean);

                    if self.config.use_colors {
                        let colored = if comparison.stats.is_a_significantly_better() {
                            value.green()
                        } else if comparison.stats.is_b_significantly_better() {
                            value.red()
                        } else {
                            value.white()
                        };
                        write!(fmt, "{colored:>12}")?;
                    } else {
                        write!(fmt, "{value:>12}")?;
                    }
                }
            }
            writeln!(fmt)?;

            // Standard error row
            if self.config.show_std_errors {
                write!(fmt, "{:12}", "")?;
                for (j, _) in self.matrix.options.iter().enumerate() {
                    if i == j {
                        write!(fmt, "{:>12}", "")?;
                    } else {
                        let comparison = comparison_grid[i * n + j].unwrap();
                        write!(
                            fmt,
                            "{:>12}",
                            format!("(±{:.2})", comparison.stats.std_error)
                        )?;
                    }
                }
                writeln!(fmt)?;
            }
        }

        // Legend
        if self.config.use_colors && self.config.show_legend {
            writeln!(fmt, "\n{}", "Legend:".bold())?;
            writeln!(fmt, "  {} - Option A significantly better", "Green".green())?;
            writeln!(fmt, "  {} - Option B significantly better", "Red".red())?;
            writeln!(fmt, "  {} - No significant difference", "White".white())?;
        }

        Ok(())
    }
}

// ============================================================================
// LEAD ANALYSIS FORMATTER
// ============================================================================

/// Formatter for lead analysis results
pub struct LeadStatsFormatter<'a> {
    analysis: &'a LeadAnalysis,
    config: FormatterConfig,
}

impl<'a> LeadStatsFormatter<'a> {
    /// Create a new formatter with default configuration
    #[must_use]
    #[inline]
    pub fn new(analysis: &'a LeadAnalysis) -> Self {
        Self {
            analysis,
            config: FormatterConfig::default(),
        }
    }

    /// Create with custom configuration
    #[must_use]
    #[inline]
    pub fn with_config(analysis: &'a LeadAnalysis, config: FormatterConfig) -> Self {
        Self { analysis, config }
    }

    /// Print to stdout
    #[inline]
    pub fn print(&self) {
        println!("{self}");
    }

    /// Print suit analysis
    #[inline]
    pub fn print_suit_analysis(&self) {
        println!("{}", self.suit_analysis_string());
    }

    /// Get suit analysis as string
    #[must_use]
    #[inline]
    pub fn suit_analysis_string(&self) -> String {
        let mut buffer = String::with_capacity(1000);

        writeln!(buffer, "\n{}", "SUIT ANALYSIS".bold()).unwrap();
        writeln!(buffer, "{}", "=".repeat(80)).unwrap();

        for analysis in self.analysis.analyze_all_suits() {
            writeln!(
                buffer,
                "\n{} {}:",
                analysis.suit,
                format!(
                    "(Suit avg: {:.2} tricks, {:.2}% set)",
                    analysis.suit_avg_tricks, analysis.suit_avg_set_percentage
                )
                .dimmed()
            )
            .unwrap();

            writeln!(
                buffer,
                "  Best by avg tricks: {} ({:.2})",
                analysis.best_by_avg_tricks.card,
                analysis.best_by_avg_tricks.statistics.mean_dd_tricks()
            )
            .unwrap();
            writeln!(
                buffer,
                "  Best by set %:      {} ({:.2}%)",
                analysis.best_by_set_percentage.card,
                analysis.best_by_set_percentage.statistics.set_percentage
            )
            .unwrap();
        }

        buffer
    }

    /// Print compact summary (one line per card)
    pub fn print_compact(&self) {
        println!("{}", self.compact_string());
    }

    /// Get compact summary as string
    #[must_use]
    #[inline]
    pub fn compact_string(&self) -> String {
        let mut buffer = String::with_capacity(500);

        writeln!(buffer, "Lead Analysis for {}:", self.analysis.contract).unwrap();
        for value in self.analysis.rankings().iter().take(5) {
            writeln!(
                buffer,
                "  {}: {:.2} tricks, {:.2}% set",
                value.card,
                value.statistics.mean_dd_tricks(),
                value.statistics.set_percentage
            )
            .unwrap();
        }

        buffer
    }
}

impl std::fmt::Display for LeadStatsFormatter<'_> {
    #[allow(clippy::cast_precision_loss)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Title
        if self.config.show_title {
            writeln!(fmt, "\n{}", "LEAD ANALYSIS".bold())?;
            writeln!(fmt, "{}", "=".repeat(80))?;
            writeln!(fmt, "Contract: {}", self.analysis.contract)?;
            writeln!(fmt, "Tricks to beat: {}", self.analysis.tricks_to_beat())?;
        }

        // Best leads summary
        writeln!(fmt, "\n{}", "Best Leads:".bold())?;
        match self.analysis.best_by_set_percentage() {
            Some(best) => writeln!(
                fmt,
                "  By set %:     {} ({:.2}% set, {:.2} avg tricks)",
                best.card,
                best.statistics.set_percentage,
                best.statistics.mean_dd_tricks(),
            )?,
            None => writeln!(fmt, "  By set %:     -")?,
        }
        match self.analysis.best_by_avg_tricks() {
            Some(best) => writeln!(
                fmt,
                "  By avg tricks: {} ({:.2} avg tricks, {:.2}% set)",
                best.card,
                best.statistics.mean_dd_tricks(),
                best.statistics.set_percentage,
            )?,
            None => writeln!(fmt, "  By avg tricks: -")?,
        }

        // Detailed table
        writeln!(fmt, "\n{}", "All Leads:".bold())?;
        writeln!(fmt, "{}", "-".repeat(80))?;

        let rankings = self.analysis.rankings();
        let best_avg_card = self.analysis.best_by_avg_tricks().map(|value| value.card);

        // Determine column widths based on data
        let max_tricks = rankings
            .iter()
            .filter_map(|value| {
                value
                    .statistics
                    .dd_tricks
                    .iter()
                    .rposition(|&count| count > 0)
            })
            .max()
            .unwrap_or(13);

        // Header
        write!(fmt, "{:<4} {:>6} {:>6}   ", "Card", "Avg", "%Set")?;
        for tricks in 0..=max_tricks {
            write!(fmt, "{tricks:>4}")?;
        }
        writeln!(fmt)?;
        writeln!(fmt, "{}", "-".repeat(80))?;

        // Data rows
        for (idx, value) in rankings.iter().enumerate() {
            let card_str = format!("{}", value.card);
            let avg = value.statistics.mean_dd_tricks();
            let avg_str = if best_avg_card == Some(value.card) {
                format!("*{avg:<5.2}")
            } else {
                format!("{avg:>6.2}")
            };

            let set_str = format!("{:>6.2}", value.statistics.set_percentage);

            if self.config.use_colors && idx == 0 {
                write!(
                    fmt,
                    "{} {} {}   ",
                    card_str.green(),
                    avg_str.green(),
                    set_str.green()
                )?;
            } else {
                write!(fmt, "{card_str:<4} {avg_str} {set_str}   ")?;
            }

            // Trick distribution
            let deals = value.statistics.deals;
            for tricks in 0..=max_tricks {
                let count = value.statistics.dd_tricks[tricks];
                let percentage = if deals == 0 {
                    0.0
                } else {
                    count as f64 / deals as f64 * 100.0
                };
                if percentage > 0.0 {
                    write!(fmt, "{percentage:>4.0}")?;
                } else {
                    write!(fmt, "{:>4}", "-")?;
                }
            }
            writeln!(fmt)?;
        }

        // Legend
        if self.config.use_colors && self.config.show_legend {
            writeln!(fmt, "\n{}", "Legend:".bold())?;
            writeln!(fmt, "  {} - Best lead by set percentage", "Green".green())?;
            writeln!(fmt, "  * - Best lead by average tricks")?;
            writeln!(
                fmt,
                "  Numbers in columns show % of deals achieving that trick count"
            )?;
        }

        Ok(())
    }
}

// ============================================================================
// FORMATTER CONFIGURATION
// ============================================================================

/// Configuration for formatters
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Show colors in output
    pub use_colors: bool,

    /// Show title/header
    pub show_title: bool,

    /// Show legend explaining colors/symbols
    pub show_legend: bool,

    /// Show standard errors in payoff matrices
    pub show_std_errors: bool,

    /// Show rankings in payoff matrices
    pub show_rankings: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            use_colors: true,
            show_title: true,
            show_legend: true,
            show_std_errors: true,
            show_rankings: true,
        }
    }
}

impl FormatterConfig {
    /// Create a minimal configuration (no colors, titles, or extras)
    #[must_use]
    #[inline]
    pub fn minimal() -> Self {
        Self {
            use_colors: false,
            show_title: false,
            show_legend: false,
            show_std_errors: false,
            show_rankings: false,
        }
    }

    /// Create a configuration suitable for non-interactive output (files, logs)
    #[must_use]
    #[inline]
    pub fn plain() -> Self {
        Self {
            use_colors: false,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::test_support::scripted_lead_analysis;
    use squeezer_cards::{Card, Suit};
    use squeezer_model::{Contract, Doubled, Seat, Strain, Vulnerable};

    fn three_spades() -> Contract {
        Contract::new(
            3,
            Strain::Spades,
            Seat::West,
            Vulnerable::No,
            Doubled::NotDoubled,
        )
    }

    /// Five defender tricks set 3S. All but H3 set; S4 takes the most, so the
    /// ranking is S4, H2, S3, D5, H3.
    fn scripted_analysis() -> LeadAnalysis {
        scripted_lead_analysis(
            three_spades(),
            &[
                (Card::S4, &[7, 7]),
                (Card::S3, &[5, 5]),
                (Card::H2, &[6, 6]),
                (Card::H3, &[3, 3]),
                (Card::D5, &[5, 5]),
            ],
        )
    }

    #[test]
    fn lead_formatter_prints_the_ranked_leads_and_tricks_to_beat() {
        let analysis = scripted_analysis();
        let formatter = LeadStatsFormatter::with_config(&analysis, FormatterConfig::plain());
        let output = formatter.to_string();

        assert!(output.contains("Tricks to beat: 5"));
        assert!(output.contains(&format!(
            "By set %:     {} (100.00% set, 7.00 avg tricks)",
            Card::S4
        )));
        assert!(output.contains(&format!(
            "By avg tricks: {} (7.00 avg tricks, 100.00% set)",
            Card::S4
        )));
        for card in [Card::S4, Card::H2, Card::S3, Card::D5, Card::H3] {
            assert!(
                output.contains(&format!("{card}")),
                "the table must list {card}"
            );
        }
    }

    #[test]
    fn lead_formatter_groups_suit_analysis_without_empty_suits() {
        let analysis = scripted_analysis();
        let formatter = LeadStatsFormatter::with_config(&analysis, FormatterConfig::minimal());
        let output = formatter.suit_analysis_string();

        assert!(output.contains(&format!("Best by avg tricks: {} (7.00)", Card::S4)));
        assert!(output.contains(&format!("Best by set %:      {} (100.00%)", Card::S3)));
        assert!(output.contains("Suit avg: 4.50 tricks, 50.00% set"));
        assert!(
            !output.contains(&format!("{} (Suit avg", Suit::Clubs)),
            "a suit with no legal lead has no section"
        );
    }

    #[test]
    fn lead_formatter_compact_summary_follows_the_ranking() {
        let analysis = scripted_analysis();
        let formatter = LeadStatsFormatter::with_config(&analysis, FormatterConfig::minimal());
        let output = formatter.compact_string();

        assert!(output.starts_with(&format!("Lead Analysis for {}:", three_spades())));
        assert!(output.contains(&format!("  {}: 7.00 tricks, 100.00% set", Card::S4)));
    }
}
