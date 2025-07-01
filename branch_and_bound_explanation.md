# Branch and Bound Implementation for Cocktail Optimization

## Overview

This implementation solves the **cocktail ingredient optimization problem**: Given a collection of cocktails (each requiring 2-6 ingredients) and a budget of n ingredients, which ingredients should you buy to maximise the number of different cocktails you can make?

The algorithm uses a branch and bound approach with three bounding functions to efficiently prune the search space.

## The Three Bounds

### 1. Total Bound (Most Optimistic)

```rust
fn total_bound(
    &self,
    candidates: &FxHashSet<IngredientSeti>,
    _partial: &FxHashSet<IngredientSeti>,
    _partial_ingredients: &IngredientSeti,
) -> i32 {
    candidates.len() as i32
}
```

**What it does**: Returns the total number of candidate cocktails remaining.

**Logic**: This is the most optimistic bound - it assumes we could make ALL remaining cocktails.

**When it prunes**: When even making all remaining cocktails wouldn't beat our current best solution.

### 2. Singleton Bound (Unique Ingredient Constraint)

```rust
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
```

**What it does**: Accounts for cocktails that have unique ingredients (ingredients used by only that cocktail).

**Mathematical insight**: 
- Each cocktail with a unique ingredient costs at least 1 ingredient from our budget
- We can't make more unique-ingredient cocktails than our remaining budget
- Formula: `non_unique_cocktails + min(unique_cocktails, remaining_budget)`

**When it prunes**: When we have many cocktails with unique ingredients but limited ingredient budget.

### 3. Concentration Bound (Most Sophisticated)

```rust
fn concentration_bound(
    &self,
    candidates: &FxHashSet<IngredientSeti>,
    _partial: &FxHashSet<IngredientSeti>,
    partial_ingredients: &IngredientSeti,
) -> i32 {
    // Calculate total ingredients needed
    let mut candidate_ingredients = BitSet::new();
    for cocktail in candidates.iter() {
        candidate_ingredients = candidate_ingredients | cocktail;
    }
    
    // Find excess over budget
    let mut excess_ingredients = 
        (candidate_ingredients | partial_ingredients).len() as i32 - self.max_size as i32;
    
    // Sort cocktails by how many NEW ingredients they add
    // Remove greediest cocktails until under budget
    // Return count of cocktails that could fit
}
```

**What it does**: Considers the best-case scenario where excess ingredients are concentrated in a few cocktails.

**Mathematical insight**: 
- If we need more ingredients than our budget allows, we must drop some cocktails
- Best case: drop the "greediest" cocktails (those adding the most new ingredients)
- This gives us the maximum possible cocktails we could make

**When it prunes**: When even optimally removing high-ingredient cocktails won't give us enough.

## How the Bounds Work Together

```rust
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
            return false;  // Prune this branch
        }
    }
    true  // Continue exploring
}
```

The bounds are checked in order of computational complexity:
1. **Total bound** - O(1) operation
2. **Singleton bound** - O(n) scan with cached data
3. **Concentration bound** - O(n log n) due to sorting

If ANY bound indicates we can't beat the current best, we prune immediately.

## Concrete Example

Let's work through a simple example with 5 cocktails and a budget of 4 ingredients:

### Setup
- **Cocktails**:
  - Mojito: {rum, mint, lime}
  - Daiquiri: {rum, lime}
  - Martini: {gin, vermouth}
  - Gin & Tonic: {gin, tonic}
  - Moscow Mule: {vodka, ginger}

- **Current best**: 3 cocktails (say we already found {rum, lime, gin, vermouth} → Mojito, Daiquiri, Martini)
- **Current state**: We have {rum, lime} (2 ingredients used, can make Daiquiri)
- **Candidates**: Mojito, Martini, Gin & Tonic, Moscow Mule

### Applying the Bounds

**Threshold**: 3 (current best) - 1 (current partial) = 2 (need 2 more cocktails to beat current best)

**1. Total Bound**
- Candidates: 4 cocktails
- Bound: 4
- Check: 4 > 2 ✓ (continue)

**2. Singleton Bound**
- Unique ingredients: mint (Mojito), vermouth (Martini), tonic (G&T), vodka & ginger (Moscow Mule)
- Unique cocktails: 4 (all have at least one unique ingredient)
- Remaining budget: 4 - 2 = 2 ingredients
- Bound: 4 - 4 + min(4, 2) = 0 + 2 = 2
- Check: 2 ≤ 2 ✗ (PRUNE!)

The singleton bound tells us: with only 2 ingredients left and 4 cocktails each needing unique ingredients, we can make at most 2 more cocktails. This exactly meets our threshold, so we can't beat the current best.

### Why This Works

The singleton bound caught something the total bound missed: even though we have 4 candidate cocktails, the ingredient constraints mean we can't possibly make all of them. This saves us from exploring a branch that would ultimately fail.

## Efficiency Analysis

The branch and bound approach with these three bounds is effective because:

1. **Complementary perspectives**: Each bound catches different infeasible scenarios
2. **Early termination**: We stop as soon as any bound fails
3. **Increasing precision**: Simple bounds filter obvious failures; complex bounds catch subtle ones
4. **Exponential pruning**: Each pruned branch eliminates an exponential number of recursive calls

Without these bounds, the algorithm would need to explore all 2^n possible cocktail combinations. With effective bounding, large portions of the search space are eliminated early, making the problem tractable even for large cocktail databases.

## Key Implementation Details

- **Min cover**: Precomputed minimum cocktail frequency for each ingredient (used in singleton bound)
- **Min amortised cost**: Precomputed cost metric for choosing which cocktail to branch on
- **BitSet operations**: Efficient set operations for ingredient combinations
- **Forbidden checker**: Ensures we don't count cocktails that would be inadvertently included

This combination of bounds and efficient data structures enables the algorithm to find optimal solutions for realistic problem sizes (100+ cocktails, 100+ ingredients) in reasonable time.
