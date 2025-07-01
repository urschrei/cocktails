//! Modular bound functions for the branch and bound algorithm

use crate::{BitSet, IngredientSeti};
use rustc_hash::{FxHashMap, FxHashSet};

/// Context containing all data needed by bound functions
pub struct BoundContext<'a> {
    pub candidates: &'a FxHashSet<IngredientSeti>,
    pub partial: &'a FxHashSet<IngredientSeti>,
    pub partial_ingredients: &'a IngredientSeti,
    pub max_size: usize,
    pub min_cover: &'a FxHashMap<BitSet, i32>,
    pub min_amortized_cost: &'a FxHashMap<IngredientSeti, f64>,
}

/// Trait for implementing bound functions
pub trait BoundFunction: Send + Sync {
    /// Compute the bound given the current context
    fn compute(&self, context: &BoundContext) -> i32;

    /// Name of this bound function for debugging/logging
    fn name(&self) -> &'static str;
}

/// Total bound: returns the number of candidate cocktails
pub struct TotalBound;

impl BoundFunction for TotalBound {
    fn compute(&self, context: &BoundContext) -> i32 {
        context.candidates.len() as i32
    }

    fn name(&self) -> &'static str {
        "TotalBound"
    }
}

/// Singleton bound: accounts for cocktails with unique ingredients
pub struct SingletonBound;

impl BoundFunction for SingletonBound {
    fn compute(&self, context: &BoundContext) -> i32 {
        let n_unique_cocktails = context
            .candidates
            .iter()
            .filter(|cocktail| context.min_cover.get(cocktail).unwrap() == &1)
            .count();
        let ingredient_budget = context.max_size - context.partial_ingredients.len();
        context.candidates.len() as i32 - n_unique_cocktails as i32
            + (n_unique_cocktails.min(ingredient_budget) as i32)
    }

    fn name(&self) -> &'static str {
        "SingletonBound"
    }
}

/// Concentration bound: considers best-case ingredient distribution
pub struct ConcentrationBound;

impl BoundFunction for ConcentrationBound {
    fn compute(&self, context: &BoundContext) -> i32 {
        let mut candidate_ingredients = BitSet::new();
        for cocktail in context.candidates.iter() {
            candidate_ingredients = candidate_ingredients | cocktail;
        }
        let mut excess_ingredients = (candidate_ingredients | context.partial_ingredients).len()
            as i32
            - context.max_size as i32;

        // Use a small array on the stack for common cases
        const STACK_SIZE: usize = 128;
        let mut stack_increases = [0i32; STACK_SIZE];
        let mut increases_count = 0;

        for cocktail in context.candidates.iter() {
            if increases_count < STACK_SIZE {
                stack_increases[increases_count] =
                    (cocktail - context.partial_ingredients).len() as i32;
                increases_count += 1;
            }
        }

        // Sort only the used portion
        stack_increases[..increases_count].sort_unstable_by(|a, b| b.cmp(a));

        let mut upper_increment = context.candidates.len();
        for &ingredient_increase in &stack_increases[..increases_count] {
            if excess_ingredients <= 0 {
                break;
            }
            upper_increment -= 1;
            excess_ingredients -= ingredient_increase;
        }
        upper_increment as i32
    }

    fn name(&self) -> &'static str {
        "ConcentrationBound"
    }
}
