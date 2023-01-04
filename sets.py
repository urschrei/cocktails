"""
Original here: https://gist.github.com/tmcw/c6bdcfe505057ed6a0f356cfd02d4d52?permalink_comment_id=3357246#gistcomment-3357246
Revised version here: https://replit.com/@fgregg/cocktails 
Author: Forest Gregg
"""

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
        self.highest_ingredients = set()

        self.min_cover = {}

    def search(
        self, candidates: Cocktails, partial: Optional[Cocktails] = None
    ) -> Cocktails:

        if partial is None:
            # We'll only hit this conditional on the first call, so
            # we set up our cocktail scoring dictionary
            partial = set()
            cardinality = {}
            for cocktail in candidates:
                for ingredient in cocktail:
                    if ingredient in cardinality:
                        cardinality[ingredient] += 1
                    else:
                        cardinality[ingredient] = 1

            for cocktail in candidates:
                self.min_cover[cocktail] = min(
                    cardinality[ingredient] for ingredient in cocktail
                )

        if self.calls <= 0:
            print("early stop")
            return self.highest

        self.calls -= 1

        score = len(partial)

        if score > self.highest_score:
            self.highest = partial
            self.highest_score = score
            self.highest_ingredients = frozenset.union(*self.highest)
            # print(sorted(cocktails[k] for k in self.highest))
            # print(self.highest_score)
            # print(len(self.highest_ingredients))

        # what cocktails could be added without going over our
        # ingredient budget?
        partial_ingredients: Ingredients
        partial_ingredients = set().union(*partial)
        candidates = {
            cocktail
            for cocktail in candidates
            if len(cocktail | partial_ingredients) <= self.max_size
        }

        upper_increment = len(candidates)

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
            # There are many cocktails that have an unique ingredient.
            #
            # Each cocktail with a unique ingredient will cost at least
            # one ingredient from our ingredient budget, and the total
            # possible increase due to these unique cocktails is bounded
            # by the ingredient budget

            n_unique_cocktails = sum(
                1 for cocktail in candidates if self.min_cover[cocktail] == 1
            )
            ingredient_budget = self.max_size - len(partial_ingredients)

            upper_increment_a = (
                len(candidates)
                - n_unique_cocktails
                + min(n_unique_cocktails, ingredient_budget)
            )

            # alternatively:
            #
            # best case is that excess ingredients are concentrated in
            # some cocktails. if we are in this best case, then if we
            # remove the cocktails that add the most new ingredients
            # we'll be back under the ingredient budget
            #
            # note that we are just updating the bound. it could actually
            # be that we want to add one of these cocktails that add
            # a lot of ingredients
            upper_increment_b = len(candidates)
            ingredient_increases = sorted(
                (len(cocktail - partial_ingredients) for cocktail in candidates),
                reverse=True,
            )
            for ingredient_increase in ingredient_increases:
                if excess_ingredients <= 0:
                    break
                upper_increment_b -= 1
                excess_ingredients -= ingredient_increase

            upper_increment = min(upper_increment_a, upper_increment)

        window = self.highest_score - score

        if candidates and upper_increment > window:

            # the best heuristic i've found is to pick the candidates
            # is the least unique in its ingredient list
            best = max(candidates, key=lambda x: self.min_cover[x])

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

    print(sorted(set().union(*best)))
    print(sorted(cocktails[k] for k in best))
