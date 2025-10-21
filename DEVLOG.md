
Look at TODO.md, Claude.md, and PROJECT_VISION and get ready to work.

Dependencies:

```
sudo dnf install -y golang
go install github.com/steveyegge/beads/cmd/bd@latest
export PATH=$HOME/go/bin:$PATH
```

[2025.10.19] {Additional prompts beyond the original setup}
================================================================================

It looks like we have lots of methods that take a player ID as an EntityID: `get_player_zones`, `Card::new`, etc. This is rather weakly typed. EntityID is used for many different kinds of entities---cards, players etc.

Before we go further let's refactor this so that we use distinct types for these different uses of `EntityID` and get a type error otherwise. They can all share the same 
physical representation of an unboxed integer.

We can accomplish this by making EntitID take a "phantom type parameter", e.g. `EntityID<Player>`. That indicates which kind of entity it as an ID for, using the actual type (`Player`) of the entity that will be looked up. This makes entity lookup strongly typed, because an `EntityID<T>` will resolve to a `T`.

Then, for the common types we can introduce type aliases as shorthand:

```
PlayerID = EntityID<Player>
CardID = EntityID<Card>
(etc..)
```

Note I have also updated Claude.md to reflect the emphasis on strong typing and include some extra requirements on validation and commit messages.


-------------------------------- 

I also noticed weakly typed indices into maps:

```
pub subtypes: SmallVec<[String; 2]>,
```

Let's get rid of all of those and use more specific types (enums, newtypes, or at least aliases).


Implement and test the undo functionality
----------------------------------------

First, a minor refactor. In ModifyLife let's rename `amount` `delta` instead so it is clear it is an increment/decrement and not an absolute amount.

Next, let's finish and test this undo log. It looks like the types are there, but that it is not actually USED by the game logic. Let's start to remedy that. We need to insure that EVERY mutable method on `GameState` (i.e. which takes a `&mut self`) also adds to an UndoLog stored in the `GameState`. Ensure this is the case. Then, as a first test, at the end of the lightning bolt example, report the length of the undo log and print the list of actions stored in it.


CURRENT INTERNAL TODO
----------------------------------------

    ✻ Refactoring ModifyLife… (esc to interrupt · ctrl+t to hide todos)
      ⎿  ☐ Rename amount to delta in ModifyLife action
         ☐ Add UndoLog field to GameState
         ☐ Add undo logging to all mutable GameState methods
         ☐ Print undo log at end of lightning bolt example

--------------------

When printing EntityIDs, let's print them more concisely. Right now, we have a very ugly printout of the _phantom field in the Action history:

```
Action history:
  1: MoveCard { card_id: EntityId { id: 3, _phantom: PhantomData<mtg_forge_rs::core::card::Card> }, from_zone: Hand, to_zone: Battlefield, owner: EntityId { id: 0, _phantom: PhantomData<mtg_forge_rs::core::player::Player> } }
```

Let's simply print IDs as "3" "0" etc.

--------------------

Now that the UndoLog is integrated, let's actually use it. At the end of the lightning game, unwind the state by applying the UndoLog one action at a time, printing the Game State in the normal human readable format each time.

Add all the counter types
----------------------------------------

Our `CounterType` is currently a newtype wrapper around a String. This is not faithful to `CounterEnumType` in the Java implementation. An enum is more efficient than a string, more complete, and more typesafe. Port the entire `CounterEnumType` over to an analogous enum in our Rust codebase.

-----

You can delete the code having to do with colors and graphical display. There will be no graphical display for this game engine.


Create a Makefile
----------------------------------------

Create a Makefile just as a simple reminder of the basic actions to take on this repository.
For example, `make build` can run `cargo build`. And `make test` can run cargo test, but `make validate` should be the more expensive, pre-commit validation and should run both the unit tests and any e2e examples like the lightning_bolt_game.


DONE: Implement more of an engine driving the game with callbacks to the player/controller
------------------------------------------------------------------------------------

Right now our `lightning_bolt_game.rs` example directly mutates the game by calling a series of methods on it (play_land, tap_for_mana, cast_spell, etc).

