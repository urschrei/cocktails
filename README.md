# Calculating an Optimal Cocktail Ingredient List

## [Tom](https://gist.github.com/tmcw/c6bdcfe505057ed6a0f356cfd02d4d52) Explains the Problem

- You have 100 different ingredients
- You have 20 cocktails, each of which use 2-6 ingredients
- Which 5 ingredients maximize the cocktail-making possibilities? What about 10 ingredients?

In this case, we have **117 ingredients**, and **104 available cocktails**.
Can we work out an "ideal" list of cocktail ingredients of an **arbitrary length** that will maximise the number of unique cocktails that we can make?

This is – in an abstract sense – a classic combinatorial optimisation problem. One way of solving combinatorics problems is ["branch and bound"](https://en.wikipedia.org/wiki/Branch_and_bound) (BnB). This is a Rust implementation of the solution provided by Forest Gregg (reproduced here, lightly edited).

The Rust version doesn't currently have great performance compared to the Python version as it uses a `BTreeSet` to hold ingredients and these sets – of varying length – are recalculated frequently as the algorithm runs. Unfortunately, `BTreeSet` uses [hashbrown](https://stackoverflow.com/q/20832279/416626) as its hashing algorithm, and it's far too high-quality (slow) compared to [the hashing algorithm used](https://stackoverflow.com/q/20832279/416626) for Python's `frozenset`. I have some ideas for replacing `BTreeSet` which will hopefully give an order-of-magnitude speedup over the Python version, but we shall see.

## Running the code
Run `cargo build --release`. The binary (from [`main.rs`](src/main.rs)) can be run using e.g. `./target/release/branchbound`

## Performance
By "not great" I mean:

- Rust takes around 15 wall-clock seconds to calculate a **set of 12 ingredients**
- Python takes around 7 wall-clock seconds

Note that time complexity rises pretty steeply: producing a list of 16 ingredients takes almost ten minutes.

Both versions take around 100k iterations to converge on a solution. While we previously used a random remaining candidate cocktail to test the quality of our current search – which resulted in a lot of "misses" – we now use a new heuristic: the cocktail among the remaining candidates which is the "least unique" in its ingredients. This has almost halved the number of search rounds, and produces an optimal solution for this heuristic:

1. Champagne
2. Cognac
3. Crème de cassis
4. Galliano
5. Gin
6. Grenadine
7. Lemon juice
8. Lime juice
9. Simple syrup
10. Triple sec
11. White crème de menthe
12. White rum

Allowing you to mix:

1. Bacardi cocktail
2. Between the sheets
3. Daiquiri
4. French 75
5. Gimlet
6. Kir royal
7. Sidecar
8. Stinger
9. White lady
10. Yellow bird
