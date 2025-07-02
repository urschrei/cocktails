use branchbound::{BitSet, BranchBound, Ingredient, IngredientSet, IngredientSeti, Ingredienti};
use clap::Parser;
use csv::ReaderBuilder;
use rustc_hash::{FxHashMap, FxHashSet};
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

/// Cocktail Ingredients Optimiser - Find optimal ingredient combinations
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of ingredients to select (2-109)
    #[arg(short, long, default_value_t = 12)]
    ingredients: usize,

    /// Maximum search iterations
    #[arg(short, long, default_value_t = 8_000_000)]
    max_calls: i32,

    /// Output format: table, json, or simple
    #[arg(short, long, default_value = "table")]
    format: String,

    /// Generate markdown documentation (hidden)
    #[arg(long, hide = true)]
    markdown_help: bool,
}

fn main() {
    let args = Args::parse();

    // Generate markdown documentation if requested
    if args.markdown_help {
        clap_markdown::print_help_markdown::<Args>();
        return;
    }

    // Validate arguments
    if args.ingredients < 2 || args.ingredients > 109 {
        eprintln!("Error: Number of ingredients must be between 2 and 109");
        std::process::exit(1);
    }

    let mut map: FxHashMap<IngredientSet, String> = FxHashMap::default();

    let f = File::open("cocktails.csv").unwrap();
    let reader = BufReader::new(f);
    let csvr = ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(reader);
    let records = csvr.into_records();
    // populate map
    records.for_each(|record| {
        let r = record.unwrap();
        let key = r
            .iter()
            .skip(1)
            .map(std::borrow::ToOwned::to_owned)
            .collect::<IngredientSet>();
        let value = r.iter().next().unwrap().to_owned();
        // println!("key {:?}", &key);
        // println!("value {:?}", &value);
        map.insert(key, value);
    });
    let mut res: FxHashSet<IngredientSeti> = FxHashSet::default();
    // build mapping from cocktail <--> set<i32> and ingredient <--> i32
    let mut ingredient_lookup = FxHashMap::default();
    let mut cocktail_lookup = FxHashMap::default();
    let mut ingredient_lookup_reverse = FxHashMap::default();
    let mut cocktail_lookup_reverse = FxHashMap::default();

    let mut numeric_set = FxHashSet::default();
    // populate lookups
    let mut counter = 0;
    map.iter().enumerate().for_each(|(i, (ingset, name))| {
        cocktail_lookup.entry(name).or_insert(i);
        for ingredient in ingset {
            if !ingredient_lookup.contains_key(ingredient) {
                ingredient_lookup.insert(ingredient, counter);
                ingredient_lookup_reverse.insert(counter, ingredient);
                counter += 1;
            }
        }
        let ingredientset = BitSet::bitset_from_iter(
            ingset
                .iter()
                .map(|ingredient| *ingredient_lookup.get(ingredient).unwrap()),
        );
        // populate mapping for optimisation
        numeric_set.insert(ingredientset.clone());
        cocktail_lookup_reverse.insert(ingredientset, name);
    });
    println!(
        "Optimizing for {} ingredients with up to {} search iterations...",
        args.ingredients, args.max_calls
    );

    let start_time = Instant::now();
    let mut bb = BranchBound::new(args.max_calls, args.ingredients);

    let best = bb.search(&mut numeric_set, &mut res, &mut None);
    // map back from sets of i32 to cocktail names
    let mut best_names = best
        .iter()
        .map(|cocktail| cocktail_lookup_reverse.get(cocktail).unwrap())
        .collect::<Vec<&&String>>();
    best_names.sort_unstable();

    let mut fset = FxHashSet::default();
    for cocktail in best.iter() {
        for ingredient in cocktail.iter() {
            fset.insert(ingredient as Ingredienti);
        }
    }
    // map back from i32 to ingredient names
    let mut fset_names = fset
        .iter()
        .map(|entry| ingredient_lookup_reverse.get(entry).unwrap())
        .collect::<Vec<&&Ingredient>>();
    fset_names.sort_unstable();

    let duration = start_time.elapsed();

    match args.format.as_str() {
        "json" => print_json_output(&args, &bb, &fset_names, &best_names, duration),
        "simple" => print_simple_output(&args, &bb, &fset_names, &best_names, duration),
        _ => print_table_output(&args, &bb, &fset_names, &best_names, duration),
    }
}