We need to move toward a state where the engine drives the game and the player (AI or human) drives the CHOICES through either a UI (such as the text UI) or through an API.

Let's start to move towards that. The lightning bolt example should set up the initial game state, but then it should pass control to the engine. Examine how the Java version does this and make informed choices. There should be some kind of callbacks passed into the engine for as an instance of the controller. In this case, we want to make a simplistic controller that just plays a mountain, taps a mountain, then plays a lighting bolt.

But this means that we already need to begin developing the API for how the gamestate (including battlefield/hand) is presented to the PlayerController, and how that player chooses from available actions. First, explain how we would implement this Lightning Bolt example in the Java APIs, and then come up with a suitable version for our Rust reimplementation.

--------------------

Improvement: I notice in the current GameStateView, there are some clone() calls in `battlefield()` and `hand()`. Let's avoid copying `Vec`s into freshly allocated heap objects where possible. Instead, return an iterator with the same lifetime as the original reference to the GameStateView.

--------------------

Create a third variant of the lightning bolt test -- in this case use the new deck loading capability to read in an intiial state where both players already have a land on the battlefield (and are at life 11 and 12 respectively). It's P1's second turn and they have a Lighting Bolt in their hand, which they then cast. We don't complete the game but we verify that P2's life total dropped to 19.

--------------------

Build a main binary entrypoint (the default target of `cargo run`) which implements what will become our main CLI command. For now, it will just have one subcommand `mtg tui <deck1> <deck2>`.  We won't implement the full interactive TUI yet, but for now set both players to the random AI player and execute the game till completion. Give both players the basic lightning bolt deck for an initial test.


Minor vcs history surgery
----------------------------------------

AI Accidentally committed extra stuff, so had to split resulting in this:

    commit 179b2e88d1ff7d8d2597505ce46b5028ec13c5ea (HEAD, main)
    Author: Claude Code <claude@anthropic.com>
    Date:   Mon Oct 20 08:27:06 2025 -0400

        Update beads and add Cargo.lock


SWITCH off of beads to a single TODO.md file
----------------------------------------

It's having an install problem in a container and it isn't worth
messing with because right now the agent is just using a SINGLE issue
as a tracking list. That is no better than a single TODO.md. Maybe
when things get more complicated we could explode the task graph into
lots of small beads issues.

Actually, will just ask the agent to do this for now:

Let's migrate off of beads (`bd`) for now. Move the single tracking
issue to a top-level TODO.md file instead. Remove beads from version
control and update Claude.md appropriately to explain the new issue
tracking.


done? Fix github CI
----------------------------------------

The CI is failing on clippy and formatting. Make sure `make validate` runs these and that therefore they get allocated before we make any commit.

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`

--------------------

After the message `Loaded 31438 cards`, print the amount of time it took in milliseconds.


Fix the infinite loop in AI-vs-AI example
----------------------------------------
This seems to be an infinite loop. Running the ai_vs_ai_game example does not terminate for me and just prints this message again and again. Please analyze the code for its looping behavior and fix the bug.

```
$ cargo run --example ai_vs_ai_game
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running `/home/newton/work/mtg-forge-rs/target/debug/examples/ai_vs_ai_game`
=== MTG Forge - AI vs AI Game ===

Demonstrating:
  - Complete game loop with turn phases
  - Priority system
  - Random AI controllers
  - Win condition checking

Warning: cardsfolder not found, using simplified manual cards
Running simplified game with manual card creation...

Created simplified decks:
  - 20 Mountains per player
  - 20 Lightning Bolts per player

Starting hands drawn:
  - Alice: 7 cards
  - Bob: 7 cards

=== Starting Game Loop ===

Warning: Priority round exceeded max actions, forcing exit
Warning: Priority round exceeded max actions, forcing exit
Warning: Priority round exceeded max actions, forcing exit
Warning: Priority round exceeded max actions, forcing exit
Warning: Priority round exceeded max actions, forcing exit
Warning: Priority round exceeded max actions, forcing exit
```


Run ALL examples and add one for combat
----------------------------------------

Both `make examples` and the Github CI script are running ONLY one
lightning bolt example. As mentioned in Claude.md, we need to keep
our examples all running as part of testing. 

To this end, rather than try to keep our Makefile and our github CI
always up-to-date let's dynamically run ALL examples. It's possible to
list which are available by running with no argument:

```
$ cargo run --example
error: "--example" takes one argument.
Available examples:
    ai_vs_ai_game
    lightning_bolt_game
    lightning_bolt_game_v2
    lightning_bolt_game_v3
