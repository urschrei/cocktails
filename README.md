# Calculating an Optimal Cocktail Ingredient List

## [Tom](https://gist.github.com/tmcw/c6bdcfe505057ed6a0f356cfd02d4d52) Explains the Problem

- You have 100 different ingredients
- You have 20 cocktails, each of which use 2-6 ingredients
- Which 5 ingredients maximize the cocktail-making possibilities? What about 10 ingredients?

In this case, we have **117 ingredients**, and **104 available cocktails**.
Can we work out an "ideal" list of cocktail ingredients of an **arbitrary length** that will maximise the number of unique cocktails that we can make?

This is – in an abstract sense – a classic combinatorial optimisation problem. One way of solving combinatorics problems is ["branch and bound"](https://en.wikipedia.org/wiki/Branch_and_bound) (BnB). This is a Rust implementation of the solution provided by Forest Gregg (reproduced here, lightly edited).

The Rust version doesn't currently have great performance compared to the Python version as it uses a `BTreeSet` to hold ingredients and these sets – of varying length – are recalculated frequently as the algorithm runs. Unfortunately, `BTreeSet` uses [hashbrown](https://stackoverflow.com/q/20832279/416626) as its hashing algorithm, and it's far too high-quality (slow) compared to [the hashing algorithm used](https://stackoverflow.com/q/20832279/416626) for Python's `frozenset`. 
## Running the code
Run `cargo build --release`. The binary (from [`main.rs`](src/main.rs)) can be run using e.g. `./target/release/branchbound`

## Performance
By "not great" I mean that the time to calculate a set of **12** ingredients on an M2 is:

- 1.95 wall-clock seconds using Rust 1.72 (around 20 % faster)
- 2.37 wall-clock seconds using Python 3.11

This isn't a great result for Rust, and getting to this speed increase involved building lookup tables to map the ingredients and cocktail names to unique `i32` values (considerably faster to hash than `String`), in order to work around the relatively slow `std` Swisstable-based `BTreeSet` hash algorithm.

Note that time complexity rises pretty steeply: producing a list of 16 ingredients takes around 95 seconds.

Both versions take around 100k iterations to converge on a 12-ingredient solution. While we previously used a random remaining candidate cocktail to test the quality of our current search – which resulted in a lot of "misses" – [we now use a new heuristic](https://github.com/fgregg/cocktails): the cocktail among the remaining candidates which is the "least unique" in its ingredients, calculated using a minimum amortized cost function. This has almost halved the number of search rounds, and produces an optimal solution for this heuristic:

1. Amaretto
2. Champagne
3. Cognac
4. Crème de cassis
5. Galliano
6. Gin
7. Grenadine
8. Lemon juice
9. Lime juice
10. Simple syrup
11. Triple sec
12. White rum

Allowing you to mix:

1. Bacardi cocktail
2. Between the sheets
3. Daiquiri
4. French 75
5. French Connection
6. Gimlet
7. Kir royal
8. Sidecar
9. White lady
10. Yellow bird
