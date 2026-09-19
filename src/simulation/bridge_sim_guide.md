# Bridge Simulation API - Complete Redesign Documentation

## Table of Contents

1. [Overview](#overview)
2. [Key Design Changes](#key-design-changes)
3. [Architecture](#architecture)
4. [Migration Guide](#migration-guide)
5. [Usage Examples](#usage-examples)
6. [API Reference](#api-reference)

---

## Overview

This redesign completely restructures the Bridge Monte Carlo simulation API to be GUI-friendly while maintaining backwards compatibility where possible. The core principle is **separation of concerns**: raw data collection, statistical computation, and presentation are now completely independent.

### Goals Achieved

✅ **Raw data access** - All individual deal results are accessible for custom visualization
✅ **Computed statistics** - Pre-calculated metrics ready for display
✅ **GUI-friendly** - No terminal output in core types
✅ **Flexible consumption** - Multiple ways to access and use results
✅ **Type safety** - Clear, documented interfaces
✅ **Extensibility** - Easy to add new analysis types

---

## Key Design Changes

### 1. Simulation Output Structure

**Before:**
```rust
// Old API returned results directly
let payoff: Payoff<Contract> = simulation.run()?;
payoff.report(); // Prints to terminal
// No way to access raw data
```

**After:**
```rust
// New API returns structured output
let output: SimulationOutput<RawPayoffData<Contract>, PayoffMatrix<Contract>> = simulation.run()?;

// Access raw data
for item in &output.raw.items {
    for comp in &item.comparisons {
        // Individual deal-by-deal results
        println!("Deal {}: diff = {}", comp.deal_index, comp.difference);
    }
}

// Access computed statistics
println!("Mean: {}", output.statistics.comparisons[0].stats.mean);

// Terminal output is separate
use squeezer::simulation::formatters::PayoffMatrixFormatter;
PayoffMatrixFormatter::new(&output.statistics).print();
```

### 2. Builder Pattern for Configuration

**Before:**
```rust
let sim = PayoffSimulation::new(1000, dealer, contracts, scoring_fn);
```

**After:**
```rust
let sim = ContractPayoffSimulation::builder()
    .num_deals(1000)
    .dealer(dealer)
    .contracts(contracts)
    .scoring_function(scoring_fn)
    .build()?;
```

### 3. Separation of Formatters

**Before:**
- Terminal output baked into `SimulationResult::report()`
- Can't customize output without modifying core types

**After:**
- Formatters are completely separate in `formatters` module
- Core types have no display logic
- Easy to create custom formatters for any output format

### 4. Enhanced Statistics

**Before:**
```rust
// Only mean and standard deviation
struct Payoff {
    results: Vec<(f64, f64)>, // (mean, stderr)
}
```

**After:**
```rust
// Comprehensive statistics
struct PairwiseStatistics {
    mean: f64,
    std_dev: f64,
    std_error: f64,
    min: i32,
    max: i32,
    q1: f64,
    median: f64,
    q3: f64,
    a_better_count: usize,
    b_better_count: usize,
    equal_count: usize,
}
```

---

## Architecture

### Module Structure

```
simulation/
├── core.rs              # Core traits and data structures
│   ├── Simulation trait
│   ├── SimulationOutput
│   ├── DealResult
│   ├── ComparisonResult
│   ├── PairwiseStatistics
│   └── DistributionData
│
├── payoff.rs            # Contract and card payoff simulations
│   ├── ContractPayoffSimulation
│   ├── CardPayoffSimulation
│   ├── RawPayoffData
│   └── PayoffMatrix
│
├── lead.rs              # Opening lead simulations
│   ├── LeadSimulation
│   ├── RawLeadData
│   └── LeadStatistics
│
└── formatters.rs        # Terminal output formatters
    ├── PayoffMatrixFormatter
    ├── LeadStatsFormatter
    └── export utilities
```

### Data Flow

```
         ┌─────────────────┐
         │   Simulation    │
         │   (configured)  │
         └────────┬────────┘
                  │
                  │ .run()
                  ▼
┌─────────────────────────────────────┐
│      SimulationOutput               │
├─────────────────────────────────────┤
│  ┌─────────────┐  ┌──────────────┐  │
│  │  Raw Data   │  │  Statistics  │  │
│  │             │  │              │  │
│  │ - Individual│  │ - Means      │  │
│  │   deals     │  │ - Std devs   │  │
│  │ - Scores    │  │ - Percentiles│  │
│  │ - Tricks    │  │ - Rankings   │  │
│  └─────────────┘  └──────────────┘  │
│                                     │
│        ┌──────────────┐             │
│        │   Metadata   │             │
│        └──────────────┘             │
└─────────────────────────────────────┘
                  │
┌─────────────┬───┴──────────┬────────────┐
▼             ▼              ▼            ▼
GUI/Chart    CSV Export    JSON Export   Terminal
```

---

## Migration Guide

### Contract Payoff Simulation

**OLD CODE:**
```rust
use squeezer::prelude::*;

let contracts = vec![
    Contract::from_str("3CN")?,
    Contract::from_str("3NN")?,
];

let sim = PayoffSimulation::new(1000, dealer, contracts, imps);
let payoff = sim.run()?;
payoff.report(); // Prints to terminal
```

**NEW CODE:**
```rust
use squeezer::prelude::*;
use squeezer::simulation::payoff::*;
use squeezer::simulation::formatters::*;

let contracts = vec![
    Contract::from_str("3CN")?,
    Contract::from_str("3NN")?,
];

// Build simulation
let sim = ContractPayoffSimulation::builder()
    .num_deals(1000)
    .dealer(dealer)
    .contracts(contracts)
    .scoring_function(imps)
    .build()?;

// Run simulation
let results = sim.run()?;

// For terminal output (optional)
PayoffMatrixFormatter::new(&results.statistics).print();

// For GUI: access raw data
for item in &results.raw.items {
    println!("Comparing {} vs {}", item.option_a, item.option_b);
    // Plot histogram of differences
    let diffs = item.differences();
    plot_histogram(diffs);
}

// For GUI: use computed statistics
for comp in &results.statistics.comparisons {
    display_comparison(
        &comp.option_a,
        &comp.option_b,
        comp.stats.mean,
        comp.stats.std_error,
    );
}
```

### Card Payoff Simulation

**OLD CODE:**
```rust
let cards = vec![Card::new(Suit::Spades, 14), Card::new(Suit::Hearts, 13)];
let sim = CardPayoffSimulation::new(
    1000,
    dealer,
    cards,
    contract,
    imps,
    Seat::North,
    Cards::EMPTY,
)?;
let payoff = sim.run()?;
payoff.report();
```

**NEW CODE:**
```rust
let cards = vec![Card::new(Suit::Spades, 14), Card::new(Suit::Hearts, 13)];

let sim = CardPayoffSimulation::builder()
    .num_deals(1000)
    .dealer(dealer)
    .cards(cards)
    .contract(contract)
    .scoring_function(imps)
    .seat(Seat::North)
    .play_trace(Cards::EMPTY)
    .build()?;

let results = sim.run()?;

// Access individual card performance
for opt_result in &results.raw.option_results {
    let card = opt_result.option;
    let avg_score = opt_result.deals.iter()
        .map(|d| d.score)
        .sum::<i32>() as f64 / opt_result.deals.len() as f64;

    println!("{}: avg score = {:.2}", card, avg_score);
}
```

### Lead Simulation

**OLD CODE:**
```rust
let sim = LeadSimulation::new(1000, dealer, Contract::from_str("5CN")?);
let results = sim.run()?;
results.report();
```

**NEW CODE:**
```rust
let sim = LeadSimulation::builder()
    .num_deals(1000)
    .dealer(dealer)
    .contract(Contract::from_str("5CN")?)
    .with_deal_details() // Optional: store per-deal information
    .build()?;

let results = sim.run()?;

// For terminal output
LeadStatsFormatter::new(&results.statistics).print();

// For GUI: access raw data
for card_result in &results.raw.cards {
    println!("Card: {}", card_result.card);

    // Get histogram data for plotting
    let histogram = card_result.tricks_histogram();
    plot_bar_chart(histogram);

    // Get cumulative distribution
    let cumulative = card_result.cumulative_distribution();
    plot_line_chart(cumulative);
}

// For GUI: use computed statistics
let best = &results.statistics.best_by_set_percentage;
display_best_lead(
    best.card,
    best.average_tricks,
    best.set_percentage,
);
```

---

## Usage Examples

### Example 1: Basic Contract Comparison for GUI

```rust
use squeezer::prelude::*;
use squeezer::simulation::payoff::*;

fn compare_contracts_for_gui(
    dealer: StandardDealer,
    contracts: Vec<Contract>,
) -> Result<GuiData, SqueezerError> {
    let sim = ContractPayoffSimulation::builder()
        .num_deals(10000)
        .dealer(dealer)
        .contracts(contracts)
        .scoring_function(imps)
        .build()?;

    let results = sim.run()?;

    // Extract data for GUI
    let mut gui_data = GuiData::new();

    // Rankings for display
    for (rank, ranking) in results.statistics.rankings.iter().enumerate() {
        gui_data.add_ranking(
            rank + 1,
            ranking.option.to_string(),
            ranking.average_score,
            ranking.wins,
            ranking.losses,
        );
    }

    // Detailed comparisons for drill-down
    for comp in &results.statistics.comparisons {
        gui_data.add_comparison(
            comp.option_a.to_string(),
            comp.option_b.to_string(),
            comp.stats.clone(),
            comp.distribution.clone(),
        );
    }

    Ok(gui_data)
}
```

### Example 2: Lead Analysis with Visualization Data

```rust
use squeezer::prelude::*;
use squeezer::simulation::lead::*;

fn analyze_leads_for_chart(
    dealer: StandardDealer,
    contract: Contract,
) -> Result<ChartData, SqueezerError> {
    let sim = LeadSimulation::builder()
        .num_deals(5000)
        .dealer(dealer)
        .contract(contract)
        .with_deal_details()
        .build()?;

    let results = sim.run()?;

    let mut chart_data = ChartData::new();

    // Bar chart: average tricks per card
    for card_stat in &results.statistics.cards {
        chart_data.add_bar(
            card_stat.card.to_string(),
            card_stat.average_tricks,
            card_stat.std_dev,
        );
    }

    // Heatmap: trick distribution
    for card_result in &results.raw.cards {
        let histogram = card_result.tricks_histogram();
        chart_data.add_distribution(
            card_result.card.to_string(),
            histogram,
        );
    }

    // Suit analysis for grouped display
    for suit_analysis in results.statistics.analyze_all_suits() {
        chart_data.add_suit_summary(
            suit_analysis.suit,
            suit_analysis.suit_avg_tricks,
            suit_analysis.suit_avg_set_percentage,
        );
    }

    Ok(chart_data)
}
```

### Example 3: Exporting Results to CSV

```rust
use squeezer::simulation::formatters::export::*;
use std::fs::File;

fn export_simulation_results(
    results: &SimulationOutput<RawLeadData, LeadStatistics>,
    base_filename: &str,
) -> std::io::Result<()> {
    // Export summary statistics
    let mut stats_file = File::create(format!("{}_stats.csv", base_filename))?;
    lead_stats_to_csv(&results.statistics, &mut stats_file)?;

    // Export detailed distribution
    let mut dist_file = File::create(format!("{}_distribution.csv", base_filename))?;
    lead_distribution_to_csv(&results.statistics, &mut dist_file)?;

    Ok(())
}
```

### Example 4: JSON Serialization (with serde feature)

```rust
#[cfg(feature = "serde")]
use serde_json;

fn serialize_results_to_json(
    results: &SimulationOutput<RawPayoffData<Contract>, PayoffMatrix<Contract>>,
) -> Result<String, serde_json::Error> {
    // Serialize everything
    let json = serde_json::to_string_pretty(results)?;
    Ok(json)
}

fn serialize_stats_only(
    stats: &PayoffMatrix<Contract>,
) -> Result<String, serde_json::Error> {
    // Just the statistics for API response
    let json = serde_json::to_string(stats)?;
    Ok(json)
}
```

### Example 5: Custom Analysis

```rust
use squeezer::simulation::core::*;

fn custom_analysis(raw: &RawPayoffData<Contract>) -> CustomMetrics {
    let mut metrics = CustomMetrics::new();

    // Analyze volatility
    for item in &raw.items {
        let differences = item.differences();
        let variance = calculate_variance(&differences);
        metrics.add_volatility(
            item.option_a.to_string(),
            item.option_b.to_string(),
            variance,
        );
    }

    // Find extreme outcomes
    for opt_result in &raw.option_results {
        let worst_score = opt_result.deals.iter()
            .map(|d| d.score)
            .min()
            .unwrap_or(0);
        metrics.add_worst_case(
            opt_result.option.to_string(),
            worst_score,
        );
    }

    metrics
}
```

---

## API Reference

### Core Types

#### `SimulationOutput<R, S>`

Container for simulation results.

**Fields:**
- `raw: R` - Raw simulation data
- `statistics: S` - Computed statistics
- `metadata: SimulationMetadata` - Simulation metadata

**Methods:**
- `into_raw(self) -> R` - Extract raw data
- `into_statistics(self) -> S` - Extract statistics
- `into_parts(self) -> (R, S, SimulationMetadata)` - Extract all components

#### `PairwiseStatistics`

Comprehensive statistics for comparing two options.

**Fields:**
- `mean: f64` - Mean difference
- `std_dev: f64` - Standard deviation
- `std_error: f64` - Standard error of the mean
- `min: i32` - Minimum difference
- `max: i32` - Maximum difference
- `q1: f64` - 25th percentile
- `median: f64` - Median
- `q3: f64` - 75th percentile
- `a_better_count: usize` - Deals where option A was better
- `b_better_count: usize` - Deals where option B was better
- `equal_count: usize` - Deals where they were equal

**Methods:**
- `is_a_significantly_better() -> bool`
- `is_b_significantly_better() -> bool`
- `are_equivalent() -> bool`

### Payoff Simulations

#### `ContractPayoffSimulation::builder()`

Create a builder for contract payoff simulations.

**Builder Methods:**
- `.num_deals(usize)` - Set number of deals
- `.dealer(D)` - Set dealer
- `.contracts(Vec<Contract>)` - Set contracts to compare
- `.add_contract(Contract)` - Add a single contract
- `.scoring_function(F)` - Set scoring function (imps, matchpoints, etc.)
- `.build()` - Build the simulation

#### `CardPayoffSimulation::builder()`

Create a builder for card payoff simulations.

**Builder Methods:**
- `.num_deals(usize)` - Set number of deals
- `.dealer(D)` - Set dealer
- `.cards(Vec<Card>)` - Set cards to compare
- `.add_card(Card)` - Add a single card
- `.contract(Contract)` - Set the contract
- `.scoring_function(F)` - Set scoring function
- `.seat(Seat)` - Set the seat playing
- `.play_trace(Cards)` - Set cards already played
- `.build()` - Build the simulation

#### `PayoffMatrix<T>`

Computed statistics for all pairwise comparisons.

**Fields:**
- `options: Vec<T>` - Options being compared
- `comparisons: Vec<PayoffComparison<T>>` - All pairwise comparisons
- `rankings: Vec<OptionRanking<T>>` - Overall rankings

**Methods:**
- `get_stats(&T, &T) -> Option<&PairwiseStatistics>` - Get stats for a comparison
- `rank_of(&T) -> Option<usize>` - Get ranking of an option
- `best_option() -> Option<&T>` - Get best performing option
- `sorted_options() -> Vec<&T>` - Get options sorted by performance

### Lead Simulations

#### `LeadSimulation::builder()`

Create a builder for lead simulations.

**Builder Methods:**
- `.num_deals(usize)` - Set number of deals
- `.dealer(D)` - Set dealer
- `.contract(Contract)` - Set the contract
- `.with_deal_details()` - Store per-deal information (uses more memory)
- `.build()` - Build the simulation

#### `LeadStatistics`

Computed statistics for all lead cards.

**Fields:**
- `cards: Vec<CardLeadStatistics>` - Statistics for each card (sorted by set %)
- `best_by_avg_tricks: CardLeadStatistics` - Best card by average tricks
- `best_by_set_percentage: CardLeadStatistics` - Best card by set percentage
- `contract: Contract` - The contract analyzed
- `tricks_to_beat: u8` - Tricks needed to beat contract

**Methods:**
- `stats_for(Card) -> Option<&CardLeadStatistics>` - Get stats for a card
- `rank_by_set_percentage(Card) -> Option<usize>` - Get ranking of a card
- `cards_by_suit() -> HashMap<Suit, Vec<&CardLeadStatistics>>` - Group by suit
- `analyze_suit(Suit) -> Option<SuitLeadAnalysis>` - Analyze specific suit
- `analyze_all_suits() -> Vec<SuitLeadAnalysis>` - Analyze all suits
- `compare_cards(Card, Card) -> Option<LeadComparison>` - Compare two cards

#### `CardLeadStatistics`

Statistics for a single lead card.

**Fields:**
- `card: Card`
- `average_tricks: f64`
- `set_percentage: f64`
- `std_dev: f64`
- `min_tricks: u8`
- `max_tricks: u8`
- `median_tricks: f64`
- `distribution_percentages: Vec<(u8, f64)>` - Percentage of deals for each trick count

### Formatters

#### `PayoffMatrixFormatter`

Format payoff matrices for terminal output.

**Methods:**
- `new(&PayoffMatrix) -> Self` - Create with default config
- `with_config(&PayoffMatrix, FormatterConfig) -> Self` - Create with custom config
- `print_detailed_comparison(&T, &T)` - Print detailed comparison
- `detailed_comparison_string(&T, &T) -> String` - Get detailed comparison as string

#### `LeadStatsFormatter`

Format lead statistics for terminal output.

**Methods:**
- `new(&LeadStatistics) -> Self` - Create with default config
- `with_config(&LeadStatistics, FormatterConfig) -> Self` - Create with custom config
- `print()` - Print to stdout
- `to_string() -> String`
- `print_suit_analysis()` - Print analysis by suit
- `suit_analysis_string() -> String`
- `print_compact()` - Print compact one-line-per-card summary
- `compact_string() -> String`

#### `FormatterConfig`

Configuration for formatters.

**Fields:**
- `use_colors: bool` - Enable terminal colors
- `show_title: bool` - Show title/header
- `show_legend: bool` - Show legend
- `show_std_errors: bool` - Show standard errors (payoff matrices)
- `show_rankings: bool` - Show rankings (payoff matrices)

**Constructors:**
- `default()` - All features enabled with colors
- `minimal()` - No colors, titles, or extras
- `plain()` - All features but no colors (for file output)

### Export Utilities

#### `export::payoff_to_csv()`

Export payoff matrix to CSV.

```rust
pub fn payoff_to_csv<T: Clone + Eq + std::hash::Hash + Display>(
    matrix: &PayoffMatrix<T>,
    writer: &mut dyn Write,
) -> std::io::Result<()>
```

#### `export::lead_stats_to_csv()`

Export lead statistics to CSV.

```rust
pub fn lead_stats_to_csv(
    stats: &LeadStatistics,
    writer: &mut dyn Write,
) -> std::io::Result<()>
```

#### `export::lead_distribution_to_csv()`

Export detailed trick distribution to CSV.

```rust
pub fn lead_distribution_to_csv(
    stats: &LeadStatistics,
    writer: &mut dyn Write,
) -> std::io::Result<()>
```

### Chart Utilities

The `charts` module provides utilities for preparing data for visualization.

#### Types

- `HistogramData` - Data for histogram charts
- `BoxPlotData` - Data for box plots
- `BarChartData` - Data for bar charts

#### Functions

- `payoff_comparison_histogram()` - Histogram from payoff comparison
- `payoff_comparison_boxplot()` - Box plot from payoff comparison
- `payoff_rankings_barchart()` - Bar chart of rankings
- `lead_stats_barchart()` - Bar charts for lead statistics
- `lead_distribution_stackedbars()` - Stacked bars for trick distributions

---

## Advanced Usage Patterns

### Pattern 1: Streaming Results for Large Simulations

For very large simulations, you may want to process results incrementally to avoid memory pressure. While the current API computes everything upfront, you can work with raw data in chunks:

```rust
fn process_large_simulation(results: SimulationOutput<RawPayoffData<Contract>, PayoffMatrix<Contract>>) {
    // Process raw data in batches
    const BATCH_SIZE: usize = 1000;

    for item in &results.raw.items {
        for chunk in item.comparisons.chunks(BATCH_SIZE) {
            // Process this batch
            let batch_stats = compute_batch_statistics(chunk);
            send_to_frontend(batch_stats);
        }
    }
}
```

### Pattern 2: Progressive Statistics Updates

Update your UI as statistics are computed:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn simulation_with_progress_callback<F>(
    sim: ContractPayoffSimulation<StandardDealer>,
    callback: F,
) -> Result<SimulationOutput<RawPayoffData<Contract>, PayoffMatrix<Contract>>, SqueezerError>
where
    F: Fn(f64) + Send + 'static,
{
    let progress = Arc::new(Mutex::new(0.0));
    let progress_clone = progress.clone();

    // Spawn thread to monitor progress
    let handle = thread::spawn(move || {
        loop {
            let current = *progress_clone.lock().unwrap();
            callback(current);
            if current >= 100.0 {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    // Run simulation (would need internal progress tracking)
    let results = sim.run()?;

    *progress.lock().unwrap() = 100.0;
    handle.join().unwrap();

    Ok(results)
}
```

### Pattern 3: Caching and Persistence

Save simulation results for later analysis:

```rust
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn save_results<R, S>(
    results: &SimulationOutput<R, S>,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: Serialize,
    S: Serialize,
{
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, results)?;
    Ok(())
}

fn load_results<R, S>(
    path: &str,
) -> Result<SimulationOutput<R, S>, Box<dyn std::error::Error>>
where
    R: for<'de> Deserialize<'de>,
    S: for<'de> Deserialize<'de>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let results = serde_json::from_reader(reader)?;
    Ok(results)
}
```

### Pattern 4: Comparative Analysis

Compare results from multiple simulations:

```rust
fn compare_simulations(
    results1: &LeadStatistics,
    results2: &LeadStatistics,
) -> ComparisonReport {
    let mut report = ComparisonReport::new();

    // Compare best leads
    if results1.best_by_set_percentage.card != results2.best_by_set_percentage.card {
        report.add_difference(
            "best_lead_changed",
            format!("{} -> {}",
                results1.best_by_set_percentage.card,
                results2.best_by_set_percentage.card
            ),
        );
    }

    // Compare all cards that appear in both
    for card_stat1 in &results1.cards {
        if let Some(card_stat2) = results2.stats_for(card_stat1.card) {
            let diff = card_stat1.average_tricks - card_stat2.average_tricks;
            if diff.abs() > 0.1 {
                report.add_difference(
                    "card_performance",
                    format!("{}: {:.2} vs {:.2} (diff: {:.2})",
                        card_stat1.card,
                        card_stat1.average_tricks,
                        card_stat2.average_tricks,
                        diff
                    ),
                );
            }
        }
    }

    report
}
```

### Pattern 5: Real-time Web Dashboard

Serve simulation results via a web API:

```rust
use actix_web::{web, App, HttpResponse, HttpServer};

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

async fn run_simulation_endpoint(
    params: web::Json<SimulationParams>,
) -> HttpResponse {
    // Run simulation
    let sim = ContractPayoffSimulation::builder()
        .num_deals(params.num_deals)
        .dealer(params.dealer.clone())
        .contracts(params.contracts.clone())
        .scoring_function(imps)
        .build();

    match sim {
        Ok(sim) => match sim.run() {
            Ok(results) => {
                // Return only statistics to client
                HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(results.statistics),
                    error: None,
                })
            }
            Err(e) => HttpResponse::InternalServerError().json(ApiResponse {
                success: false,
                data: None::<PayoffMatrix<Contract>>,
                error: Some(e.to_string()),
            }),
        },
        Err(e) => HttpResponse::BadRequest().json(ApiResponse {
            success: false,
            data: None::<PayoffMatrix<Contract>>,
            error: Some(e.to_string()),
        }),
    }
}

async fn get_comparison_endpoint(
    path: web::Path<(String, String)>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let (option_a, option_b) = path.into_inner();

    // Retrieve cached results
    if let Some(results) = data.get_cached_results() {
        let contract_a = Contract::from_str(&option_a).unwrap();
        let contract_b = Contract::from_str(&option_b).unwrap();

        if let Some(stats) = results.statistics.get_stats(&contract_a, &contract_b) {
            return HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(stats),
                error: None,
            });
        }
    }

    HttpResponse::NotFound().json(ApiResponse {
        success: false,
        data: None::<PairwiseStatistics>,
        error: Some("Comparison not found".to_string()),
    })
}
```

---

## Performance Considerations

### Memory Usage

The redesigned API provides flexibility in memory/detail tradeoff:

```rust
// Minimal memory: no deal details
let sim = LeadSimulation::builder()
    .num_deals(100000)
    .dealer(dealer)
    .contract(contract)
    // Don't call .with_deal_details()
    .build()?;

// More memory: store deal details for analysis
let sim = LeadSimulation::builder()
    .num_deals(10000)
    .dealer(dealer)
    .contract(contract)
    .with_deal_details() // Stores every deal result
    .build()?;
```

### Computational Efficiency

The new design doesn't change the simulation engine, which remains highly optimized:

- **Parallel DDS solving** - Multiple deals solved simultaneously
- **Batch processing** - Efficient use of DDS solver API
- **Welford's algorithm** - Single-pass statistical computation
- **Smart chunking** - Optimal batch sizes for solver

### When to Use What

| Use Case | Best Approach |
|----------|---------------|
| Quick CLI tool | Use formatters directly |
| Web dashboard | Serialize statistics to JSON |
| Data analysis | Export raw data to CSV |
| Mobile app | Use statistics, create custom formatters |
| Research | Store complete raw data with deal details |
| Real-time updates | Process statistics incrementally |

---

## Extending the API

### Adding Custom Statistics

Create your own statistical computations:

```rust
pub struct CustomStatistics {
    pub skewness: f64,
    pub kurtosis: f64,
    pub confidence_interval_95: (f64, f64),
}

impl CustomStatistics {
    pub fn from_comparisons(comparisons: &[ComparisonResult]) -> Self {
        let diffs: Vec<f64> = comparisons.iter()
            .map(|c| c.difference as f64)
            .collect();

        let skewness = calculate_skewness(&diffs);
        let kurtosis = calculate_kurtosis(&diffs);
        let ci = calculate_confidence_interval(&diffs, 0.95);

        Self {
            skewness,
            kurtosis,
            confidence_interval_95: ci,
        }
    }
}
```

### Creating Custom Formatters

Implement your own output format:

```rust
pub struct JsonFormatter<'a, T: Clone + Eq + std::hash::Hash> {
    matrix: &'a PayoffMatrix<T>,
}

impl<'a, T: Clone + Eq + std::hash::Hash + Serialize> JsonFormatter<'a, T> {
    pub fn new(matrix: &'a PayoffMatrix<T>) -> Self {
        Self { matrix }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct JsonOutput<'b, U: Clone + Eq + std::hash::Hash> {
            rankings: &'b [OptionRanking<U>],
            comparisons: Vec<JsonComparison>,
        }

        #[derive(Serialize)]
        struct JsonComparison {
            option_a: String,
            option_b: String,
            mean: f64,
            conclusion: String,
        }

        let comparisons = self.matrix.comparisons.iter()
            .map(|c| JsonComparison {
                option_a: format!("{}", c.option_a),
                option_b: format!("{}", c.option_b),
                mean: c.stats.mean,
                conclusion: if c.stats.is_a_significantly_better() {
                    "A is better".to_string()
                } else if c.stats.is_b_significantly_better() {
                    "B is better".to_string()
                } else {
                    "No significant difference".to_string()
                },
            })
            .collect();

        let output = JsonOutput {
            rankings: &self.matrix.rankings,
            comparisons,
        };

        serde_json::to_string_pretty(&output)
    }
}
```

### Adding New Simulation Types

Implement the `Simulation` trait:

```rust
pub struct DefenseSimulation<D: Dealer> {
    num_deals: usize,
    dealer: D,
    contract: Contract,
    // ... other fields
}

impl<D: Dealer> Simulation for DefenseSimulation<D> {
    type RawResult = RawDefenseData;
    type Statistics = DefenseStatistics;

    fn run(self) -> Result<SimulationOutput<Self::RawResult, Self::Statistics>, SqueezerError> {
        // Implement simulation logic
        todo!()
    }
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairwise_statistics() {
        let comparisons = vec![
            ComparisonResult::new(0, 100, 90, 10),
            ComparisonResult::new(1, 95, 100, -5),
            ComparisonResult::new(2, 110, 95, 15),
        ];

        let stats = PairwiseStatistics::from_comparisons(&comparisons);

        assert!((stats.mean - 6.67).abs() < 0.01);
        assert_eq!(stats.a_better_count, 2);
        assert_eq!(stats.b_better_count, 1);
    }

    #[test]
    fn test_payoff_matrix_rankings() {
        // Create test data
        let contracts = vec![
            Contract::from_str("3CN").unwrap(),
            Contract::from_str("3NN").unwrap(),
        ];

        let raw_data = create_test_raw_data(contracts);
        let matrix = PayoffMatrix::from_raw_data(&raw_data);

        assert!(matrix.best_option().is_some());
        assert_eq!(matrix.rankings.len(), 2);
    }
}
```

### Integration Tests

```rust
#[test]
fn test_full_contract_payoff_simulation() {
    let dealer = StandardDealer::default();
    let contracts = vec![
        Contract::from_str("3CN").unwrap(),
        Contract::from_str("3NN").unwrap(),
    ];

    let sim = ContractPayoffSimulation::builder()
        .num_deals(100)
        .dealer(dealer)
        .contracts(contracts)
        .scoring_function(imps)
        .build()
        .unwrap();

    let results = sim.run().unwrap();

    // Verify structure
    assert_eq!(results.raw.options.len(), 2);
    assert_eq!(results.statistics.rankings.len(), 2);
    assert_eq!(results.metadata.num_deals, 100);

    // Verify data consistency
    for item in &results.raw.items {
        assert_eq!(item.comparisons.len(), 100);
    }
}
```

---

## Troubleshooting

### Common Issues

#### Issue: "Builder field not set"

```rust
// ❌ Error: missing num_deals
let sim = ContractPayoffSimulation::builder()
    .dealer(dealer)
    .contracts(contracts)
    .build()?;

// ✅ Solution: set all required fields
let sim = ContractPayoffSimulation::builder()
    .num_deals(1000)  // Add this
    .dealer(dealer)
    .contracts(contracts)
    .scoring_function(imps)  // And this
    .build()?;
```

#### Issue: "Card not in hand"

```rust
// ❌ Error: card not in specified hand
let sim = CardPayoffSimulation::builder()
    .cards(vec![Card::new(Suit::Spades, 14)])
    .seat(Seat::North)
    // ... but North doesn't have the ♠A
    .build()?;

// ✅ Solution: ensure dealer pre-deals correct hands
let mut dealer = StandardDealer::default();
dealer.predeal_hand(Seat::North, hand_with_spade_ace);
```

#### Issue: "Out of memory"

```rust
// ❌ Too much memory with deal details
let sim = LeadSimulation::builder()
    .num_deals(1_000_000)
    .with_deal_details()  // Stores 1M deals in memory
    .build()?;

// ✅ Solution: don't store deal details for large sims
let sim = LeadSimulation::builder()
    .num_deals(1_000_000)
    // Omit .with_deal_details()
    .build()?;
```

---

## Changelog from Old API

### Breaking Changes

1. **Return types changed**: `Simulation::run()` now returns `SimulationOutput<R, S>` instead of direct result type
2. **Report method removed**: Use formatters instead of `.report()` method
3. **Constructors changed**: Prefer builders over direct constructors
4. **Statistics structure**: Expanded from `(f64, f64)` to `PairwiseStatistics`

### New Features

1. **Raw data access**: Complete access to individual deal results
2. **Builder pattern**: Fluent API for configuration
3. **Metadata**: Automatic tracking of simulation parameters
4. **Extended statistics**: Percentiles, min/max, win/loss counts
5. **Export utilities**: CSV, JSON serialization
6. **Chart preparation**: Helpers for visualization
7. **Suit analysis**: Grouped analysis for lead simulations
8. **Comparison utilities**: Direct card-to-card comparisons

### Backwards Compatibility

Legacy constructors are still available:

```rust
// Still works (deprecated)
let sim = PayoffSimulation::new(1000, dealer, contracts, imps);

// Equivalent new API
let sim = ContractPayoffSimulation::new(1000, dealer, contracts, imps);
```

To get old-style terminal output:

```rust
// Old API
payoff.report();

// New API
use squeezer::simulation::formatters::*;
PayoffMatrixFormatter::new(&results.statistics).print();
```

---

## Best Practices

### 1. Use Builders for Clarity

```rust
// ✅ Clear and self-documenting
let sim = ContractPayoffSimulation::builder()
    .num_deals(10000)
    .dealer(dealer)
    .contracts(contracts)
    .scoring_function(imps)
    .build()?;
```

### 2. Separate Concerns

```rust
// ✅ Simulation, analysis, and presentation are separate
let results = sim.run()?;
let custom_stats = my_analysis(&results.raw);
let formatted = MyFormatter::new(&custom_stats).to_string();
```

### 3. Handle Errors Properly

```rust
// ✅ Proper error handling
let sim = ContractPayoffSimulation::builder()
    .num_deals(1000)
    .dealer(dealer)
    .contracts(contracts)
    .scoring_function(imps)
    .build()
    .map_err(|e| {
        log::error!("Failed to build simulation: {}", e);
        e
    })?;
```

### 4. Choose Appropriate Detail Level

```rust
// For production GUI: no deal details
let sim = LeadSimulation::builder()
    .num_deals(100000)
    .dealer(dealer)
    .contract(contract)
    .build()?;

// For debugging/research: with deal details
let sim = LeadSimulation::builder()
    .num_deals(1000)
    .dealer(dealer)
    .contract(contract)
    .with_deal_details()
    .build()?;
```

### 5. Cache Results When Appropriate

```rust
// ✅ Save expensive computations
let results = sim.run()?;
save_results(&results, "cache/simulation_results.json")?;

// Later...
let results = load_results("cache/simulation_results.json")?;
```

---

## Conclusion

This redesigned API provides a clean separation between simulation logic, data storage, statistical computation, and presentation. It's designed to be:

- **Flexible**: Use raw data or computed statistics as needed
- **Extensible**: Easy to add new analysis types and formatters
- **Efficient**: No unnecessary computation or storage
- **Type-safe**: Clear interfaces and comprehensive documentation
- **GUI-friendly**: Structured data perfect for visualization

The modular design means you can use exactly what you need - from simple terminal output to complex interactive dashboards - without modifying core simulation code.
