use rand::seq::IteratorRandom;
use std::collections::HashSet;

#[derive(Debug)]
pub struct BranchBound {
    pub calls: i32,
    pub max_size: usize,
    pub highest_score: usize,
    pub highest: HashSet<HashSet<String>>,
}

impl BranchBound {
    fn new(max_calls: i32, max_size: usize) -> BranchBound {
        BranchBound {
            calls: max_calls,
            max_size,
            highest_score: 0usize,
            highest: HashSet::new(),
        }
    }

    fn search(
        mut self,
        candidates: &mut HashSet<HashSet<String>>,
        partial: &mut HashSet<HashSet<String>>,
    ) -> HashSet<HashSet<String>> {
        let mut rng = rand::thread_rng();
        if self.calls <= 0 {
            println!("{:?}", "Early return!");
            return self.highest;
        }
        self.calls -= 1;
        let score = partial.iter().len();
        if score > self.highest_score {
            self.highest = partial.clone();
            self.highest_score = score;
        }
        let mut partial_ingredients = HashSet::new();
        partial_ingredients.extend(partial.iter().collect());
        candidates
            .into_iter()
            .filter(|cocktail| (cocktail | &partial_ingredients).len() <= self.max_size)
            .collect();
        let possible_increment = candidates.iter().len();

        let candidate_ingredients = HashSet::new();
        partial_ingredients.extend(candidates.iter().collect());
        let excess_ingredients =
            (&candidate_ingredients | &partial_ingredients).len() - self.max_size;
        if excess_ingredients > 0 {
            let mut ingredient_increases = candidates
                .iter()
                .map(|cocktail| (cocktail - &partial_ingredients).iter().len())
                .collect::<Vec<usize>>();
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
            let best = candidates.iter().choose(&mut rng).unwrap();
            let new_partial_ingredients = &partial_ingredients | best;
            let mut foo = HashSet::from_iter(
                candidates
                    .into_iter()
                    .filter(|cocktail| cocktail.is_subset(&new_partial_ingredients)),
            );
            let covered_candidates = candidates
                .into_iter()
                .filter(|cocktail| cocktail.is_subset(&new_partial_ingredients))
                .collect();
            self.search(
                *candidates - &*covered_candidates,
                partial | covered_candidates,
            );
            let mut remaining = candidates
                .iter()
                .filter(|cocktail| !best.is_subset(&(*cocktail | &partial_ingredients)))
                .collect();
            self.search(remaining, partial);
        }
        return self.highest;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
