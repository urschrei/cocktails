use rand::rngs::ThreadRng;
/// What is the set of n ingredients that will allow you to make the highest number of different cocktails?
/// E.g.:
/// You have 100 different ingredients
/// You have 20 cocktails, each of which use 2-6 ingredients
/// Which 5 ingredients maximize the cocktail-making possibilities? What about 10 ingredients?
use rand::seq::IteratorRandom;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub type Ingredient = String;
#[derive(Debug, Clone)]
pub struct IngredientSet(pub HashSet<Ingredient>);

impl PartialEq for IngredientSet {
    fn eq(&self, other: &IngredientSet) -> bool {
        self.0.is_subset(&other.0) && other.0.is_subset(&self.0)
    }
}

impl Eq for IngredientSet {}

impl IngredientSet {
    pub fn new() -> Self {
        IngredientSet(HashSet::new())
    }
}

impl Default for IngredientSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for IngredientSet {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let mut a: Vec<&Ingredient> = self.0.iter().collect();
        a.sort();
        for s in a.iter() {
            s.hash(state);
        }
    }
}

#[derive(Debug)]
pub struct BranchBound {
    pub calls: i32,
    pub max_size: usize,
    pub highest_score: usize,
    pub highest: HashSet<IngredientSet>,
    pub random: ThreadRng,
}

impl BranchBound {
    pub fn new(max_calls: i32, max_size: usize) -> BranchBound {
        BranchBound {
            calls: max_calls,
            max_size,
            highest_score: 0usize,
            highest: HashSet::new(),
            random: rand::thread_rng(),
        }
    }

    pub fn search(
        &mut self,
        candidates: &mut HashSet<IngredientSet>,
        partial: &mut HashSet<IngredientSet>,
    ) -> HashSet<IngredientSet> {
        if self.calls <= 0 {
            println!("{:?}", "Early return!");
            return self.highest.clone();
        }
        self.calls -= 1;
        let score = partial.iter().len();
        if score > self.highest_score {
            self.highest = partial.clone();
            self.highest_score = score;
            println!(
                "Found {:?} valid ingredient combinations",
                &self.highest_score
            );
        }

        // what cocktails could be added without blowing our ingredient budget?
        let partial_ingredients_ = partial
            .iter()
            .cloned()
            .flat_map(|ingredient| ingredient.0)
            .collect::<HashSet<Ingredient>>();
        let mut partial_ingredients = IngredientSet::new();
        partial_ingredients.0 = partial_ingredients_;

        // if adding all the associated ingredients of the candidates
        // takes us over the ingredient budget, then not all the
        // candidates can feasibly be added to our partial
        // solution. So, if there will be excess ingredients we'll
        // reduce the upper bound of how many cocktails we might be
        // able to cover (possible_increment)
        candidates.retain(|cocktail| (&cocktail.0 | &partial_ingredients.0).len() <= self.max_size);

        let mut possible_increment = candidates.iter().len();

        let candidate_ingredients_ = candidates
            .iter()
            .cloned()
            .flat_map(|ing| ing.0)
            .collect::<HashSet<Ingredient>>();
        let mut candidate_ingredients = IngredientSet::new();
        candidate_ingredients.0 = candidate_ingredients_;
        let mut excess_ingredients =
            (&candidate_ingredients.0 | &partial_ingredients.0).len() as i32 - self.max_size as i32;

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
                .map(|cocktail| (&cocktail.0 - &partial_ingredients.0).len() as i32)
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
            let best = candidates.iter().cloned().choose(&mut self.random).unwrap();

            let new_partial_ingredients = &partial_ingredients.0 | &best.0;
            let covered_candidates: HashSet<IngredientSet> = candidates
                .iter()
                .cloned()
                .filter(|cocktail| cocktail.0.is_subset(&new_partial_ingredients))
                .collect();

            self.search(
                &mut (&*candidates - &covered_candidates),
                &mut (&*partial | &covered_candidates),
            );

            // if a cocktail is not part of the optimum set,
            // the optimum set cannot have the cocktail as a subset
            let mut remaining: HashSet<IngredientSet> = candidates
                .iter()
                .cloned()
                .filter(|cocktail| !best.0.is_subset(&(&cocktail.0 | &partial_ingredients.0)))
                .collect();

            self.search(&mut remaining, partial);
        }
        self.highest.clone()
    }
}
