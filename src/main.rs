use branchbound::{BranchBound, Ingredient, IngredientSet, IngredientSeti, Ingredienti};
use csv::ReaderBuilder;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufReader;

fn main() {
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
            if ingredient_lookup.get(ingredient).is_none() {
                ingredient_lookup.insert(ingredient, counter);
                ingredient_lookup_reverse.insert(counter, ingredient);
                counter += 1;
            }
        }
        let ingredientset = ingset
            .iter()
            .map(|ingredient| *ingredient_lookup.get(ingredient).unwrap())
            .collect::<BTreeSet<i32>>();
        // populate mapping for optimisation
        numeric_set.insert(ingredientset.clone());
        cocktail_lookup_reverse.insert(ingredientset, name);
    });
    let mut bb = BranchBound::new(8_000_000, 12);

    let best = bb.search(&mut numeric_set, &mut res, &mut None);
    // map back from sets of i32 to cocktail names
    let mut best_names = best
        .iter()
        .map(|cocktail| cocktail_lookup_reverse.get(cocktail).unwrap())
        .collect::<Vec<&&String>>();
    best_names.sort_unstable();

    let fset = best
        .iter()
        .flatten()
        .copied()
        .collect::<FxHashSet<Ingredienti>>();
    // map back from i32 to ingredient names
    let mut fset_names = fset
        .iter()
        .map(|entry| ingredient_lookup_reverse.get(entry).unwrap())
        .collect::<Vec<&&Ingredient>>();
    fset_names.sort_unstable();

    println!("Search rounds {:?}", bb.counter);
    println!("Ingredient set ({}): {:?}", &fset_names.len(), &fset_names);
    println!(
        "Possible cocktails ({}) with this set: {:?}",
        &best_names.len(),
        &best_names
    );
}
