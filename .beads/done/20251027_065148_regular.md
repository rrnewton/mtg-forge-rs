TODO: rename test_decks "decks"
----------------------------------------
Let's use a shorter name. Just be sure to search for all references across the repository and replace them all, making sure that validation passes.

TODO: search for cardsfolder
------------------------------------------
Right now, we expect `./cardsfolder` to exist. Later we'll have a proper installer,
but for now let's make our search process this:
 - `./cardsfolder` if it exists, if not
 - go to the directory containing the `mtg` binary, look for `cardsfolder` there
 - if not found, go up to the parent directory, repeating the search for `./cardsfolder`.
 - if we reach the root `/` and don't find it, then error.

TODO: Find and fix our leaky tests
--------------------
We had a report that some of our tests are leaking memory. Investigate and fix.


TODO: minor: if one deck is passed to `mtg tui` use that for both players
-------------------
This is just a convenience feature for me using the `mtg tui` on the command line.


TODO: overhaul snapshot serialization
--------------------------------------------

First, produce a criterion benchmark that times the saving of snapshot to disk. You probably want to play midway into a game to get a good representative snapshot to benchmark.

Second, stop pretty-printing the json snapshots. We don't need that and the user can always use `jq`.

Third, introduce a flag to control json/binary serialization, and make it binary by default. You should be able to use the same `Serialize` instance but with a different backend. You can use the `bincode` serde backend because we don't need them to be versioned, self-describing, or shared with non-Rust languages.



TODO: Separate seed for initial shuffle vs subsequent game
-------------------------------------------------------------
We want to retain the ability to deterministically test by controlling RNG
seeds. But sometimes we may want to sample many DIFFERENT games from the space
of all possible games between two decks.

To this end, in addition to `--seed` let's have a separate `--deck-seed`. If not
provided, it is initalized from the main seed. If it is provided, then the deck
seed is used only for the initial random decisions before the game starts
(shuffling) and the --seed can be varied independently to sample different runs
of the same game while keeping the inital hands the same.

This will be useful for testing if one agent can be "smarter" and win under the
same conditions that another loses.

Also --seed currently takes numbers only. If it is passed "clock" then let's
seed the RNG from the system clock using real entropy in the usual way.