```

So let's have a little shell script, `run_examples.sh`, which
dynamically discovers the examples and runs them all, only exiting
with a success code if all examples do.

After you've got this working, commit it, and then write one more
example with a controlled starting point which demonstrates an attack,
blocking, and combat damage. You may select the cards in the starting
state to use, but for all examples use basic older cards from limited
to fourth edition (set "4ED").


Upgrade combat damage demo to use game loop
----------------------------------------

The current demo demonstrates the basic building blocks of attackin /
declaring blockers and assigning damage. But this and future demos
need to be written at a high level where they use the GameLoop to
drive the logic (like the ai_vs_ai_game demo).

A demo like this should pass in a custom Controller with specialized
logic for the particular example. The engine needs to ask the Alice/P1
Controller to declare attackers, to which it responds by declaring two
attackers (bear and ogre). After which the hardcoded P2 controller is
asked what blockers and it declares two blockers.



Start adding TUI support
----------------------------------------

The main binary entrypoint for this project should be an `mtg` binary
with an `mtg tui` subcommand.  At minumum the tui CLI takes two deck files.

We will iterate on this TUI implementation until it has feature parity
with the current Java one, described below. A first milestone will be
to be able to execute a game to completion betwoon two decks, while
always naively making the first choice in every menu (e.g. the `--p1=zero
--p2=zero` options described below).

### Use the Java TUI for comparison

The Java TUI is incomplete, but the underlying engine is much more
complete than what we've implemented in Rust so far. Thus you should
run the Java TUI and examine its output to understand the gap between
what it does and

```
[node@e9df094c0507 /workspace/forge-java]  $ decks=`pwd`/forge-headless/test_decks/

# Run the TUI but always choose the first option for P1:
yes 0 | ./headless.sh sim -d $decks/monored.dck $decks/monored.dck

# Run an AI-vs-AI simulation that prints more minimal log output.
./headless.sh sim -d $decks/monored.dck $decks/monored.dck
```

And we can optionally use the `headless.sh` script to invoke the TUI.

```
root@c8d7e426b2af:/workspace/forge-java# 
./headless.sh tui `pwd`/forge-headless/test_decks/monored.dck `pwd`/forge-headless/test_decks/monored.dck
```

You can see the full CLI options for the Java forge-headless TUI here:

```
$ cd forge/
$ ./headless.sh tui -h
=== Forge Text UI Mode ===
Text UI Mode - Interactive Forge Gameplay

Usage: forge-headless tui <player1_deck> <player2_deck> [options]
       forge-headless tui -h | --help

Arguments:
  player1_deck  - Deck file (.dck) or deck name for player 1
  player2_deck  - Deck file (.dck) or deck name for player 2

Options:
  -h, --help             - Show this help message
  -f <format>            - Game format (default: Constructed)
  --p1 <type>            - Player 1 agent type (default: tui)
  --p2 <type>            - Player 2 agent type (default: ai)
                           Agent types: tui, ai, random, zero
                           - tui: Interactive via stdin
                           - ai: Forge built-in AI
                           - random: Random valid choices
                           - zero: Always pass (choose 0)
  --askmana [true/false] - Prompt for mana abilities (default: false)
  --numeric-choices      - Use numeric-only input (no text commands)
  --seed <long>          - Set random seed for deterministic testing
  --start-state <file>   - Load game state from .pzl file

Note: Flags support both --flag=value and --flag value syntax

Examples:
  forge-headless tui --help
  forge-headless tui a.dck b.dck
  forge-headless tui deck1.dck deck2.dck --p1=ai --p2=ai
  forge-headless tui deck1.dck deck2.dck --p2=tui
  forge-headless tui deck1.dck deck2.dck --p1=random --seed=12345
  forge-headless tui deck1.dck deck2.dck --start-state=puzzle.pzl

