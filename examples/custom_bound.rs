//! Example showing how to create and use custom bound functions

use branchbound::bounds::{BoundContext, BoundFunction};
use branchbound::{BitSet, BranchBoundBuilder};
use rustc_hash::FxHashSet;

/// A custom bound that uses a simple heuristic
struct SimpleBound {
    factor: f64,
}

impl BoundFunction for SimpleBound {
    fn compute(&self, context: &BoundContext) -> i32 {
        // Simple heuristic: just scale the candidate count
        (context.candidates.len() as f64 * self.factor) as i32
    }

    fn name(&self) -> &'static str {
        "SimpleBound"
    }
}

fn main() {
    // Create a branch and bound solver with custom bounds
    let solver = BranchBoundBuilder::new(1000000, 10)
        .with_default_bounds() // Add the standard bounds
        .with_bound(Box::new(SimpleBound { factor: 0.8 })) // Add our custom bound
        .build();

    println!(
        "Created solver with {} bound functions",
        solver.bound_functions.len()
    );

    // You can also create a solver with only specific bounds
    let custom_solver = BranchBoundBuilder::new(1000000, 10)
        .with_bound(Box::new(SimpleBound { factor: 0.9 }))
        .build();

    println!(
        "Created custom solver with {} bound function",
        custom_solver.bound_functions.len()
    );

    // Example of how to create cocktails (using BitSet directly)
    let mut cocktails = FxHashSet::default();

    // Mojito: rum (0), mint (1), lime (2)
    let mut mojito = BitSet::new();
    mojito.insert(0);
    mojito.insert(1);
    mojito.insert(2);
    cocktails.insert(mojito);

    // Daiquiri: rum (0), lime (2)
    let mut daiquiri = BitSet::new();
    daiquiri.insert(0);
    daiquiri.insert(2);
    cocktails.insert(daiquiri);

    println!("Created {} cocktails for testing", cocktails.len());
}
