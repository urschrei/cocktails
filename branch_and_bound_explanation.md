# Branch and Bound Implementation for Cocktail Optimization

## Overview

This implementation solves the **cocktail ingredient optimization problem**: Given a collection of cocktails (each requiring 2 - 6 ingredients) and a budget of n ingredients, which ingredients should you buy to maximise the number of different cocktails you can make?

The algorithm uses a branch and bound approach with three configurable bounding functions to efficiently prune the search space. The modular design allows users to add custom bounds or use only specific bounds as needed.

## The Three Bounds

### 1. Total Bound (Most Optimistic)

```rust
impl BoundFunction for TotalBound {
    fn compute(&self, context: &BoundContext) -> i32 {
        context.candidates.len() as i32
    }
    
    fn name(&self) -> &'static str {
        "TotalBound"
    }
}
```

**What it does**: Returns the total number of candidate cocktails remaining.

**Logic**: This is the most optimistic bound; it assumes we could make ALL remaining cocktails.

**When it prunes**: When even making all remaining cocktails wouldn't beat our current best solution.

### 2. Singleton Bound (Unique Ingredient Constraint)

```rust
impl BoundFunction for SingletonBound {
    fn compute(&self, context: &BoundContext) -> i32 {
        let n_unique_cocktails = context
            .candidates
            .iter()
            .filter(|cocktail| context.min_cover.get(cocktail).unwrap() == &1)
            .count();
        let ingredient_budget = context.max_size - context.partial_ingredients.len();
        context.candidates.len() as i32 - n_unique_cocktails as i32
            + (n_unique_cocktails.min(ingredient_budget) as i32)
    }
    
    fn name(&self) -> &'static str {
        "SingletonBound"
    }
}
```

**What it does**: Accounts for cocktails that have unique ingredients (ingredients used by only that cocktail).

- Each cocktail with a unique ingredient costs at least 1 ingredient from our budget
- We can't make more unique-ingredient cocktails than our remaining budget
- Formula: `non_unique_cocktails + min(unique_cocktails, remaining_budget)`

**When it prunes**: When we have many cocktails with unique ingredients but limited ingredient budget.

### 3. Concentration Bound (Most Sophisticated)

```rust
impl BoundFunction for ConcentrationBound {
    fn compute(&self, context: &BoundContext) -> i32 {
        // Calculate total ingredients needed
        let mut candidate_ingredients = BitSet::new();
        for cocktail in context.candidates.iter() {
            candidate_ingredients = candidate_ingredients | cocktail;
        }
        
        // Find excess over budget
        let mut excess_ingredients = (candidate_ingredients | context.partial_ingredients).len()
            as i32 - context.max_size as i32;
        
        // Sort cocktails by how many NEW ingredients they add
        // Remove greediest cocktails until under budget
        // Return count of cocktails that could fit
        
        // ... stack-based sorting implementation ...
        upper_increment as i32
    }
    
    fn name(&self) -> &'static str {
        "ConcentrationBound"
    }
}
```

**What it does**: Considers the best-case scenario where excess ingredients are concentrated in a few cocktails.

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
    
    let context = BoundContext {
        candidates,
        partial,
        partial_ingredients,
        max_size: self.max_size,
        min_cover: &self.min_cover,
        min_amortized_cost: &self.min_amortized_cost,
    };
    
    // Use iterator methods for cleaner, more functional approach
    self.bound_functions
        .iter()
        .all(|bound| bound.compute(&context) > threshold)
}
```

The bounds are checked using iterator methods:

- Bounds implement the `BoundFunction` trait
- We can add custom bounds or use only specific ones
- Uses `.all()` to check if any bound fails
- Stops as soon as any bound indicates pruning

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

## Modular Bound Architecture

The implementation uses a trait-based approach for flexibility:

### BoundFunction Trait
```rust
trait BoundFunction: Send + Sync {
    fn compute(&self, context: &BoundContext) -> i32;
    fn name(&self) -> &'static str;
}
```

### Configurable Bounds
Users can customise which bounds to use:

```rust
// Use default bounds
let solver = BranchBound::new(1000000, 10);

// Use specific bounds only
let solver = BranchBoundBuilder::new(1000000, 10)
    .with_bound(Box::new(TotalBound))
    .with_bound(Box::new(SingletonBound))
    .build();

// Add custom bounds
let solver = BranchBoundBuilder::new(1000000, 10)
    .with_default_bounds()
    .with_bound(Box::new(MyCustomBound))
    .build();
```

### Creating Custom Bounds
Users can implement their own bounding functions:

```rust
struct MyCustomBound;

impl BoundFunction for MyCustomBound {
    fn compute(&self, context: &BoundContext) -> i32 {
        // Custom logic here
        context.candidates.len() as i32 / 2
    }
    
    fn name(&self) -> &'static str {
        "MyCustomBound"
    }
}
```

## Key Implementation Details

### Min Cover (Minimum Cocktail Coverage)

**Purpose**: For each cocktail, identifies the rarest ingredient it contains.

```rust
self.min_cover.insert(
    *cocktail,
    cocktail
        .iter()
        .map(|ingredient| *cardinality.get(&(ingredient as i32)).unwrap())
        .min()
        .unwrap(),
);
```

**Example**: If a cocktail has rum (used in 10 cocktails), lime (used in 8), and mint (used in only 1), then `min_cover = 1`.

**Role in Algorithm**: 
- Used in `singleton_bound` to identify cocktails with unique ingredients
- A cocktail with `min_cover = 1` has at least one ingredient used nowhere else
- These cocktails are "expensive" because they force us to buy an ingredient for just one cocktail

### Min Amortised Cost (Cocktail Selection Heuristic)

**Purpose**: Estimates the "ingredient cost per cocktail" if we had unlimited budget.

```rust
self.min_amortized_cost.insert(
    *cocktail,
    cocktail
        .iter()
        .map(|ingredient| {
            1f64 / f64::from(*cardinality.get(&(ingredient as i32)).unwrap())
        })
        .sum::<f64>(),
);
```

**Example**: For a cocktail with:
- Rum (used in 10 cocktails): contributes 1/10 = 0.1
- Lime (used in 8 cocktails): contributes 1/8 = 0.125
- Mint (unique): contributes 1/1 = 1.0
- Total amortised cost: 1.225

**Role in Algorithm**:
- Used in branching decision to select the next cocktail to consider
- Lower cost cocktails share more ingredients with others, making them efficient choices
- This greedy heuristic helps find good solutions faster

**Why "Amortised"**: The cost is spread across all cocktails that could use each ingredient. Popular ingredients (like rum or lime) have their cost divided among many cocktails, while unique ingredients bear their full cost.

### BitSet Operations

- **Purpose**: Efficient set operations for ingredient combinations
- **Benefits**: Fast union, intersection, and subset checking using bit manipulation
- **Example**: Checking if we have all ingredients for a cocktail is a single bitwise operation

### Forbidden Checker

- **Purpose**: Ensures we don't count cocktails that would be inadvertently included
- **How it works**: When we decide NOT to include a cocktail in a branch, we must ensure no superset of its ingredients appears in our final solution
- **Implementation**: Maintains masks of forbidden ingredient combinations (lines 16-37)

This combination of modular bound functions, sophisticated algorithms, and efficient data structures enables the algorithm to find optimal solutions for realistic problem sizes (100+ cocktails, 100+ ingredients) in reasonable time. The trait-based architecture makes it easy to experiment with different bounding strategies and extend the algorithm for specific use cases.