During gameplay, you will be prompted with options:
  0. Pass priority (do nothing)
  1-N. Play lands, cast spells, etc.

Interactive commands during gameplay:
  ?  - Show help
  v  - View cards in detail
  g  - View graveyards
  b  - View battlefield/game state
  s  - View current stack
```

[Continuing]
----------------------------------------

Above it looks like there was a bit of manual testing (test_deck.dck). But let's actually add that test deck to ./test_decks/ and add an e2e test that replicates the result
there by invoking the TUi.

--

Note that if two random agents play eachother, they should succeed in casting mountains and casting lightning bolts to damage eachother before their respective decks run out. Keep iterating until enough of the Controller choice framework works that this outcome is possible (successfully playing mountains and bolts via deterministic random actions for both agents p0 and p1).

---

You didn't explain how to run the e2e tests or whether they are consistently run by `make validate` and `github workflows`.  How do you run them?

Let's be more verbose in e2e tests
--------------

If I run one of your e2e tests I noly see the final outcome of the game:

```
$ cargo test test_tui_random_vs_random_deals_damage -- --nocapture
...
running 1 test
Random AI loses with -4 life.
Random AI wins with 2 life!
test test_tui_random_vs_random_deals_damage ... ok
```

Above, you successfully ran the Forge Java TUI and saw that it has verbose output for every action taken, every turn (including listing battlefield state), every step within the turn. Let's start adding incrementally to this GameLoop and TUI to more closely match the Java TUI and also see more of what is happening with our agents during the game.


Study the ability syntax (DSL) and document it
----------------------------------------------

Above, when it came to parsing the ability syntax in the lightning bolt card, you took many shortcuts and hardcoded certain things above to push the lightning bolt example through. That's fine, but we need to keep track of where we're at relative to the larger goal of parsing ALL cards accurately and playing full games of MTG.

Study the syntax used for cards in a sample of >30,000 cards stored here:

```
./forge-java/forge-gui/res/cardsfolder/
```

Also study the ability parser in the Java version. In a design doc somewhere in this repository, write a description of the syntax for the card description language. Or reference such a spec in the Java repository if it exsists.  Then, in the TODO file you can reference this spec and summarize what works and what we have left to implement in our basic initial parser.

---

After we have a clear notion of what remains on card parsing, do the next increment of work by selecting 10 random cards of the 30,000 and pushing through a generalization of the parser until they all parse. This will require separate tests for card loading.  Eventually, we will test the card loading of all the cards stored in that folder.


Implement discard phase
----------------------------------------

One missing game mechanic is the discard step. In this test I noticed 39 cards in hand:

```
$ cargo test test_tui_random_vs_random_deals_damage -- --nocapture
  Player 1: 11 life, 39 cards in hand, 19 in library
