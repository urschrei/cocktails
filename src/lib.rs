//! What is the set of n ingredients that will allow you to make the highest number of different cocktails?
//! E.g.:
//! You have 117 different ingredients
//! You have 104 cocktails, each of which use 2-6 ingredients
//! Which 5 ingredients maximize the cocktail-making possibilities? What about 10 ingredients?
//! Here's a branch and bound solution
//! Original here: https://gist.github.com/tmcw/c6bdcfe505057ed6a0f356cfd02d4d52
use rand::rngs::ThreadRng;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{cmp::Ordering, collections::BTreeSet};

mod bitset;
pub use bitset::BitSet;

pub type Ingredient = String;
pub type IngredientSet = BTreeSet<Ingredient>;

pub type Ingredienti = i32;
pub type IngredientSeti = BitSet;

#[derive(Debug)]
pub struct BranchBound {
    pub calls: i32,
    pub max_size: usize,
    pub highest_score: usize,
    pub highest: FxHashSet<IngredientSeti>,
    pub highest_ingredients: BitSet,
    pub random: ThreadRng,
    pub counter: u32,
    pub min_cover: FxHashMap<BitSet, i32>,
    pub min_amortized_cost: FxHashMap<IngredientSeti, f64>,
    pub initial: bool,
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

impl BranchBound {
    #[must_use]
    pub fn new(max_calls: i32, max_size: usize) -> BranchBound {
        BranchBound {
            calls: max_calls,
            max_size,
            highest_score: 0usize,
            highest: FxHashSet::default(),
            highest_ingredients: BitSet::new(),
            random: rand::thread_rng(),
            counter: 0,
            min_cover: FxHashMap::default(),
            min_amortized_cost: FxHashMap::default(),
            initial: true,
        }
    }

    #[inline(always)]
    pub fn search(
        &mut self,
        candidates: &mut FxHashSet<IngredientSeti>,
        partial: &mut FxHashSet<IngredientSeti>,
        forbidden: &mut Option<FxHashSet<IngredientSeti>>,
    ) -> FxHashSet<IngredientSeti> {
        // first run-through, so populate min_cover, amortized cost and cocktail cardinality
        // this SHOULD be a great use of Option, but it's actually such a pain to work with
        if self.initial {
            *forbidden = Some(FxHashSet::default());
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
                    *cocktail,
                    cocktail
                        .iter()
                        .map(|ingredient| {
                            1f64 / f64::from(*cardinality.get(&(ingredient as i32)).unwrap())
                        })
                        .sum::<f64>(),
                );
                self.min_cover.insert(
                    *cocktail,
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
            self.highest.clone_from(partial);
            self.highest_score = score;
        }

        // what cocktails could be added without blowing our ingredient budget?
        // this will be empty on the first iteration
        let mut partial_ingredients = BitSet::new();
        for cocktail in partial.iter() {
            partial_ingredients = partial_ingredients | cocktail;
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
                .to_owned();
            let new_partial_ingredients = partial_ingredients | best;
            let covered_candidates = candidates
                .iter()
                .filter(|&cocktail| cocktail.is_subset(&new_partial_ingredients))
                .cloned()
                .collect();
            let mut permitted_candidates = FxHashSet::default();
            (&*candidates - &covered_candidates)
                .iter()
                .for_each(|cocktail| {
                    let extended_ingredients = cocktail | new_partial_ingredients;
                    if extended_ingredients.len() <= self.max_size {
                        // when we branch, we need to not only remove a cocktail
                        // from the candidate set, but ensure that it's impossible
                        // for the final ingredient list to be a superset of the cocktail.
                        // otherwise, we could undercount the score of the branch.
                        // this is O(N^2), alas.
                        let forbidden_cover =
                            forbidden
                                .as_mut()
                                .unwrap()
                                .iter()
                                .any(|forbidden_cocktail| {
                                    forbidden_cocktail.is_subset(&extended_ingredients)
                                });
                        if !forbidden_cover {
                            permitted_candidates.insert(*cocktail);
                        }
                    }
                });

            self.search(
                &mut permitted_candidates,
                &mut (&*partial | &covered_candidates),
                forbidden,
            );

            let mut remaining = candidates.clone();
            remaining.remove(&best);
            remaining.retain(|cocktail| {
                let test = cocktail | partial_ingredients;
                !best.is_subset(&test)
            });
            let mut new_forbidden = forbidden.as_ref().unwrap().clone();
            new_forbidden.insert(best);

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
        let bound_functions = [
            Self::total_bound,
            Self::singleton_bound,
            Self::concentration_bound,
        ];
        for func in bound_functions {
            let bound = func(self, candidates, partial, partial_ingredients);
            if bound <= threshold {
                return false;
            };
        }
        true
    }

    fn total_bound(
        &self,
        candidates: &FxHashSet<IngredientSeti>,
        _partial: &FxHashSet<IngredientSeti>,
        _partial_ingredients: &IngredientSeti,
    ) -> i32 {
        candidates.len() as i32
    }

    /// There are many cocktails that have an unique ingredient.
    ///
    /// Each cocktail with a unique ingredient will cost at least
    /// one ingredient from our ingredient budget, and the total
    /// possible increase due to these unique cocktails is bounded
    /// by the ingredient budget
    fn singleton_bound(
        &self,
        candidates: &FxHashSet<IngredientSeti>,
        _partial: &FxHashSet<IngredientSeti>,
        partial_ingredients: &IngredientSeti,
    ) -> i32 {
        let n_unique_cocktails = candidates
            .iter()
            .filter(|cocktail| self.min_cover.get(cocktail).unwrap() == &1)
            .count();
        let ingredient_budget = self.max_size - partial_ingredients.len();
        candidates.len() as i32 - n_unique_cocktails as i32
            + (n_unique_cocktails.min(ingredient_budget) as i32)
    }
    /// best case is that excess ingredients are concentrated in
    /// some cocktails. if we are in this best case, then if we
    /// remove the cocktails that add the most new ingredients
    /// we'll be back under the ingredient budget
    /// note that we are just updating the bound. it could actually
    /// be that we want to add one of these cocktails that add
    /// a lot of ingredients
    fn concentration_bound(
        &self,
        candidates: &FxHashSet<IngredientSeti>,
        _partial: &FxHashSet<IngredientSeti>,
        partial_ingredients: &IngredientSeti,
    ) -> i32 {
        let mut candidate_ingredients = BitSet::new();
        for cocktail in candidates.iter() {
            candidate_ingredients = candidate_ingredients | cocktail;
        }
        let mut excess_ingredients =
            (candidate_ingredients | partial_ingredients).len() as i32 - self.max_size as i32;
        let mut ingredient_increases = candidates
            .iter()
            .map(|cocktail| (cocktail - partial_ingredients).len() as i32)
            .collect::<Vec<i32>>();
        ingredient_increases.sort_unstable_by(|a, b| b.cmp(a));
        let mut upper_increment = candidates.len();
        for ingredient_increase in ingredient_increases {
            if excess_ingredients <= 0 {
                break;
            }
            upper_increment -= 1;
            excess_ingredients -= ingredient_increase;
        }
        upper_increment as i32
    }
}
