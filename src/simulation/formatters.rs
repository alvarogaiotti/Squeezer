// Copyright (C) 2025 Alvaro Gaiotti
// See end of file for license information

//! # Terminal Formatters
//!
//! This module provides formatters for displaying simulation results in the terminal.
//! These are completely separate from the core simulation logic, making it easy to
//! create custom output formats or skip terminal output entirely in a GUI context.
//!
//! ## Example
//!
//! ```
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use squeezer::prelude::*;
//! use squeezer::simulation::formatters::*;
//!
//! let sim = ContractPayoffSimulation::builder()
//!     .num_deals(1000)
//!     .dealer(StandardDealer::new())
//!     .contracts(vec![
//!         Contract::from_str("3CN")?,
//!         Contract::from_str("3NN")?,
//!     ])
//!     .scoring_function(imps)
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

use colored::Colorize;
use std::fmt::{self, Write as FmtWrite};

use crate::{Comparable, LeadStatistics, PayoffComparison, PayoffMatrix, Suit};

// ============================================================================
// PAYOFF MATRIX FORMATTER
// ============================================================================

/// Formatter for payoff matrices (contract or card comparisons)
pub struct PayoffMatrixFormatter<'a, T: Comparable + fmt::Display> {
    matrix: &'a PayoffMatrix<T>,
    config: FormatterConfig,
}

impl<'a, T: Comparable + fmt::Display> PayoffMatrixFormatter<'a, T> {
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

impl<T: Comparable + fmt::Display> std::fmt::Display for PayoffMatrixFormatter<'_, T> {
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