```

If a player has more than 7 cards at the end of their turn, they must choose cards to discard down to 7. (Unless their max hand size is modified by the effect of a spell.)

Implement our first benchmark
----------------------------------------

We are already accumulating some performance anti-patterns that we
will need to fix. But first, we need to be able to measure how we are
doing in order to improve it.

We already have enough basic game mechanics working to play random
games and time them. The `test_tui_random_vs_random_deals_damage` test
can serve as an initial basis for the benchmark, and later we will
replace the lightning bolt deck with more complicated decks for better
coverage.

The basic idea is to run through a random game and see how long it
takes. Make sure the RNG is deterministically seeded for this
benchmark so that the test is deterministic. We will want to be able
to run in a quiet mode where the verbose stdout logging is disabled
during the game loop and instead we only return the final outcome and
gamestate.

We will need more instrumentation to count the metrics we want. Return

- average cycles/turn, 
- actions/sec, 
- turns/sec. 
- allocation bytes/turn
- allocation bytes/sec

For our metrics, "actions" can be a count of everything added to the
UndoLog. In fact, the UndoLog could do double duty as a counter of
actions.

The code for the benchmark should be parameterized to take a parameter
N and run the benchmark N times. We will use Criterion.rs to get a
more accurate sense of the marginal per-game execution times, exposing
control of the N parameter to Criterion.

The reason we want control of the loop that runs N iterations is to
support three different modes for this benchmark:

1. Fresh -- destroy and allocate a fresh game on each loop iteration.
2. Rewind -- use the undo log to rewind the game from the end all the way back to the beginning before starting the next iteration.
3. Snapshot -- create a snapshot of the gamestate before we run the
   game, and restore it at the end of the iteration. (This should be
   almost identical to the Fresh variant, but there may be
   opportunities for either optimizations or inefficiencies to make
   them perform differently.)

Our goal for the project is to achieve very fast single-threaded
execution, and later we will add parallelism as we explore the game
tree.

----

This looks good. Keep working to complete it along the lines of what
you mentioned above. But first, add "games/sec" to the other
metrics. Then add the allocation tracking.

Also, add a `make profile` target which will run the benchmark (just
one, not multiple different seeds) with profiling turned on and export
the results. Use `cargo-flamegraph` to do perf-based profiling.

Then get back to the other benchmark TODO items you mentioned.

Heap profile with heaptrack
----------------------------------------

We have more allocation per game than we should.  Let's use Linux
heaptrack to profile our allocation hotspots. Use cargo-heaptrack for
convenience.

For the heap profiling run we can invoke the profiling binary with
only, say 1-100 iterations rather than the larger number for Criterion
time profiling. To that end make it take an optional CLI argument for
the iterations.


```
game_execution/fresh/42 time:   [146.60 µs 147.20 µs 147.92 µs]
                        change: [+2.8323% +3.3045% +3.7688%] (p = 0.00 < 0.05)
                        Performance has regressed.
```



TODO: why do we spend soo much time in the loader
------------------------------------------------

[It's simply because we're loading all 31438 cards even though we don't
need to. And 1000 games was NOT enough to amortize the cost.  5000
amortizes it more, but this is overall silly.]

```
$ perf record -g --call-graph dwarf target/release/profile
$ perf report | head -n50
# To display the perf.data header info, please use --header/--header-only options.
#
#
# Total Lost Samples: 0
#
# Samples: 2K of event 'cycles:P'
# Event count (approx.): 2245400649
#
# Children      Self  Command  Shared Object              Symbol
# ........  ........  .......  .........................  ............................................................................................................................................................................................................................
#
    54.32%     0.19%  profile  profile                    [.] mtg_forge_rs::loader::database::CardDatabase::load_directory
            |
             --54.27%--mtg_forge_rs::loader::database::CardDatabase::load_directory
                       |
                       |--36.29%--mtg_forge_rs::loader::card::CardLoader::load_from_file (inlined)
                       |          |
                       |           --36.10%--std::fs::read_to_string (inlined)
                       |                     std::fs::read_to_string::inner
                       |                     |
                       |                     |--16.11%--std::fs::File::open (inlined)
                       |                     |          std::fs::OpenOptions::open (inlined)
                       |                     |          |
                       |                     |           --15.97%--std::fs::OpenOptions::_open
