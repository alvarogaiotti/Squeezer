// Phase 1: Basic Rule Engine Implementation
// This shows the core structure for parsing rules and applying constraints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;

use crate::Strain;

#[derive(Debug, Clone, PartialEq)]
pub struct Bid {
    pub level: u8,
    pub suit: Strain,
    pub is_pass: bool,
    pub is_double: bool,
    pub is_redouble: bool,
}

#[derive(Debug, Clone)]
pub struct HandConstraints {
    pub hcp: Range<u8>,
    pub spades: Range<u8>,
    pub hearts: Range<u8>,
    pub diamonds: Range<u8>,
    pub clubs: Range<u8>,
    pub balanced: Option<bool>, // Some(true) = must be balanced, None = unknown
}

impl HandConstraints {
    pub fn new_unrestricted() -> Self {
        Self {
            hcp: 0..40,
            spades: 0..14,
            hearts: 0..14,
            diamonds: 0..14,
            clubs: 0..14,
            balanced: None,
        }
    }

    pub fn intersect(&mut self, other: &HandConstraints) -> Result<(), String> {
        // Helper function to intersect ranges
        fn intersect_range(a: &Range<u8>, b: &Range<u8>) -> Result<Range<u8>, String> {
            let start = a.start.max(b.start);
            let end = a.end.min(b.end);
            if start >= end {
                return Err("No valid intersection".to_string());
            }
            Ok(start..end)
        }

        self.hcp = intersect_range(&self.hcp, &other.hcp)?;
        self.spades = intersect_range(&self.spades, &other.spades)?;
        self.hearts = intersect_range(&self.hearts, &other.hearts)?;
        self.diamonds = intersect_range(&self.diamonds, &other.diamonds)?;
        self.clubs = intersect_range(&self.clubs, &other.clubs)?;

        // Handle balanced constraint
        match (self.balanced, other.balanced) {
            (Some(a), Some(b)) if a != b => {
                return Err("Conflicting balanced constraints".to_string())
            }
            (None, Some(b)) => self.balanced = Some(b),
            (Some(_), None) => {} // Keep existing constraint
            _ => {}
        }

        Ok(())
    }
}

