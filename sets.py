import random
from typing import AbstractSet, Set, FrozenSet, Optional

Cocktail = FrozenSet[str]
Cocktails = AbstractSet[Cocktail]
Ingredients = Set[str]


class BranchBound(object):
    def __init__(self, max_calls: int, max_size: int) -> None:
        self.calls: int = max_calls
        self.max_size: int = max_size

        self.highest_score: int = 0
        self.highest: Cocktails = set()

    def search(
        self, candidates: Cocktails, partial: Optional[Cocktails] = None
    ) -> Cocktails:
        if partial is None:
            partial = set()

        if self.calls <= 0:
            print("early stop")
            return self.highest

        self.calls -= 1

        score = len(partial)

        if score > self.highest_score:
            self.highest = partial
            self.highest_score = score
            print(self.highest_score)

        # what cocktails could be added without going over our
        # ingredient budget?
        partial_ingredients: Ingredients
        partial_ingredients = set().union(*partial)
        candidates = {
            cocktail
            for cocktail in candidates
            if len(cocktail | partial_ingredients) <= self.max_size
        }

        possible_increment = len(candidates)

        # if adding all the associated ingredients of the candidates
        # takes us over the ingredient budget, then not all the
        # candidates can feasibly be added to our partial
        # solution. So, if there will be excess ingredients, we'll
        # reduce the upper bound of how many cocktails we might be
        # able to cover (possible_increment).
        candidate_ingredients: Ingredients
        candidate_ingredients = set().union(*candidates)
        excess_ingredients = (
            len(candidate_ingredients | partial_ingredients) - self.max_size
        )

        if excess_ingredients > 0:
            # best case is that excess ingredients are concentrated in
            # some cocktails. if we are in this best case, then if we
            # remove the cocktails that add the most new ingredients
            # we'll be back under the ingredient budget
            #
            # note that we are just updating the bound. it could actually
            # be that we want to add one of these cocktails that add
            # a lot of ingredients
            ingredient_increases = sorted(
                (len(cocktail - partial_ingredients) for cocktail in candidates),
                reverse=True,
            )
            for ingredient_increase in ingredient_increases:
                possible_increment -= 1
                excess_ingredients -= ingredient_increase
                if excess_ingredients <= 0:
                    break

        threshold = self.highest_score - score

        if candidates and possible_increment > threshold:

            # i've tried a few heuristics like "next smallest
            # cocktail," and a random choice seems to be best so far
            best = min(candidates, key=lambda x: random.random())

            new_partial_ingredients = partial_ingredients | best
            covered_candidates = {
                cocktail
                for cocktail in candidates
                if cocktail <= new_partial_ingredients
            }

            self.search(candidates - covered_candidates, partial | covered_candidates)

            # if a cocktail is not part of the optimum set than then
            # the optimum set cannot have the cocktail as a subset
            remaining = {
                cocktail
                for cocktail in candidates - set(best)
                if not (best <= (cocktail | partial_ingredients))
            }

            self.search(remaining, partial)

        return self.highest


if __name__ == "__main__":
    import csv

    cocktails = {}

    with open("cocktails.csv") as f:
        reader = csv.reader(f)
        for row in reader:
            name, *ingredients = row
            cocktails[frozenset(ingredients)] = name

    bb = BranchBound(8000000, 12)
    best = bb.search(cocktails.keys())

    print(f"Ingredients: {sorted(set().union(*best))}")
