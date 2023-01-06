"""
Latest version from https://github.com/fgregg/cocktails/blob/866ddc12af604c787c229f1fb9d3875eb9fb4483/cocktails.py
(04/01/2023)
"""

from typing import AbstractSet, Set, FrozenSet, Optional

Cocktail = FrozenSet[str]
Cocktails = AbstractSet[Cocktail]
Ingredients = Set[str]


class BranchBound(object):
    def __init__(self, max_calls: int, max_size: int) -> None:
        self.rounds: int = 0
        self.calls: int = max_calls
        self.max_size: int = max_size

        self.highest_score: int = 0
        self.highest: Cocktails = set()

        self.min_amortized_cost: dict[Cocktail, float] = {}
        self.min_cover: dict[Cocktail, int] = {}

    def search(
        self, candidates: Cocktails, partial: Optional[Cocktails] = None
    ) -> Cocktails:
        self.rounds += 1
        if partial is None:
            # We'll only hit this conditional on the first call, so
            # we set up our cocktail scoring dictionary
            partial = set()
            cardinality: dict[str, int] = {}
            for cocktail in candidates:
                for ingredient in cocktail:
                    if ingredient in cardinality:
                        cardinality[ingredient] += 1
                    else:
                        cardinality[ingredient] = 1

            # For each cocktail, we can calculate the minimum
            # amoritized cost. That is, if we were to have enough
            # ingredients to make all the cocktails, how much should
            # we pay, in ingredient-cost, for each cocktail. For
            # example, if a cocktail has a unique ingredient, and two
            # other ingredients shared by one other cocktail, then the
            # amortized cost would be 1/1 + 1/2 + 1/2 = 2
            #
            # The minimum amoritized cost is a lower bound to how much
            # we will ever pay in ingredient cost for a cocktail.
            for cocktail in candidates:
                self.min_amortized_cost[cocktail] = sum(
                    1 / cardinality[ingredient] for ingredient in cocktail
                )
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
            # We will calculate three upper bounds to the possible cocktail
            # increment, and then we'll choose the tightest bound.

            # Firstly,
            #
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

            # secondly:
            #
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

            upper_increment_b = len(candidates)
            for ingredient_increase in ingredient_increases:
                if excess_ingredients <= 0:
                    break
                upper_increment_b -= 1
                excess_ingredients -= ingredient_increase

            # thirdly:
            #
            # we can see how many more cocktails we can fit into the ingredient
            # budget assuming minimum, amortized costs for all cocktails
            amortized_budget = self.max_size - sum(
                self.min_amortized_cost[cocktail] for cocktail in partial
            )
            candidate_amortized_costs = sorted(
                self.min_amortized_cost[cocktail] for cocktail in candidates
            )

            total_cost = 0.0
            upper_increment_c = 0
            for cost in candidate_amortized_costs:
                total_cost += cost
                if total_cost > amortized_budget:
                    break
                upper_increment_c += 1

            upper_increment = min(
                upper_increment_a, upper_increment_b, upper_increment_c
            )

        window = self.highest_score - score

        if candidates and upper_increment > window:

            # the best heuristic i've found is to pick the candidates
            # is the least unique in its ingredient list
            best = min(candidates, key=lambda x: self.min_amortized_cost[x])

            new_partial_ingredients = partial_ingredients | best
            covered_candidates = {
                cocktail
                for cocktail in candidates
                if cocktail <= new_partial_ingredients
            }

            self.search(candidates - covered_candidates, partial | covered_candidates)

            # if a cocktail is not part of the optimum set then then
            # the optimum set cannot have the cocktail as a subset
            remaining = {
                cocktail
                for cocktail in candidates - set([best])
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

    print(f"Rounds: {bb.rounds}")
    print(sorted(set().union(*best)))
    print(sorted(cocktails[k] for k in best))