// Rule definition structures
#[derive(Debug, Deserialize, Serialize)]
pub struct SuitConstraint {
    pub suit: String, // "spades", "hearts", etc.
    pub min: Option<u8>,
    pub max: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BidRule {
    pub bid: String,  // "1NT", "1S", "2C", etc.
    pub seat: String, // "opener", "responder", "any"
    pub description: String,
    pub hcp_min: Option<u8>,
    pub hcp_max: Option<u8>,
    pub suit_constraints: Vec<SuitConstraint>,
    pub balanced: Option<bool>,
    pub forcing: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RuleSet {
    pub name: String,
    pub rules: Vec<BidRule>,
}

// Basic rule engine
pub struct RuleEngine {
    rules: HashMap<String, Vec<BidRule>>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn load_from_yaml(&mut self, yaml_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let rule_set: RuleSet = serde_yaml::from_str(yaml_content)?;

        for rule in rule_set.rules {
            let bid_key = rule.bid.clone();
            self.rules
                .entry(bid_key)
                .or_insert_with(Vec::new)
                .push(rule);
        }

        Ok(())
    }

    pub fn find_rules_for_bid(&self, bid: &Bid, seat: &str) -> Vec<&BidRule> {
        let bid_string = self.bid_to_string(bid);

        self.rules
            .get(&bid_string)
            .map(|rules| {
                rules
                    .iter()
                    .filter(|rule| rule.seat == "any" || rule.seat == seat)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn bid_to_string(&self, bid: &Bid) -> String {
        if bid.is_pass {
            return "Pass".to_string();
        }
        if bid.is_double {
            return "Double".to_string();
        }
        if bid.is_redouble {
            return "Redouble".to_string();
        }

        let suit_str = match bid.suit {
            Strain::Clubs => "C",
            Strain::Diamonds => "D",
            Strain::Hearts => "H",
            Strain::Spades => "S",
            Strain::NoTrumps => "NT",
        };

        format!("{}{}", bid.level, suit_str)
    }
}

// Constraint applicator - converts rules to constraints
pub struct ConstraintApplicator;

impl ConstraintApplicator {
    pub fn rule_to_constraints(&self, rule: &BidRule) -> HandConstraints {
        let mut constraints = HandConstraints::new_unrestricted();

        // Apply HCP constraints
        if let (Some(min), Some(max)) = (rule.hcp_min, rule.hcp_max) {
            constraints.hcp = min..max + 1;
        } else if let Some(min) = rule.hcp_min {
            constraints.hcp = min..40;
        } else if let Some(max) = rule.hcp_max {
            constraints.hcp = 0..max + 1;
        }

        // Apply suit constraints
        for suit_constraint in &rule.suit_constraints {
            let range = match (suit_constraint.min, suit_constraint.max) {
                (Some(min), Some(max)) => min..max + 1,
                (Some(min), None) => min..14,
                (None, Some(max)) => 0..max + 1,
                (None, None) => continue,
            };

            match suit_constraint.suit.as_str() {
                "spades" => constraints.spades = range,
                "hearts" => constraints.hearts = range,
                "diamonds" => constraints.diamonds = range,
                "clubs" => constraints.clubs = range,
                _ => eprintln!("Unknown suit: {}", suit_constraint.suit),
            }
        }

        // Apply balanced constraint
        constraints.balanced = rule.balanced;

        constraints
    }
}

// Basic inference engine for Phase 1
pub struct BasicInferenceEngine {
    rule_engine: RuleEngine,
    constraint_applicator: ConstraintApplicator,
    hand_constraints: [HandConstraints; 4], // N, E, S, W
}

impl BasicInferenceEngine {
    pub fn new() -> Self {
        Self {
            rule_engine: RuleEngine::new(),
            constraint_applicator: ConstraintApplicator,
            hand_constraints: [
                HandConstraints::new_unrestricted(),
                HandConstraints::new_unrestricted(),
                HandConstraints::new_unrestricted(),
                HandConstraints::new_unrestricted(),
            ],
        }
    }

    pub fn load_rules(&mut self, yaml_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.rule_engine.load_from_yaml(yaml_content)
    }

    pub fn apply_bid(&mut self, bid: &Bid, seat_index: usize) -> Result<(), String> {
        let seat_name = match seat_index {
            0 => "opener",    // North opens first in our model
            1 => "responder", // East responds
            2 => "responder", // South responds
            3 => "responder", // West responds
            _ => return Err("Invalid seat index".to_string()),
        };

        let applicable_rules = self.rule_engine.find_rules_for_bid(bid, seat_name);

        if applicable_rules.is_empty() {
            return Err(format!("No rules found for bid: {:?}", bid));
        }

        // For Phase 1, we'll use the first matching rule
        // In later phases, we'll handle multiple rules and ambiguity
        let rule = applicable_rules[0];
        let new_constraints = self.constraint_applicator.rule_to_constraints(rule);

        // Apply constraints to the hand
        self.hand_constraints[seat_index].intersect(&new_constraints)?;

        Ok(())
    }

    pub fn get_constraints(&self, seat_index: usize) -> Option<&HandConstraints> {
        self.hand_constraints.get(seat_index)
    }

    pub fn reset(&mut self) {
        self.hand_constraints = [
            HandConstraints::new_unrestricted(),
            HandConstraints::new_unrestricted(),
            HandConstraints::new_unrestricted(),
            HandConstraints::new_unrestricted(),
        ];
    }
}

// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rule_application() {
        let mut engine = BasicInferenceEngine::new();

        // Load some basic rules
        let rules = r#"
name: "Basic 2/1"
rules:
  - bid: "1NT"
    seat: "opener"
    description: "15-17 HCP, balanced"
    hcp_min: 15
    hcp_max: 17
    balanced: true
    suit_constraints:
      - suit: "spades"
        min: 2
        max: 5
      - suit: "hearts"
        min: 2
        max: 5
  - bid: "1S"
    seat: "opener"
    description: "12+ HCP, 5+ spades"
    hcp_min: 12
    suit_constraints:
      - suit: "spades"
        min: 5
"#;

        engine.load_rules(rules).unwrap();

        // Test 1NT opening
        let one_nt = Bid {
            level: 1,
            suit: Strain::NoTrumps,
            is_pass: false,
            is_double: false,
            is_redouble: false,
        };

        engine.apply_bid(&one_nt, 0).unwrap();
        let constraints = engine.get_constraints(0).unwrap();

        assert_eq!(constraints.hcp, 15..18);
        assert_eq!(constraints.spades, 2..6);
        assert_eq!(constraints.hearts, 2..6);
        assert_eq!(constraints.balanced, Some(true));
    }
}

// Example rule file structure for Phase 1
const EXAMPLE_RULES: &str = r#"
name: "Basic 2/1 Opening Bids"
rules:
  - bid: "1C"
    seat: "opener"
    description: "12+ HCP, 3+ clubs (may be short)"
    hcp_min: 12
    suit_constraints:
      - suit: "clubs"
        min: 3

  - bid: "1D"
    seat: "opener"
    description: "12+ HCP, 4+ diamonds"
    hcp_min: 12
    suit_constraints:
      - suit: "diamonds"
        min: 4

  - bid: "1H"
    seat: "opener"
    description: "12+ HCP, 5+ hearts"
    hcp_min: 12
    suit_constraints:
      - suit: "hearts"
        min: 5

  - bid: "1S"
    seat: "opener"
    description: "12+ HCP, 5+ spades"
    hcp_min: 12
    suit_constraints:
      - suit: "spades"
        min: 5

  - bid: "1NT"
    seat: "opener"
    description: "15-17 HCP, balanced"
    hcp_min: 15
    hcp_max: 17
    balanced: true
    suit_constraints:
      - suit: "spades"
        min: 2
        max: 5
      - suit: "hearts"
        min: 2
        max: 5
      - suit: "diamonds"
        min: 2
        max: 5
      - suit: "clubs"
        min: 2
        max: 5

  - bid: "2NT"
    seat: "opener"
    description: "20-21 HCP, balanced"
    hcp_min: 20
    hcp_max: 21
    balanced: true
"#;
