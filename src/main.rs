use branchbound::{BranchBound, Ingredient, IngredientSet};
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
            .map(|s| s.to_owned())
            .collect::<IngredientSet>();
        let value = r.iter().next().unwrap().to_owned();
        // println!("key {:?}", &key);
        // println!("value {:?}", &value);
        map.insert(key, value);
    });
    let mut cocktails = map.keys().cloned().collect();
    let mut bar: FxHashSet<IngredientSet> = FxHashSet::default();
    let mut bb = BranchBound::new(8000000, 12);
    let best = bb.search(&mut cocktails, &mut bar);
    let fset = best
        .iter()
        .flatten()
        .cloned()
        .collect::<FxHashSet<Ingredient>>();
    // let mut v = Vec::from_iter(fset);
    // v.sort();
    let mut possible_cocktails = best
        .iter()
        .map(|ings| map.get(ings).unwrap())
        .collect::<Vec<_>>();
    possible_cocktails.sort();
    println!("Search rounds {:?}", bb.counter);
    println!("Ingredient set ({}): {:?}", &fset.len(), &fset);
    println!(
        "Possible cocktails ({}) with this set: {:?}",
        &possible_cocktails.len(),
        &possible_cocktails
    );
}
