//! What is the set of n ingredients that will allow you to make the highest number of different cocktails?
//! E.g.:
//! You have 117 different ingredients
//! You have 104 cocktails, each of which use 2-6 ingredients
//! Which 5 ingredients maximize the cocktail-making possibilities? What about 10 ingredients?
//! Here's a branch and bound solution
//! Original here: https://gist.github.com/tmcw/c6bdcfe505057ed6a0f356cfd02d4d52
use rand::rngs::ThreadRng;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeSet;

pub type Ingredient = String;
pub type IngredientSet = BTreeSet<Ingredient>;

#[derive(Debug)]
pub struct BranchBound {
    pub calls: i32,
    pub max_size: usize,
    pub highest_score: usize,
    pub highest: FxHashSet<IngredientSet>,
    pub highest_ingredients: BTreeSet<Ingredient>,
    pub random: ThreadRng,
    pub counter: u32,
    pub min_cover: FxHashMap<BTreeSet<String>, i32>,
}

impl BranchBound {
    pub fn new(max_calls: i32, max_size: usize) -> BranchBound {
        BranchBound {
            calls: max_calls,
            max_size,
            highest_score: 0usize,
            highest: FxHashSet::default(),
            highest_ingredients: BTreeSet::new(),
            random: rand::thread_rng(),
            counter: 0,
            min_cover: FxHashMap::default(),
        }
    }

    pub fn search(
        &mut self,
        candidates: &mut FxHashSet<IngredientSet>,
        partial: &mut FxHashSet<IngredientSet>,
    ) -> FxHashSet<IngredientSet> {
        self.counter += 1;
        if self.calls <= 0 {
            println!("{:?}", "Early return!");
            return self.highest.clone();
        }
        self.calls -= 1;
        let score = partial.len();
        if score > self.highest_score {
            self.highest = partial.clone();
            self.highest_ingredients = self
                .highest_ingredients
                .union(&self.highest.iter().flatten().cloned().collect())
                .cloned()
                .collect();
            self.highest_score = score;
        }

        // first run-through, so populate min_cover and cocktail cardinality
        if partial.is_empty() {
            let mut cardinality = FxHashMap::default();
            candidates
                .iter()
                .flatten()
                .for_each(|ingredient| *cardinality.entry(ingredient).or_insert(0) += 1);

            candidates.iter().for_each(|cocktail| {
                self.min_cover.insert(
                    cocktail.clone(),
                    *cocktail
                        .iter()
                        .map(|ingredient| cardinality.get(ingredient).unwrap())
                        .min()
                        .unwrap(),
                );
            });
        }

        // what cocktails could be added without blowing our ingredient budget?
        // this will be empty on the first iteration
        let partial_ingredients = partial.iter().flatten().cloned().collect::<IngredientSet>();

        // if adding all the associated ingredients of the candidates
        // takes us over the ingredient budget, then not all the
        // candidates can feasibly be added to our partial
        // solution. So, if there will be excess ingredients we'll
        // reduce the upper bound of how many cocktails we might be
        // able to cover (possible_increment)
        candidates.retain(|cocktail| (cocktail | &partial_ingredients).len() <= self.max_size);

        let mut upper_increment = candidates.len();

        let candidate_ingredients = candidates
            .iter()
            .flatten()
            .cloned()
            .collect::<IngredientSet>();
        let mut excess_ingredients =
            (&candidate_ingredients | &partial_ingredients).len() as i32 - self.max_size as i32;

        if excess_ingredients > 0 {
            // There are many cocktails that have a unique ingredient.
            //
            // Each cocktail with a unique ingredient will cost at least
            // one ingredient from our ingredient budget, and the total
            // possible increase due to these unique cocktails is bounded
            // by the ingredient budget
            let n_unique_cocktails = candidates
                .iter()
                .filter(|cocktail| self.min_cover.get(cocktail).unwrap() == &1)
                .count();
            let ingredient_budget = self.max_size - partial_ingredients.len();
            upper_increment =
                candidates.len() - n_unique_cocktails + n_unique_cocktails.min(ingredient_budget);

            // Alternatively:
            // The best case is that excess ingredients are concentrated in
            // some cocktails. If we're in this best case, removing
            // the cocktails that add the most new ingredients
            // brings us back under the ingredient budget
            //
            // note that we're just updating the bound; it could actually
            // be the case that we want to add one of these cocktails that add
            // a lot of ingredients
            let mut upper_increment_b = candidates.len();
            let mut ingredient_increases = candidates
                .iter()
                .map(|cocktail| (cocktail - &partial_ingredients).len() as i32)
                .collect::<Vec<i32>>();
            ingredient_increases.sort_by(|a, b| b.cmp(a));
            for increase in ingredient_increases {
                upper_increment_b -= 1;
                excess_ingredients -= increase;
                if excess_ingredients <= 0 {
                    break;
                }
            }
            upper_increment = upper_increment.min(upper_increment_b);
        }

        let window = self.highest_score - score;

        if !candidates.is_empty() && upper_increment > window {
            // new best heuristic: pick the candidate cocktail
            // which is the "least unique" in its ingredient list
            let best = candidates
                .iter()
                .max_by(|a, b| {
                    self.min_cover
                        .get(a)
                        .unwrap()
                        .cmp(self.min_cover.get(b).unwrap())
                })
                .unwrap()
                .clone();

            let new_partial_ingredients = &partial_ingredients | &best;
            let covered_candidates = candidates
                .iter()
                .cloned()
                .filter(|cocktail| cocktail.is_subset(&new_partial_ingredients))
                .collect();

            self.search(
                &mut (&*candidates - &covered_candidates),
                &mut (&*partial | &covered_candidates),
            );

            // if a cocktail is not part of the optimum set,
            // the optimum set cannot have the cocktail as a subset
            candidates.retain(|cocktail| !best.is_subset(&(cocktail | &partial_ingredients)));
            self.search(candidates, partial);
        }
        self.highest.clone()
    }
}