```


DONE Fine-grained async card loading
----------------------------------------

It's pretty silly that we load all 31438 cards even when we don't need to. Rust has excellent support for async programming. Let's rework the CardDatabase to support on-demand async loading.

The idea is that when we load a deck, we will call `get_card` on each card in the deck (say, 20 distinct cards) and each one will return an async future. That will begin the IO process that uses tokio::fs to perform file operations in parallel (on Linux this should automatically use tokio-uring). Then, to complete deck loading, we will force all the futures waiting for the IO to complete and resulting in all our `CardDefinition`s residing in memory.

When doing this kind of "sparse" access to the on disk cardsfolder, we can rely on the known layou of `./cardsfolder`. So when we want to fetch card "Lightning Bolt" we don't need to search through directories, because we know it will reside in `cardsfolder/l/lightning_bolt.txt` (i.e. convert to lower case, space to underscores, and at an intermediate directory based on first letter).

Each time we load a deck in this way print timing for deck loading:

```
Loaded deck of 20 distinct cards in <TIME>
```

Mainly for testing purposes, let's ALSO have a flag --eager-load-cards which uses tokio (and tokio::fs) to recursively walk the `cardsfolder` directory and load everything it finds. This should also be maximally parallel and use all available system resources, and it should print a timing message when it completes:

```
Loaded card database with 31438 cards in 339.79ms
```

----

Let's rename deck_async's load_deck_cards as prefetch_deck_cards. With the new async database it should be FINE to let the cards load the first time we access them. This method is essentially just a prefetching hint. We could even asynchronously combine it with other initialization work.

Update the profile.rs example to use this prefetching method and NOT load the entire card database. It should still report how long teh deck prefetch took.


: Can we name the binary "mtg" while keeping the package name?
------------------------------------------------

The repository and folder are named `mtg-forge-rs`.
But I would like the binary to be simply `target/release/mtg` 
instead of `target/release/mtg-forge-rs`.

Is this possible?


Use streaming for card file discovery 
--------------------------------------------

The following code waits until we walk the directory (probably about 10ms)
before we begin loading any card, which is unnecessarily synchronous:

        // Recursively collect all .txt file paths
        let paths = Self::collect_card_paths(&self.cardsfolder)?;

Instead, we should asynchronously perform the tree-walking IO in
parallel.  There may be a good library function to use for
this. Probably try jwalk.  For example, when there are multiple
children the tree walking process can fork and proceed in parallel,
asynchronously merging the substreams of results. It should return a
stream of paths that we can start consuming and loading immediately.

Thus it is important that the individual card loading and parsing
begin while the directory traversal is still going. Creating optimal
performance for the combined workloads may be a bit tricky. Jwalk will
spawn its own workers and use Rayon. Async_walkdir would work along
with the tokio async runtime we're already using but it does NOT use
parallelism for multiple children (subdirectories).

Eagerly load all cards
----------------------------------------

As noted in the TODO, finish --eager-load-cards. Switch the default
for `mtg tui` to load only the cards for the two input decks.  But
provide the option to load all cards.  Actually, name the flag
`--load-all-cards` instead.



Reduce the TODO.md description of past work
----------------------------------------

Let's leave a very short description of phase 1 & phase 2, and compress the description of the already completed portions of phase 3. Leave the descriptions of future work.


Make card load errors fatal
----------------------------------------

We should in general NOT have Warnings and non-fatal errors in our system. We want to fail fast if anything is wrong.

```
eprintln!("Warning: Failed to parse card {}: {}", name, e);
```

Make this warning fatal and add TODO items to our backlog for any
other places you see silent failures.

TODO: CLI argument validation should also be fatal and early
----------------------------------------

I ran this command with a typo and noticed that execution got pretty far BEFORE the error mesage indicating that `tue` is not a valid value for `--p1`:

```
time cargo run --release --bin mtg -- tui test_decks/simple_bolt.dck test_decks/simple_bolt.dck --p1=tue
```

All argument validation should happen early and exit with a help message.  The problem here is that we are using a basic `String` type for CLI arguments `p1` and `p2`. Remember, as per CLAUDE.md, WE HATE STRINGS. Prefer strong types. We want these arguments to be enums and update the clap argument processing appropriately.


Provide a control for verbosity level when running games
----------------------------------------

This output is still minimal:

```
$ cargo run --release --bin mtg -- tui test_decks/simple_bolt.dck test_decks/simple_bolt.dck --p1=random --p2=random
=== MTG Forge Rust - Text UI Mode ===

Loading deck files...
  Player 1: 60 cards
  Player 2: 60 cards

Loading card database...
  Loaded 4 cards in 0.70ms

Initializing game...
Game initialized!
  Player 1: Player 1 (Random)
  Player 2: Player 2 (Random)

=== Starting Game ===


=== Game Over ===
Winner: Player 2
Turns played: 89
Reason: PlayerDeath(0)

=== Final State ===
  Player 1: -1 life
  Player 2: -1 life
```

Add a flag to control verbosity levels. At verbosity level 1 we can print the O(1) messages above, i.e. game outcome. Let's make verbosity 2 the default, and at that level let's at least print:

- every turn (with battlefield/game state at start of turn)
- every step of every turn
- actions like which card you draw, and cards added to the stack and resolving


---

Right now the printing is pretty wasteful of vertical space with blank lines:

```
--- Declare Attackers Step ---