fn print_table_output(
    args: &Args,
    bb: &BranchBound,
    ingredients: &[&&Ingredient],
    cocktails: &[&&String],
    duration: std::time::Duration,
) {
    const TABLE_WIDTH: usize = 55; // Interior width of the table

    println!("\n┌─────────────────────────────────────────────────────────┐");
    println!(
        "│ {:^width$} │",
        "Cocktail Optimiser Results",
        width = TABLE_WIDTH
    );
    println!("├─────────────────────────────────────────────────────────┤");

    let line1 = format!("Target ingredients: {}", args.ingredients);
    println!("│ {line1:<TABLE_WIDTH$} │");

    let line2 = format!("Search iterations: {}", bb.counter);
    println!("│ {line2:<TABLE_WIDTH$} │");

    let line3 = format!("Execution time: {}ms", duration.as_millis());
    println!("│ {line3:<TABLE_WIDTH$} │");

    let line4 = format!("Optimal cocktails: {}", cocktails.len());
    println!("│ {line4:<TABLE_WIDTH$} │");

    let line5 = format!("Ingredients used: {}", ingredients.len());
    println!("│ {line5:<TABLE_WIDTH$} │");

    println!("└─────────────────────────────────────────────────────────┘");

    println!("\n🛒 Optimal Ingredient List ({}):", ingredients.len());
    for (i, ingredient) in ingredients.iter().enumerate() {
        println!("  {:2}. {}", i + 1, ingredient);
    }

    println!("\n🍸 Possible Cocktails ({}):", cocktails.len());
    for (i, cocktail) in cocktails.iter().enumerate() {
        println!("  {:2}. {}", i + 1, cocktail);
    }
}

fn print_simple_output(
    args: &Args,
    bb: &BranchBound,
    ingredients: &[&&Ingredient],
    cocktails: &[&&String],
    duration: std::time::Duration,
) {
    println!("Target: {} ingredients", args.ingredients);
    println!("Iterations: {}", bb.counter);
    println!("Time: {:.1}ms", duration.as_millis());
    println!("Cocktails: {}", cocktails.len());
    println!("Ingredients: {}", ingredients.len());

    println!("\nIngredients:");
    for ingredient in ingredients {
        println!("  {ingredient}");
    }

    println!("\nCocktails:");
    for cocktail in cocktails {
        println!("  {cocktail}");
    }
}

fn print_json_output(
    args: &Args,
    bb: &BranchBound,
    ingredients: &[&&Ingredient],
    cocktails: &[&&String],
    duration: std::time::Duration,
) {
    println!("{{");
    println!("  \"target_ingredients\": {},", args.ingredients);
    println!("  \"search_iterations\": {},", bb.counter);
    println!("  \"execution_time_ms\": {:.1},", duration.as_millis());
    println!("  \"optimal_cocktails\": {},", cocktails.len());
    println!("  \"ingredients_used\": {},", ingredients.len());
    println!("  \"ingredients\": [");
    for (i, ingredient) in ingredients.iter().enumerate() {
        let comma = if i < ingredients.len() - 1 { "," } else { "" };
        println!("    \"{ingredient}\"{comma}");
    }
    println!("  ],");
    println!("  \"cocktails\": [");
    for (i, cocktail) in cocktails.iter().enumerate() {
        let comma = if i < cocktails.len() - 1 { "," } else { "" };
        println!("    \"{cocktail}\"{comma}");
    }
    println!("  ]");
    println!("}}");
}
