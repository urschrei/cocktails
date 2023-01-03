//! What is the set of n ingredients that will allow you to make the highest number of different cocktails?
//! E.g.:
//! You have 117 different ingredients
//! You have 104 cocktails, each of which use 2-6 ingredients
//! Which 5 ingredients maximize the cocktail-making possibilities? What about 10 ingredients?
//! Here's a branch and bound solution
//! Original here: https://gist.github.com/tmcw/c6bdcfe505057ed6a0f356cfd02d4d52
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rustc_hash::FxHashSet;
use std::collections::BTreeSet;

pub type Ingredient = String;
pub type IngredientSet = BTreeSet<Ingredient>;

#[derive(Debug)]
pub struct BranchBound {
    pub calls: i32,
    pub max_size: usize,
    pub highest_score: usize,
    pub highest: FxHashSet<IngredientSet>,
    pub random: ThreadRng,
    pub counter: u32,
}

impl BranchBound {
    pub fn new(max_calls: i32, max_size: usize) -> BranchBound {
        BranchBound {
            calls: max_calls,
            max_size,
            highest_score: 0usize,
            highest: FxHashSet::default(),
            random: rand::thread_rng(),
            counter: 0,
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
            self.highest_score = score;
        }

        // what cocktails could be added without blowing our ingredient budget?
        // this will be empty on the first iteration
        let partial_ingredients = partial
            .iter()
            .flatten()
            .cloned()
            .collect::<IngredientSet>();

        // if adding all the associated ingredients of the candidates
        // takes us over the ingredient budget, then not all the
        // candidates can feasibly be added to our partial
        // solution. So, if there will be excess ingredients we'll
        // reduce the upper bound of how many cocktails we might be
        // able to cover (possible_increment)
        candidates.retain(|cocktail| (cocktail | &partial_ingredients).len() <= self.max_size);

        let mut possible_increment = candidates.len();

        let candidate_ingredients = candidates
            .iter()
            .flatten()
            .cloned()
            .collect::<IngredientSet>();
        let mut excess_ingredients =
            (&candidate_ingredients | &partial_ingredients).len() as i32 - self.max_size as i32;

        // best case is that excess ingredients are concentrated in
        // some cocktails. If we're in this best case, removing
        // the cocktails that add the most new ingredients
        // brings us back under the ingredient budget
        //
        // note that we're just updating the bound; it could actually
        // be that we want to add one of these cocktails that add
        // a lot of ingredients
        if excess_ingredients > 0 {
            let mut ingredient_increases = candidates
                .iter()
                .map(|cocktail| (cocktail - &partial_ingredients).len() as i32)
                .collect::<Vec<i32>>();
            ingredient_increases.sort_by(|a, b| b.cmp(a));
            for increase in ingredient_increases {
                possible_increment -= 1;
                excess_ingredients -= increase;
                if excess_ingredients <= 0 {
                    break;
                }
            }
        }

        let threshold = self.highest_score - score;

        if !candidates.is_empty() && possible_increment > threshold {
            // random choice seems to be the best heuristic according to the original author
            let best = candidates.iter().choose(&mut self.random).unwrap().clone();

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