--- Declare Blockers Step ---

--- Combat Damage Step ---

--- End of Combat Step ---
```

Remove these extra newlines. Also, we currently show spells resolving but not going onto the stack. Let's show spells added to the stack as well.

Also, let's move the verbose per-step headers into the top verbosity level (`--verbosity=verbose`). For the normal verbosity level, let's do something complicated. Let's LAZILY print the intra-turn-step section header before the first action / log line that happens in that step. That is, if the lightning bolt is cast, we print the header here so we know which phase it happened in:

```
--- Upkeep Step ---
  Lightning Bolt resolves from stack
```

But if nothing happened we just elide it.

----

If it's feasible, let's allow the numbers 0-3 as settings for `--verbosity` as well. This may require using clap's more advanced features for parsing the CLI argument into the enum.

---

We are making many checks to `print_step_header_if_needed`:

```
self.print_step_header_if_needed();
println!(...);
```

This is ugly. Instead wrap into a helper function the process of logging an action line, and let that helper function encapsulate the check if the lazy-header-printing is needed. This is probably a good time to factor the logging support into separate, modular code, rather than just using naked println! calls throughout the code.


Fix Remaining nondeterminism
----------------------------------------

With controlling the seed, random games should be completely deterministic. But I notice if I run this command multiple times I see "5 life" above 5% of the time and "2 life" the other 95%:

```
$ time cargo run --release --bin mtg -- tui test_decks/simple_bolt.dck test_decks/simple_bolt.dck --p1=random --p2=random --seed=41
=== Final State ===
  Player 1: -1 life
  Player 2: 5 life

$ time cargo run --release --bin mtg -- tui test_decks/simple_bolt.dck test_decks/simple_bolt.dck --p1=random --p2=random --seed=41
=== Final State ===
  Player 1: -1 life
  Player 2: 2 life