        // Data rows
        for (i, option_a) in self.matrix.options.iter().enumerate() {
            write!(fmt, "{:<12}", format!("{}", option_a))?;

            for (j, option_b) in self.matrix.options.iter().enumerate() {
                if i == j {
                    write!(fmt, "{:>12}", "-")?;
                } else {
                    let comparison = self
                        .matrix
                        .comparisons
                        .iter()
                        .find(|c| &c.option_a == option_a && &c.option_b == option_b)
                        .unwrap();

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
                for (j, option_b) in self.matrix.options.iter().enumerate() {
                    if i == j {
                        write!(fmt, "{:>12}", "")?;
                    } else {
                        let comparison = self
                            .matrix
                            .comparisons
                            .iter()
                            .find(|c| &c.option_a == option_a && &c.option_b == option_b)
                            .unwrap();
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
// LEAD STATISTICS FORMATTER
// ============================================================================

/// Formatter for lead simulation results
pub struct LeadStatsFormatter<'a> {
    stats: &'a LeadStatistics,
    config: FormatterConfig,
}

impl<'a> LeadStatsFormatter<'a> {
    /// Create a new formatter with default configuration
    #[must_use]
    #[inline]
    pub fn new(stats: &'a LeadStatistics) -> Self {
        Self {
            stats,
            config: FormatterConfig::default(),
        }
    }

    /// Create with custom configuration
    #[must_use]
    #[inline]
    pub fn with_config(stats: &'a LeadStatistics, config: FormatterConfig) -> Self {
        Self { stats, config }
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

        for suit in &[Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
            if let Some(analysis) = self.stats.analyze_suit(*suit) {
                writeln!(
                    buffer,
                    "\n{} {}:",
                    suit,
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
                    analysis.best_by_avg_tricks.card, analysis.best_by_avg_tricks.average_tricks
                )
                .unwrap();
                writeln!(
                    buffer,
                    "  Best by set %:      {} ({:.2}%)",
                    analysis.best_by_set_percentage.card,
                    analysis.best_by_set_percentage.set_percentage
                )
                .unwrap();
            }
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

        writeln!(buffer, "Lead Analysis for {}:", self.stats.contract).unwrap();
        for card_stat in self.stats.cards.iter().take(5) {
            writeln!(
                buffer,
                "  {}: {:.2} tricks, {:.2}% set",
                card_stat.card, card_stat.average_tricks, card_stat.set_percentage
            )
            .unwrap();
        }

        buffer
    }
}

impl std::fmt::Display for LeadStatsFormatter<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Title
        if self.config.show_title {
            writeln!(fmt, "\n{}", "LEAD ANALYSIS".bold())?;
            writeln!(fmt, "{}", "=".repeat(80))?;
            writeln!(fmt, "Contract: {}", self.stats.contract)?;
            writeln!(fmt, "Tricks to beat: {}", self.stats.tricks_to_beat)?;
        }

        // Best leads summary
        writeln!(fmt, "\n{}", "Best Leads:".bold())?;
        writeln!(
            fmt,
            "  By set %:     {} ({:.2}% set, {:.2} avg tricks)",
            self.stats.best_by_set_percentage.card,
            self.stats.best_by_set_percentage.set_percentage,
            self.stats.best_by_set_percentage.average_tricks,
        )?;
        writeln!(
            fmt,
            "  By avg tricks: {} ({:.2} avg tricks, {:.2}% set)",
            self.stats.best_by_avg_tricks.card,
            self.stats.best_by_avg_tricks.average_tricks,
            self.stats.best_by_avg_tricks.set_percentage,
        )?;

        // Detailed table
        writeln!(fmt, "\n{}", "All Leads:".bold())?;
        writeln!(fmt, "{}", "-".repeat(80))?;

        // Determine column widths based on data
        let max_tricks = self
            .stats
            .cards
            .iter()
            .map(|c| c.max_tricks)
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
        for (idx, card_stat) in self.stats.cards.iter().enumerate() {
            let card_str = format!("{}", card_stat.card);
            let avg_str = if idx
                == self
                    .stats
                    .cards
                    .iter()
                    .position(|c| c.card == self.stats.best_by_avg_tricks.card)
                    .unwrap_or(usize::MAX)
            {
                format!("*{:<5.2}", card_stat.average_tricks)
            } else {
                format!("{:>6.2}", card_stat.average_tricks)
            };

            let set_str = format!("{:>6.2}", card_stat.set_percentage);

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
            let dist_map = card_stat
                .distribution_percentages
                .iter()
                .enumerate()
                .map(|(t, p)| (t, *p))
                .collect::<std::collections::HashMap<_, _>>();

            for tricks in 0..=max_tricks as usize {
                let percentage = dist_map.get(&tricks).unwrap_or(&0.0);
                if *percentage > 0.0 {
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

// ============================================================================
// CONVENIENCE TRAIT FOR FORMATTED OUTPUT
// ============================================================================

/// Trait for types that can be formatted for terminal output
pub trait TerminalFormattable {
    /// Print a formatted report to stdout
    fn print_report(&self);

    /// Get formatted report as a string
    fn format_report(&self) -> String;
}

impl<T: Comparable + fmt::Display> TerminalFormattable for PayoffMatrix<T> {
    fn print_report(&self) {
        PayoffMatrixFormatter::new(self).print();
    }

    fn format_report(&self) -> String {
        PayoffMatrixFormatter::new(self).to_string()
    }
}

impl TerminalFormattable for LeadStatistics {
    fn print_report(&self) {
        LeadStatsFormatter::new(self).print();
    }

    fn format_report(&self) -> String {
        LeadStatsFormatter::new(self).to_string()
    }
}

// ============================================================================
// EXPORT UTILITIES
// ============================================================================

/// Utilities for exporting results to various formats
pub mod export {
    use super::{Comparable, LeadStatistics, PayoffMatrix, fmt};
    use std::io::Write;

    /// Export payoff matrix to CSV format
    /// # Errors
    /// Errors on IO errors: unable to open file, etc..
    pub fn payoff_to_csv<T: Comparable + fmt::Display>(
        matrix: &PayoffMatrix<T>,
        writer: &mut dyn Write,
    ) -> std::io::Result<()> {
        // Header row
        writeln!(
            writer,
            "Option_A,Option_B,Mean,StdDev,StdError,Min,Max,Q1,Median,Q3,A_Better,B_Better,Equal"
        )?;

        // Data rows
        for comp in &matrix.comparisons {
            writeln!(
                writer,
                "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                comp.option_a,
                comp.option_b,
                comp.stats.mean,
                comp.stats.std_dev,
                comp.stats.std_error,
                comp.stats.min,
                comp.stats.max,
                comp.stats.q1,
                comp.stats.median,
                comp.stats.q3,
                comp.stats.a_better_count,
                comp.stats.b_better_count,
                comp.stats.equal_count,
            )?;
        }

        Ok(())
    }

    /// Export lead statistics to CSV format
    /// # Errors
    /// Errors on IO errors: unable to open file, etc..
    pub fn lead_stats_to_csv(
        stats: &LeadStatistics,
        writer: &mut dyn Write,
    ) -> std::io::Result<()> {
        // Header row
        writeln!(
            writer,
            "Card,AvgTricks,SetPercentage,StdDev,MinTricks,MaxTricks,MedianTricks"
        )?;

        // Data rows
        for card_stat in &stats.cards {
            writeln!(
                writer,
                "{},{},{},{},{},{},{}",
                card_stat.card,
                card_stat.average_tricks,
                card_stat.set_percentage,
                card_stat.std_dev,
                card_stat.min_tricks,
                card_stat.max_tricks,
                card_stat.median_tricks,
            )?;
        }

        Ok(())
    }

    /// Export lead distribution to CSV format (for detailed analysis)
    /// # Errors
    /// Errors on IO errors: unable to open file, etc..
    pub fn lead_distribution_to_csv(
        stats: &LeadStatistics,
        writer: &mut dyn Write,
    ) -> std::io::Result<()> {
        // Find max tricks to determine columns
        let max_tricks = stats.cards.iter().map(|c| c.max_tricks).max().unwrap_or(13);

        // Header row
        write!(writer, "Card")?;
        for tricks in 0..=max_tricks {
            write!(writer, ",{tricks}_tricks")?;
        }
        writeln!(writer)?;

        // Data rows
        for card_stat in &stats.cards {
            write!(writer, "{}", card_stat.card)?;

            let dist_map: std::collections::HashMap<_, _> = card_stat
                .distribution_percentages
                .iter()
                .enumerate()
                .map(|(t, p)| (t, *p))
                .collect();

            for tricks in 0..=max_tricks as usize {
                let percentage = dist_map.get(&tricks).unwrap_or(&0.0);
                write!(writer, ",{percentage:.2}")?;
            }
            writeln!(writer)?;
        }

        Ok(())
    }
}

// ============================================================================
// CHART DATA PREPARATION
// ============================================================================

/// Utilities for preparing data for charts/graphs in a GUI
pub mod charts {
    use super::{Comparable, LeadStatistics, PayoffComparison, PayoffMatrix, fmt};

    /// Data prepared for a histogram chart
    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct HistogramData {
        pub label: String,
        pub bins: Vec<(f64, f64, usize)>, // (start, end, count)
    }

    /// Data prepared for a box plot
    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct BoxPlotData {
        pub label: String,
        pub min: f64,
        pub q1: f64,
        pub median: f64,
        pub q3: f64,
        pub max: f64,
    }

    /// Data prepared for a comparison bar chart
    #[derive(Debug, Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct BarChartData {
        pub labels: Vec<String>,
        pub values: Vec<f64>,
        pub errors: Option<Vec<f64>>, // Error bars
    }

    /// Prepare histogram data from a payoff comparison
    pub fn payoff_comparison_histogram<T: Comparable + fmt::Display>(
        comp: &PayoffComparison<T>,
    ) -> HistogramData {
        HistogramData {
            label: format!("{} vs {}", comp.option_a, comp.option_b),
            bins: comp.distribution.bins.clone(),
        }
    }

    /// Prepare box plot data from a payoff comparison
    pub fn payoff_comparison_boxplot<T: Comparable + fmt::Display>(
        comp: &PayoffComparison<T>,
    ) -> BoxPlotData {
        BoxPlotData {
            label: format!("{} vs {}", comp.option_a, comp.option_b),
            min: f64::from(comp.stats.min),
            q1: comp.stats.q1,
            median: comp.stats.median,
            q3: comp.stats.q3,
            max: f64::from(comp.stats.max),
        }
    }

    /// Prepare bar chart data for payoff matrix rankings
    #[must_use]
    #[inline]
    pub fn payoff_rankings_barchart<T: Comparable + fmt::Display>(
        matrix: &PayoffMatrix<T>,
    ) -> BarChartData {
        BarChartData {
            labels: matrix
                .rankings
                .iter()
                .map(|r| format!("{}", r.option))
                .collect(),
            values: matrix.rankings.iter().map(|r| r.average_score).collect(),
            errors: None,
        }
    }

    /// Prepare bar chart data for lead statistics
    #[must_use]
    #[inline]
    pub fn lead_stats_barchart(stats: &LeadStatistics) -> Vec<BarChartData> {
        vec![
            BarChartData {
                labels: stats.cards.iter().map(|c| format!("{}", c.card)).collect(),
                values: stats.cards.iter().map(|c| c.average_tricks).collect(),
                errors: Some(stats.cards.iter().map(|c| c.std_dev).collect()),
            },
            BarChartData {
                labels: stats.cards.iter().map(|c| format!("{}", c.card)).collect(),
                values: stats.cards.iter().map(|c| c.set_percentage).collect(),
                errors: None,
            },
        ]
    }

    /// Prepare stacked bar chart data for lead trick distributions
    #[must_use]
    #[inline]
    pub fn lead_distribution_stackedbars(stats: &LeadStatistics) -> Vec<BarChartData> {
        let max_tricks = stats.cards.iter().map(|c| c.max_tricks).max().unwrap_or(13);

        let mut result = Vec::new();

        for tricks in 0..=max_tricks {
            let mut values = Vec::new();

            for card_stat in &stats.cards {
                let percentage = card_stat
                    .distribution_percentages
                    .iter()
                    .enumerate()
                    .find(|&(t, _)| t == tricks as usize)
                    .map_or(0.0, |(_, p)| *p);
                values.push(percentage);
            }

            result.push(BarChartData {
                labels: stats.cards.iter().map(|c| format!("{}", c.card)).collect(),
                values,
                errors: None,
            });
        }

        result
    }
}
