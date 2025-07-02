//! What is the set of n ingredients that will allow you to make the highest number of different cocktails?
//! E.g.:
//! You have 117 different ingredients
//! You have 104 cocktails, each of which use 2-6 ingredients
//! Which 5 ingredients maximize the cocktail-making possibilities? What about 10 ingredients?
//! Here's a branch and bound solution
//! Original here: https://gist.github.com/tmcw/c6bdcfe505057ed6a0f356cfd02d4d52
use rand::rngs::ThreadRng;
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;
use std::{cmp::Ordering, collections::BTreeSet};

mod bitset;
pub use bitset::BitSet;

pub mod bounds;
use bounds::{BoundContext, BoundFunction, ConcentrationBound, SingletonBound, TotalBound};

/// Efficient checker for forbidden cocktails
pub struct ForbiddenChecker {
    forbidden_masks: SmallVec<[BitSet; 8]>,
}

impl ForbiddenChecker {
    fn new() -> Self {
        ForbiddenChecker {
            forbidden_masks: SmallVec::new(),
        }
    }

    fn with_base(base: &ForbiddenChecker, additional: BitSet) -> Self {
        let mut forbidden_masks = SmallVec::with_capacity(base.forbidden_masks.len() + 1);
        forbidden_masks.extend(base.forbidden_masks.iter().cloned());
        forbidden_masks.push(additional);
        ForbiddenChecker { forbidden_masks }
    }

    #[inline]
    fn is_forbidden(&self, ingredients: &BitSet) -> bool {
        self.forbidden_masks
            .iter()
            .any(|forbidden| forbidden.is_subset(ingredients))
    }
}

pub type Ingredient = String;
pub type IngredientSet = BTreeSet<Ingredient>;

pub type Ingredienti = i32;
pub type IngredientSeti = BitSet;

pub struct BranchBound {
    pub calls: i32,
    pub max_size: usize,
    pub highest_score: usize,
    pub highest: Vec<IngredientSeti>,
    pub highest_ingredients: BitSet,
    pub random: ThreadRng,
    pub counter: u32,
    pub min_cover: FxHashMap<BitSet, i32>,
    pub min_amortized_cost: FxHashMap<IngredientSeti, f64>,
    pub initial: bool,
    // Cache for frequently used operations
    pub all_cocktails: Vec<IngredientSeti>,
    pub cocktail_indices: FxHashMap<IngredientSeti, usize>,
    // Configurable bound functions
    pub bound_functions: Vec<Box<dyn BoundFunction>>,
}

/// This will obviously explode on NaN values
fn cmp_f64(a: f64, b: f64) -> Ordering {
    if a < b {
        return Ordering::Less;
    } else if a > b {
        return Ordering::Greater;
    }
    Ordering::Equal
}

pub struct BranchBoundBuilder {
    max_calls: i32,
    max_size: usize,
    bound_functions: Vec<Box<dyn BoundFunction>>,
}

impl BranchBoundBuilder {
    pub fn new(max_calls: i32, max_size: usize) -> Self {
        BranchBoundBuilder {
            max_calls,
            max_size,
            bound_functions: Vec::new(),
        }
    }

    pub fn with_bound(mut self, bound: Box<dyn BoundFunction>) -> Self {
        self.bound_functions.push(bound);
        self
    }

    pub fn with_default_bounds(self) -> Self {
        self.with_bound(Box::new(TotalBound))
            .with_bound(Box::new(SingletonBound))
            .with_bound(Box::new(ConcentrationBound))
    }

    pub fn build(self) -> BranchBound {
        let bounds = if self.bound_functions.is_empty() {
            // If no bounds specified, use defaults
            vec![
                Box::new(TotalBound) as Box<dyn BoundFunction>,
                Box::new(SingletonBound) as Box<dyn BoundFunction>,
                Box::new(ConcentrationBound) as Box<dyn BoundFunction>,
            ]
        } else {
            self.bound_functions
        };

        BranchBound {
            calls: self.max_calls,
            max_size: self.max_size,
            highest_score: 0usize,
            highest: Vec::new(),
            highest_ingredients: BitSet::new(),
            random: rand::thread_rng(),
            counter: 0,
            min_cover: FxHashMap::default(),
            min_amortized_cost: FxHashMap::default(),
            initial: true,
            all_cocktails: Vec::new(),
            cocktail_indices: FxHashMap::default(),
            bound_functions: bounds,
        }
    }
}

impl BranchBound {
    #[must_use]
    pub fn new(max_calls: i32, max_size: usize) -> BranchBound {
        BranchBoundBuilder::new(max_calls, max_size).build()
    }