```

There is remaining nondeterminism. Dig into the RNG management and seeding and find the source of this bug.

Now that we have more logging of game activities, let's use that to build an e2e test for determinism:

- Make any printout lines that are nondeterministic (i.e. include time elapsed) go to stderr rather than stdout.
- Because stdout will be deterministic, we can run the same random game 10 times (with --verbosity=3) and diff the logs to see if there was any deviation.
- For this and other tests let's structure the test be instantiated as a distinct test for every deck under `test_decks/*.dck`. When we add more decks there they should become new test cases automatically.

Keep working until you have this e2e determinism test passing.

----




TODO: Eliminate unnecessary calls to collect or clone
-----------------------------------------------

We have far too much allocation right now, and, as we cover in CLAUDE.md, one of our design goals is to really minimize allocation.  Note that current cargo flamegraph profiling results show a lot of time spent in free/malloc and drop_in_place. And if you look through our top allocation sites:

```
heaptrack_print heaptrack.profile.67034.gz | grep -E '( calls with | at src | at /workspace)' | head -n50
61600 calls with 0B peak consumption from:
      at /workspace/src/game/game_loop.rs:620
      at /workspace/src/game/game_loop.rs:126
37147 calls with 209.34K peak consumption from:
      at /workspace/src/core/types.rs:18
      at /workspace/src/loader/card.rs:57
      at /workspace/src/loader/database.rs:54
32318 calls with 457.21K peak consumption from:
      at /workspace/src/loader/database.rs:54
31964 calls with 4.58M peak consumption from:
      at /workspace/src/loader/card.rs:67
      at /workspace/src/loader/database.rs:54
31438 calls with 0B peak consumption from:
      at /workspace/src/loader/database.rs:54
128900 calls with 0B peak consumption from:
      at /workspace/src/game/game_loop.rs:745
      at /workspace/src/game/game_loop.rs:650
      at /workspace/src/game/game_loop.rs:126
25674 calls with 0B peak consumption from:
      at /workspace/src/core/mana.rs:68
      at /workspace/src/loader/database.rs:54
```

The culprits are generally calls to `.collect()`. For example:

```
let players: Vec<_> = self.game.players.iter().map(|(id, _)| *id).collect();
```

Calls to `.clone()` are also problematic as well.  These can both
usually be replaced with a zero copy pattern.  For example, you can
directly use the iterator on the players rather than collecting them
into a Vec. This may require more careful lifetime management.

For example, the function `get_card` returns a freshly allocated
CardDefinition. Instead, it can return a reference to the
CardDefinition inside the CardDatabase, and leave it up to the caller
whether they want to clone it. But this requires a lifetime parameter,
and that the returned CardDefinition reference have the same lifetime
as the `self&` reference to the CardDatabase.

Fix what you can in this initial optimization commit, and report what
change it has on benchmark results. Create a section in the TODO list
to document a backlog of any extra optimization opportunities that you
do not fix in your first commit.



TODO: Report aggregated metrics in benchmark
----------------------------------------

Each call to run_game_with_metrics below returns GameMetrics but we throw them away:

            b.iter(|| {
                run_game_with_metrics(&setup, black_box(seed))

Instead implement (or derive) the addition operation for GameMetrics
and add together the GameMetrics from each iteration.

At the end of all iterations, print them out, the same as for the "warmup" run.



TODO: Don't reveal hidden information
----------------------------------------

IF player one is a human (TUI), and player two is not, then don't reveal what card P2 draws.



TODO: Eliminate invalid actions from choice list
----------------------------------------

```
  Error: InvalidAction("Cannot play more lands this turn")
```



TODO: Arena-based allocation for intra-step temporaries
-------------------------------------------------

The temporary allocations...


TODO: Performance Anti-patterns to find and fix
----------------------------------------

TODO: Bad choice tree - combinatorial explosion of blocker/attackers
--------------------------------------------

It is silly to enumerate ALL possible (blocker,attackers) mappings. If
there are 10 attackers and 10 blockers this would quickly grow to many
possibilities.

First of all, why does `DeclareBlocker` allow multiple attackers? In MTG more than one blocker can block an attacker, but a single blocker cannot block MULTIPLE attackers.  Unless there is some special card with this ability. Is there? If so what is it called.

Look at how the Java TUI structures combat as a tree of choices. For each declared attacker, you can assign 0 blockers or you can add one additional blocker. After you add one, you can assign a second or be done (always option 0) and leave it at a single blocker. Then the process repeats for the second attacker.


TODO: Overhaul Controller choice framework and split into two layers
----------------------------------------

Further continuing with improvements to the controll interface, we actually want to have TWO different notions of a controller. The Java PlayerController has over 100 methods that have the agent answer very specific questions (which card to play, discard, which creature to block with, etc).

These targetted callback methods are better for implementing a user interface for humans (graphical or otherwise) OR for implementing the kind of heuristic AI that exists in the Java implementation. We should have a similar layer for the primary player Controller interface. It can start off as a subset of the Java one --- corresponding to the MTG game functionality that our limited prototype implements. We will then grow it over time.

Finally, on top of that we should have a DecisionTree type that wraps a Controller and translates it into a "pick option 0-N" series of decisions, similar to the current `choose_action` method. However, to truly reduce the game to a series of decision branches, we will not have very strong TYPES for those different options. Similar to the Java TUI, we may only have:

(1) a `&str` description of what the choice to make is
(2) a vector of `&str` descriptions of each option, 0-N.



TODO: use a popular and appropriate parser library
----------------------------------------

The 


TODO: get rid of any silent failure / ignoring in parser
--------------------------------------------------------

Therefore if it gets ANYTHING it doesn't recognize.

Parallel card loader test.




Skip-ahead choice to reduce depth of tree
----------------------------------------


TODO: Use real cards and make card loading fully async
----------------------------------------------




TODO: port heuristic AI from Java
----------------------------------------



TODO: agentic LLM for playing the TUI directly
----------------------------------------

TODO: begin researching OpenSpiel for understanding requirements
----------------------------------------------------------------






Comparison to transcript of Java forge-headless games
--------------------------------------------------------------------------------

```
sudo dnf install 
```


