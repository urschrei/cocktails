use rand::seq::IteratorRandom;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

type Ingredient = String;
#[derive(Debug, Clone)]
pub struct IngredientSet(HashSet<Ingredient>);

impl PartialEq for IngredientSet {
    fn eq(&self, other: &IngredientSet) -> bool {
        self.0.is_subset(&other.0) && other.0.is_subset(&self.0)
    }
}

impl Eq for IngredientSet {}

impl IngredientSet {
    fn new() -> Self {
        IngredientSet(HashSet::new())
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
}

impl BranchBound {
    pub fn new(max_calls: i32, max_size: usize) -> BranchBound {
        BranchBound {
            calls: max_calls,
            max_size,
            highest_score: 0usize,
            highest: HashSet::new(),
        }
    }

    pub fn search(
        &mut self,
        candidates: &mut HashSet<IngredientSet>,
        partial: &mut HashSet<IngredientSet>,
    ) -> HashSet<IngredientSet> {
        let mut rng = rand::thread_rng();
        if self.calls <= 0 {
            println!("{:?}", "Early return!");
            return self.highest.clone();
        }
        self.calls -= 1;
        let score = partial.iter().len();
        if score > self.highest_score {
            self.highest = partial.clone();
            self.highest_score = score;
            println!("{:?}", &self.highest_score);
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
                .map(|cocktail| (&cocktail.0 - &partial_ingredients.0).iter().len() as i32)
                .collect::<Vec<i32>>();
            ingredient_increases.sort_by(|a, b| b.cmp(a));
            for increase in ingredient_increases {
                possible_increment -= 1;
                excess_ingredients -= increase;
                if excess_ingredients == 0 {
                    break;
                }
            }
        }
        let threshold = self.highest_score - score;
        if !candidates.is_empty() && possible_increment > threshold {
            let best = candidates.iter().cloned().choose(&mut rng).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {
        let a = vec!["Vodka", "Dry vermouth", "Lemon", "Olive"];
        let b = vec!["Champagne", "Orange juice", "Orange"];
        let c = vec!["Gin", "Dry vermouth", "Lemon"];

        let mut is1 = IngredientSet::new();
        for ing in a {
            is1.0.insert(ing.to_string());
        }
        let mut is2 = IngredientSet::new();
        for ing in b {
            is2.0.insert(ing.to_string());
        }
        let mut is3 = IngredientSet::new();
        for ing in c {
            is3.0.insert(ing.to_string());
        }
        let mut foo = HashSet::new();
        foo.insert(is1);
        foo.insert(is2);
        foo.insert(is3);
        let mut bar: HashSet<IngredientSet> = HashSet::new();
        let mut bb = BranchBound::new(8000000, 16);
        let best = bb.search(&mut foo, &mut bar);
        let fset = best
            .iter()
            .cloned()
            .flat_map(|ing| ing.0)
            .collect::<HashSet<Ingredient>>();
        let mut v = Vec::from_iter(fset);
        v.sort();
        println!("Final set: {:?}", &v);
    }
}