    #[inline(always)]
    pub fn search(
        &mut self,
        candidates: &mut FxHashSet<IngredientSeti>,
        partial: &mut FxHashSet<IngredientSeti>,
        forbidden: &mut Option<ForbiddenChecker>,
    ) -> Vec<IngredientSeti> {
        // first run-through, so populate min_cover, amortized cost and cocktail cardinality
        // this SHOULD be a great use of Option, but it's actually such a pain to work with
        if self.initial {
            *forbidden = Some(ForbiddenChecker::new());

            // Cache all cocktails for index-based access
            self.all_cocktails = candidates.iter().cloned().collect();
            for (idx, cocktail) in self.all_cocktails.iter().enumerate() {
                self.cocktail_indices.insert(cocktail.clone(), idx);
            }

            let mut cardinality = FxHashMap::default();
            for cocktail in candidates.iter() {
                for ingredient in cocktail.iter() {
                    *cardinality.entry(ingredient as i32).or_insert(0) += 1;
                }
            }

            // we can calculate the minimum amortized cost for each cocktail:
            // if we were to have enough
            // ingredients to make all the cocktails, how much should
            // we pay, in ingredient-cost, for each cocktail. For
            // example, if a cocktail has a unique ingredient, and two
            // other ingredients shared by one other cocktail, then the
            // amortized cost would be 1/1 + 1/2 + 1/2 = 2
            //
            // The minimum amortized cost is a lower bound on how much
            // we will ever pay in ingredient cost for a cocktail.
            for cocktail in candidates.iter() {
                self.min_amortized_cost.insert(
                    cocktail.clone(),
                    cocktail
                        .iter()
                        .map(|ingredient| {
                            1f64 / f64::from(*cardinality.get(&(ingredient as i32)).unwrap())
                        })
                        .sum::<f64>(),
                );
                self.min_cover.insert(
                    cocktail.clone(),
                    cocktail
                        .iter()
                        .map(|ingredient| *cardinality.get(&(ingredient as i32)).unwrap())
                        .min()
                        .unwrap(),
                );
            }
            self.initial = false;
        }
        // begin
        if self.calls <= 0 {
            println!("{:?}", "Early return!");
            return self.highest.clone();
        }
        self.calls -= 1;
        self.counter += 1;
        let score = partial.len();

        if score > self.highest_score {
            self.highest = partial.iter().cloned().collect();
            self.highest_score = score;
        }

        // what cocktails could be added without blowing our ingredient budget?
        // this will be empty on the first iteration
        let mut partial_ingredients = BitSet::new();
        for cocktail in partial.iter() {
            partial_ingredients.union_assign(cocktail);
        }
        let keep_exploring = self.keep_exploring(candidates, partial, &partial_ingredients);

        if keep_exploring {
            // new best heuristic: pick the candidate cocktail
            // which is the "least unique" in its ingredient list
            let best = candidates
                .iter()
                .min_by(|a, b| {
                    cmp_f64(
                        *self.min_amortized_cost.get(a).unwrap(),
                        *self.min_amortized_cost.get(b).unwrap(),
                    )
                })
                .unwrap()
                .clone();
            let new_partial_ingredients = &partial_ingredients | &best;
            let mut covered_candidates =
                FxHashSet::with_capacity_and_hasher(candidates.len() / 2, Default::default());
            let mut permitted_candidates =
                FxHashSet::with_capacity_and_hasher(candidates.len(), Default::default());

            for cocktail in candidates.iter() {
                if cocktail.is_subset(&new_partial_ingredients) {
                    covered_candidates.insert(cocktail.clone());
                } else {
                    let extended_ingredients = cocktail | &new_partial_ingredients;
                    if extended_ingredients.len() <= self.max_size {
                        let forbidden_cover = forbidden
                            .as_ref()
                            .unwrap()
                            .is_forbidden(&extended_ingredients);
                        if !forbidden_cover {
                            permitted_candidates.insert(cocktail.clone());
                        }
                    }
                }
            }

            let mut new_partial = partial.clone();
            new_partial.extend(covered_candidates.iter().cloned());

            self.search(&mut permitted_candidates, &mut new_partial, forbidden);

            let mut remaining = FxHashSet::default();
            for cocktail in candidates.iter() {
                if cocktail != &best {
                    let test = cocktail | &partial_ingredients;
                    if !best.is_subset(&test) {
                        remaining.insert(cocktail.clone());
                    }
                }
            }
            let new_forbidden = ForbiddenChecker::with_base(forbidden.as_ref().unwrap(), best);

            self.search(&mut remaining, partial, &mut Some(new_forbidden));
        }
        // search() called from inner loop instances will return to the callee at this point
        // once those are exhausted, the final set will return to the caller
        self.highest.clone()
    }

    fn keep_exploring(
        &self,
        candidates: &mut FxHashSet<IngredientSeti>,
        partial: &mut FxHashSet<IngredientSeti>,
        partial_ingredients: &IngredientSeti,
    ) -> bool {
        let threshold = (self.highest_score - partial.len()) as i32;

        let context = BoundContext {
            candidates,
            partial,
            partial_ingredients,
            max_size: self.max_size,
            min_cover: &self.min_cover,
            min_amortized_cost: &self.min_amortized_cost,
        };

        // Use iterator methods for cleaner, more functional approach
        self.bound_functions
            .iter()
            .all(|bound| bound.compute(&context) > threshold)
    }
}
