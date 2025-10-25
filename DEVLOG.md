a
Look at TODO.md, Claude.md, and PROJECT_VISION and get ready to work.

Dependencies:

```
sudo dnf install -y golang
go install github.com/steveyegge/beads/cmd/bd@latest
export PATH=$HOME/go/bin:$PATH
```

Generic Forward progress instruction
----------------------------------------

Now you select a task to make forward progress. Review the context:

 - TODO.md
 - CLAUDE.md
 - Game rules: rules/02_mtg_rules_condensed_medium_length_gemini.md
 - PROJECT_VISION.md
 - Relevent feature of the source material, Java implementation: ./forge-java

Then select a task, make forward progress, and commit it after `make
validate` passes. As described in the project vision, the goal is to
keep expanding the implementation until the full game of MTG is
playable, including all cards and arbitrary decks, including the 6690
.dck files in forge-java/.

If you become completely stuck, write the problem to "error.txt" before you exit.

If you are successful, and `make validate` passes, then commit the
changes. Finally, push the changes (`git push origin main`). If there
are any upstream commits, pull those and merge them (fixing any merge
conflicts and revalidating) before pushing the merged results.



Playing around with claude non-interactive
----------------------------------------

```
time claude --dangerously-skip-permissions --verbose -c -p "$(cat prompt.txt)"
time claude --dangerously-skip-permissions --verbose --output-format stream-json -c -p "$(cat generic_forward_progress_task.txt)"
```

That's a NICE output stream... I should probably tee it to a file and pretty print it with jq...

---

Write a simple script to drive claude, `gogo_claude.sh ITERS`:

- select `LOG=./logs/claude_workstreamXY.jsonl` as the output log, where we increment XY until we find a filename that doesn't already exist.

- Run `time claude --dangerously-skip-permissions --verbose --output-format stream-json -c -p "$(cat generic_forward_progress_task.txt)"`
- tee the output to `$LOG`, storing the full streaming json output for later
- pipe the output to jq and extract the "result" field of every json message with type="result".
- claude produces unquoted unicode symbols, for checkmarks etc. You may need to first pipe the ouput through something to quote these, or you will get an error like `jq: parse error: Invalid string: control characters from U+0000 through U+001F must be escaped at line 7, column 2`

After each iteration of the above:
- if `error.txt` exists, exit our script with an error code and print the error.
- if we exited successfully, then repeat the loop again to do more work
- if we have completed ITER iterations successfully, then exit our script with a success code.


-----

cat claude_workstream02.jsonl | jq  -r 'select (.type == "assistant" or .type == "result") | [.message.content.[0].text, .result]'


Trying the github MCP tool
----------------------------------------

claude mcp add --transport http github https://api.githubcopilot.com/mcp -H "Authorization: Bearer
YOUR_GITHUB_PAT"

My first 4 attempts at making a token DID NOTHING.  Reducing the time from 90 days to 30 seems to have helped.

```
 $ claude mcp add --transport http github https://api.githubcopilot.com/mcp -H "Authorization: $(cat ~/.github/access_token_PAT_secret.txt)"
Added HTTP MCP server github with URL: https://api.githubcopilot.com/mcp to local config
Headers: {
  "Authorization": ...
}
File modified: /root/.claude.json [project: /workspace]
```

EXCELLENT it was able to fetch and report the CI status for the last 3 commits.

Print tool use
----------------------------------------

```
{
  "type": "assistant",
  "message": {
    "model": "claude-haiku-4-5-20251001",
    "id": "msg_016aCmVNtpUwsz3Cc8WdnHAq",
    "type": "message",
    "role": "assistant",
    "content": [
      {
        "type": "tool_use",
        "id": "toolu_0165CsM3mC8RvNCeXAPvV1wF",
        "name": "Read",
        "input": {
          "file_path": "/workspace/forge-java/forge-game/src/main/java/forge/game/phase/PhaseType.java"
        }
```

Example to see the TOOLS USED:

    cat logs/claude_workstream03.jsonl | jq 'select (.type == "assistant" and .message.content[0].type == "tool_use") | .message.content[0].name'

And here's my rough glance at plaintext messages:

    cat logs/claude_workstream03.jsonl | jq  -r 'select (.type == "assistant" or .type == "result") | [.message.content.[0].text, .result]'


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


why do we spend soo much time in the loader
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

CLI argument validation should also be fatal and early
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

The test `test_determinism_all_decks` lumps all decks together in one test. But `cargo test` has some flexibility in the declaration of tests I think that should allow us to dynamically discover the decks and then produce a series of separate test results titled `test_determinism_simple_bolt` `test_determinism_deckname2` etc.

---

Wait, regarding the determinism tests, this is NOT a very nice solution with `test_all_decks_have_dedicated_tests`. It doesn't even have any access to ground truth on WHICH tests exist, just a hardcode list of `known_tested_decks` which would require manual updating.

Search the web for methods of compile-time metaprogramming in Rust which would allow us to essentially read the test_decks directory at compile time and produce/register code for separate tests that call the helper.

---



Report aggregated metrics in benchmark
----------------------------------------

Each call to run_game_with_metrics below returns GameMetrics but we throw them away:

            b.iter(|| {
                run_game_with_metrics(&setup, black_box(seed))

Instead implement (or derive) the addition operation for GameMetrics
and add together the GameMetrics from each iteration.

At the end of all iterations, print them out, the same as for the "warmup" run.

---

There is still a fair amount of duplication between the fresh and
snapshot modes of these benchmarks.  Please factor out the pieces
where `run_game_with_metrics` can take a function as an input that
initializes the game. For the fresh case this will allocated new, and
for the snapshot case this will perform a .clone().  Everything else
should be shared between the two setups, including the code to print
the metrics.


: Eliminate unnecessary calls to collect or clone
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

----

Many of these accesses to `game.players.iter()` actually don't make
sense. We should not assume a stable ordering for hashmap iteration.
There will never be very many players, so Vec seems like a far more
logical data structure for storing the GameState::players.

Let's change that type and refactor references. But let's go one step
further.  We can abstract some of these searches for the "next player"
or "other players" into helper functions. If we track not just the
turn number and active player (ID) but also the active players index
in the Vec of players.. well, then a `next_player()` method could be
as simple as incrementing the counter (mod vector length).

---

Continue on your backlog of eliminating extra copies (collect and
clone).  If you check the `make bench` output we are still allocating
over 4000 bytes/turn on average.


---- FORK JOIN IN CLAUDE ----

This is amaziing, it did a fork-join on its own.

    ● Since there are still several test files with issues, let me use Task agents to fix them in parallel:

    ● Task(Fix tui_e2e test file)
      ⎿  Bash(cargo test --test tui_e2e 2>&1 | head -50)
         Waiting…
            Compiling mtg-forge-rs v0.1.0 (/workspace)
         error[E0308]: mismatched types
            --> src/main.rs:231:13
         … +11 lines (ctrl+o to expand)
         +7 more tool uses

    ● Task(Fix src/main.rs)
      ⎿  Update(src/main.rs)
         Updated src/main.rs with 1 addition and 1 removal
         +7 more tool uses

    ● Task(Fix bin/profile.rs)
      ⎿  Bash(cargo test 2>&1 | grep -E '(test result: |Running |Doc-tests )')
         Waiting…
         No matches found
         +6 more tool uses


[Current heaptrack results after recent changes]
----------------------------------------

Lots of stupid temporary action lists allocated and returned.

```
root@fd9a887561de:/workspace# heaptrack_print heaptrack.profile.238080.gz | grep -E '( calls with | at src | at /workspace)' | head -n70
128900 calls with 0B peak consumption from:
      at /workspace/src/game/game_loop.rs:783
      at /workspace/src/game/game_loop.rs:688
      at /workspace/src/game/game_loop.rs:160
8800 calls with 0B peak consumption from:
      at /workspace/src/game/game_loop.rs:723
      at /workspace/src/game/game_loop.rs:440
      at /workspace/src/game/game_loop.rs:326
      at /workspace/src/game/game_loop.rs:160
8800 calls with 0B peak consumption from:
      at /workspace/src/game/game_loop.rs:747
      at /workspace/src/game/game_loop.rs:491
      at /workspace/src/game/game_loop.rs:327
      at /workspace/src/game/game_loop.rs:160
8700 calls with 0B peak consumption from:
      at /workspace/src/core/types.rs:871
      at /workspace/src/game/game_loop.rs:224
      at /workspace/src/game/game_loop.rs:224
      at /workspace/src/game/game_loop.rs:323
      at /workspace/src/game/game_loop.rs:160
6600 calls with 0B peak consumption from:
      at /workspace/src/core/types.rs:871
      at /workspace/src/game/game_loop.rs:224
      at /workspace/src/game/game_loop.rs:224
      at /workspace/src/game/game_loop.rs:331
      at /workspace/src/game/game_loop.rs:160
8800 calls with 0B peak consumption from:
8800 calls with 0B peak consumption from:
      at /workspace/src/game/random_controller.rs:54
      at /workspace/src/game/game_loop.rs:327
      at /workspace/src/game/game_loop.rs:160
8800 calls with 0B peak consumption from:
      at /workspace/src/game/random_controller.rs:54
      at /workspace/src/game/game_loop.rs:326
      at /workspace/src/game/game_loop.rs:160
6600 calls with 0B peak consumption from:
      at /workspace/src/game/random_controller.rs:96
      at /workspace/src/game/game_loop.rs:331
      at /workspace/src/game/game_loop.rs:160
6600 calls with 0B peak consumption from:
      at /workspace/src/game/game_loop.rs:331
      at /workspace/src/game/game_loop.rs:160
6600 calls with 0B peak consumption from:
      at /workspace/src/game/game_loop.rs:587
      at /workspace/src/game/game_loop.rs:331
      at /workspace/src/game/game_loop.rs:160
6000 calls with 1.75K peak consumption from:
      at /workspace/src/bin/profile.rs:70
6000 calls with 622B peak consumption from:
      at /workspace/src/loader/card.rs:139
      at /workspace/src/bin/profile.rs:70
6000 calls with 720B peak consumption from:
      at /workspace/src/loader/card.rs:139
      at /workspace/src/bin/profile.rs:70
6000 calls with 2.06K peak consumption from:
      at /workspace/src/bin/profile.rs:70
8700 calls with 0B peak consumption from:
      at /workspace/src/game/game_loop.rs:393
      at /workspace/src/game/game_loop.rs:323
      at /workspace/src/game/game_loop.rs:160
8700 calls with 0B peak consumption from:
      at /workspace/src/core/types.rs:28
      at /workspace/src/game/game_loop.rs:393
      at /workspace/src/game/game_loop.rs:323
      at /workspace/src/game/game_loop.rs:160
6600 calls with 0B peak consumption from:
      at /workspace/src/game/game_loop.rs:627
      at /workspace/src/game/game_loop.rs:331
      at /workspace/src/game/game_loop.rs:160
6600 calls with 0B peak consumption from:
```


: Overhaul Controller choice framework and split into two layers
----------------------------------------

Now for improvements to the controller interface. We actually want to have TWO different notions of a controller. The Java PlayerController has over 100 methods that have the agent answer very specific questions (which card to play, discard, which creature to block with, etc).

These targetted callback methods are better for implementing a user interface for humans (graphical or otherwise) and ALSO better for implementing the kind of heuristic AI that exists in the Java implementation. We should have a similar layer for the primary player Controller interface. It can start off as a subset of the Java one --- the subset corresponding to the MTG game functionality that our limited prototype implements. We will then grow it over time.

Remember our zero-copy principles. As you look at the Java PlayerController version, there are plenty of examples of methods taking and returning `List` types (i.e. heap allocating collections). As much as possible we want to instead pass around lists by using array slice references or mutable iterators. Or, when all else fails, use SmallVec.

Finally, on top of that we should have a DecisionTree type that wraps a Controller and translates it into a "pick option 0-N" series of decisions, similar to the current `choose_action` method. However, to truly reduce the game to a series of decision branches, we will not have very strong TYPES for those different options. Similar to the Java TUI, each coice will ONLY receive:

(1) a `&str` description of what the choice to make is
(2) a vector of `&str` descriptions of each option, 0-N.

We want to be careful even here to not allocate. For instance if the options are different cards, then we get a handle on their existing name rather than copying a string.

----

When all the work is done and `make validate` passes, then do one additional refactor commit for renames only. Get rid of all the "v2" names in filenames and code so the new controller becomes THE controller with no extra complication.


This controller architecture deviates substantially from PlayerController.java
----------------------------------------

We need to pay more attention to our source material here. Examine at least PlayerController.java, PlayerControllerAI.java, and Player.java.

We have split apart choosing a land vs choosing a spell to play. In the Java version, `chooseSpellAbilityToPlay` handles both lands and other cards in hand, plus abilities. It's better to combine these choices as the Java version does.

Furthermore, it is NOT necessary to tap mana before casting a spell. Refer back back to the rules `rules/02_mtg_rules_condensed_medium_length_gemini.md`, section 5.3, paying costs is step 7 of 8 in casting a spell.
Adding the spell to the stack happens before costs are payed. See for example HumanPlaySpellAbility.playAbility in the Java version. The computer AI uses a method called `payManaCost` to accomplish this. In any case, the Controller should be prompted for the choice after choosing which card (spell) to play and it going on the stack.

--

IF you complete the above (controller overhaul) task, and it's
committed with `make validate` fully passing, then please tackle this
next task.

Compute which mana costs are payable
----------------------------------------

We're going to start moving towards presenting only FEASIBLE, legal
options.  I.e. don't ask which card to play and include non-playable
cards. This will help us get to the point where the random controller
does not create any illegal actions that crash the game OR puts spells
on the stack that are not able to have their costs met. (Though in the
latter case I think MTG may have rules for what happens if a spell is
on the stack but costs are not paid.)

In order to move in this direction let's add a function for computing
whether, given the current board state, the active player is ABLE to
produce a given ManaCost.

We cannot simply compute the max mana we can produce of each
color. Because some lands produce, e.g. {R} or {G}, so therefore we
cannot cast a spell that costs one of each, {R}{G}.

In fact, it is a tiny search process to compute ManaCost
payabality. But in practice it should be very cheap.  We should
abstract an object that queries our mana capabilities and maintains
state to precompute or cache for efficient querying. It needs to be
updated every time a new mana source hits the battlefield, and queried
every time we want to know if a cost is playable.

Inside this per-player mana engine, we should store the lands on the
battlefield partitioned into "simple" mana sources and complex ones,
including choices (A or B) and conditional costs (pay A to get B). The
simple sources can indeed just be added up.  The max production in
each color should just be cached in a simple struct (with six counters
RGBUWC). Comparing whether a ManaCost is less than or equal to these
counters can determine playability quickly for simple sources.

For now, stub out handling of the complex sources and just put a todo!
macro in.

Later, we will need to add even more complexity, as in
ManaCostShard.java, for special kinds of mana and costs that can be
paid with "A or B" as well.

More logging of game events: damage, etc
----------------------------------------

If we run a game between two grizzly bear decks, we don't see them being played or attacking.

```
cargo run --bin mtg -- tui test_decks/grizzly_bears.dck test_decks/grizzly_bears.dck
```

This should show messages for at least these actions at the normal --verbosity=2 level:

- spell being put on the stack
- spell resolving
- grizzly bear entering the battlefield
- grizzly bear declarad as attacker
- damage done by grizzly bear to opponent


Print CardId after the name.
----------------------------------------

We often have multiple cards in play with the same name:

    Player 2 declares Grizzly Bears (2/2) as attacker
    Player 2 declares Grizzly Bears (2/2) as attacker

Hence change the basic printing so it shows CardId in parentheses
after the name "Mountain (1)" vs "Mountain (2)". This matches the Java TUI.


Determinism failure.
----------------------------------------

We are having a determinism test failure.

Before fixing it, please improve the output on a test failure. Use an
existing library for pretty colorful diffs, and show a bounded amount
of context for the first diffs between execution 1 and execution 2
when we have a nondeterminism instance:

```
test test_deck_determinism__grizzly_bears ... FAILED
test test_different_seeds_consistency ... ok

failures:

---- test_deck_determinism__grizzly_bears stdout ----

thread 'test_deck_determinism__grizzly_bears' panicked at tests/determinism_e2e.rs:62:5:
assertion `left == right` failed: Deck /home/newton/work/mtg-forge-rs/test_decks/grizzly_bears.dck produced different output with same seed (seed=42)
  left: "=== MTG Forge Rust - Text UI Mode ===\n\nLoading deck files...\n  Player 1: 80 cards\n  Player 2: 80 cards\n\nLoading card database...\n  Loaded 4 cards\nInitializing game...\nUsing random seed: 42\nGame initialized!\n  Player 1: Player 1 (Random)\n  Player 2: Player 2 (Random)\n\n=== Starting Game ===\n\n\n========================================\nTurn 1 - Player 1's turn\n  Life: 20\n  Hand: 0 cards\n  Battlefield: 0 cards\n========================================\n--- Untap Step ---\n--- Upkeep Step ---\n--- Draw Step ---\n  (First turn - no draw)\n--- Main Phase 1 ---\n--- Beginning of Combat ---\n--- Declare Attackers Step ---\n--- Declare Blockers Step ---\n--- Combat Damage Step ---\n--- End of Combat Step ---\n--- Main Phase 2 ---\n--- End Step ---\n--- Cleanup Step ---\n\n========================================\nTurn 2 - Player 2's turn\n  Life: 20\n  Hand: 0 cards\n  Battlefield: 0 cards\n========================================\n--- Untap Step ---\n--- Upkeep Step ---\n--- Draw Step ---\n  Player 2 draws Grizzly Bears (162)\n--- Main Phase 1 ---\n--- Beginning of Combat ---\n--- Declare Attackers Step ---\n--- Declare Blockers Step ---\n---
```

This failure may have to do with the CardIDs. Make sure our
determinism test uses the same cardIDs from run to run.  More
generally, let's make sure that our deck prefetching loads the cards
in order (deck1 card1, deck1 card2, ... deck2 card1 ...) so that they
are assigned consecutive cardIDs deterministically.

(However, when we do async loading of ALL cards obviously that will be nondeterministic.)


I don't like hashmaps in CombatState
----------------------------------------

And we eagerly sort them as well with the most recent change.

1. I think we should use the SmallMap crate for these maps.

2. I think we should leave them unsorted until the final point of
   consumption.  And then sort them IFF verbosity >=
   normal. Otherwise, it's "don't-care nondeterminism" and we can
   leave it.

Fix this and make sure `make validate` passes before commit.

Lightning bolt doesn't show damage
----------------------------------------

We need to show damage to a player consistently irrespective of source.
Creature attacks show damage, but lightning bolt doesn't.

    Player 1 casts Lightning Bolt (24) (putting on stack)
    Lightning Bolt (24) resolves



Experimented with generic self-directed taskprogress...
================================================================================

Here was the final message from the stream-json output.

    {"type":"result","subtype":"success","is_error":false,"duration_ms":310078,"duration_api_ms":258868,"num_turns":72,"result":"Perfect! Everything validates successfully. Let me create a summary of what was accomplished:\n\n## Summary\n\nI successfully implemented **Flying/Reach combat blocking restrictions** for the MTG Forge Rust project, making forward progress on Phase 3 gameplay expansion.\n\n### What Was Implemented\n\n**Feature**: Flying and Reach keyword combat restrictions (MTG rules 702.9 and 702.17)\n\n**Changes Made**:\n1. **Core Card API** (`src/core/card.rs`): Added `has_reach()` helper method\n2. **Combat Logic** (`src/game/actions.rs`): Enhanced `declare_blocker()` to validate Flying/Reach restrictions\n3. **Comprehensive Testing**: Added 5 new tests covering all blocking scenarios\n4. **Documentation**: Updated TODO.md to reflect completed work\n\n### Key Design Decision\n\nThe implementation uses **direct keyword checking** rather than Java Forge's abstract static ability system. This approach is:\n- Simpler and more maintainable\n- More performant (no iteration over static abilities)\n- Functionally equivalent to the Java implementation\n- Sufficient for current needs\n\nFuture complex keyword interactions may warrant adopting Java's static ability framework.\n\n### Testing Results\n\n- **Total tests**: 122 passing (up from 117)\n  - 104 library tests\n  - 10 card loading tests\n  - 3 determinism tests\n  - 5 TUI tests\n- **Validation**: All `make validate` checks pass\n- **Code formatting**: Applied via `cargo fmt`\n\n### Commit\n\nCreated comprehensive commit `0b77174` with:\n- Detailed change description\n- Test results summary\n- Comparison to Java Forge implementation\n- Future considerations\n\nThe project continues to make steady progress toward the vision of a complete, high-performance MTG game engine suitable for AI research.","session_id":"a45d4512-7354-4933-956f-8e820b7994f9","total_cost_usd":0.8743158999999999,"usage":{"input_tokens":55,"cache_creation_input_tokens":59542,"cache_read_input_tokens":1170328,"output_tokens":6154,"server_tool_use":{"web_search_requests":0},"service_tier":"standard","cache_creation":{"ephemeral_1h_input_tokens":0,"ephemeral_5m_input_tokens":59542}},"modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":3428,"outputTokens":6984,"cacheReadInputTokens":989620,"cacheCreationInputTokens":56120,"webSearchRequests":0,"costUSD":0.20746,"contextWindow":200000},"claude-sonnet-4-5-20250929":{"inputTokens":55,"outputTokens":6154,"cacheReadInputTokens":1170328,"cacheCreationInputTokens":59542,"webSearchRequests":0,"costUSD":0.6668559,"contextWindow":200000}},"permission_denials":[],"uuid":"f0e95917-2659-4486-a2ad-e6b0870f98fe"}

I'm getting some problems with parsing on JQ

     [root@f9890d243c0c /workspace]  $ cat claude_workstream01.jsonl | jq 'length'
     14
     5
     5
     5
     5
     jq: parse error: Invalid string: control characters from U+0000 through U+001F must be escaped at line 7, column 2


Update gogo_claude.sh script to take an extra prompt
----------------------------------------

If we call `./scripts/gogo_claude.sh ITERS prompt.txt`, then the extra prompt is
one we handle in the first iteration.  After that first iteration, the remaining
iterations can go back to using the generic prompt.

---

One more thing about scripts/gogo_claude.md. Grab a fresh log file for
EACH iteration, rather than globally at the start of the script.


Write an optimization guide/backlog and improve profiling
----------------------------------------

Our key performance metrics for this project are:

- turns/sec and games/sec: though these will change over time as we add more game features
- actions/sec: should be fairly stable
- avg allocations/turn

We continue to miss zero-copy opportunities and add new methods that
return freshly allocated collections. We need to do periodic
optimization work to push back on this.

Research on the internet best practices for high performance rust and
zero copy patterns. Summarize these findings into OPTIMIZATION.md and
add a "Status and Backlog" section for tracking known inefficiences /
future optimization tasks. Reference optimization work as a
possibility from the main TODO.md and cite the OPTIMIZATION.md as
where to go for that.


In order to make incremental optimization steps, a good prereqisite to
implement first is better scripting around benchmarking and heap profiling.

When you run `cargo bench --bench game_benchmark -- fresh`, you can
directly see some of the key metrics we care about, to get a sense of
where we're at. But heap profiling is less automated. Let's improve that next.

When you run `make heapprofile` you'll see something like this.

```
Profiling complete! 100 games executed.
Heaptrack finished! Now run the following to investigate the data:

  zstd -dc < "/home/newton/work/mtg-forge-rs/heaptrack.profile.820576.raw.zst" | /usr/lib64/heaptrack/libexec/heaptrack_interpret | zstd -c > "/home/newton/work/mtg-forge-rs/heaptrack.profile.820576.zst"

Heaptrack profile saved
To view: heaptrack_gui heaptrack.profile.*.zst
Or use CLI: heaptrack_print heaptrack.profile.*.zst
```

This doesn't take the final step (the line with `zstd -dc`) which is
necessary in order to be able to call `heaptrack_print`. Once you run
that command, you can see the calling context for the most frequent
allocation calls with something like this:

```
$ heaptrack_print heaptrack.profile.??????.zst | grep -A10 'calls with' | grep -E '( calls with | at .*src/)' | head -n70
```

We probably need an extra script (e.g. in ./scripts) in order to
post-process the output of heaptrack_print (which unfortunately does
not have machine readable output like JSON). We can expand the `make
heapprofile` in order to call a subsequent script to (1) process the
raw.zst => .zst, (2) call heaptrack_print, (3) parse a subset of its
output to identify the top stackframe within the projects src/ code,
and (4) print the line of code each stack trace points at.

With this improvement we'll be able to see at a glance what our top
offenders are --- see whether they're a .collect() call, a .clone()
call, a .push() call or something else -- and add them to our backlog
in OPTIMIZATION.md accordingly.


Use real cards in testing
----------------------------------------

It's important that our implementation work for the REAL MTG cards stored in ./cardsfolder.
Look for test cases like this that use fake cards:

        let mut attacker = Card::new(attacker_id, "First Strike Bear".to_string(), p1_id);

Change at least this first strike one to instead load a real
first-strike creature from the cardsfolder.  This will ensure that we
are correctly parsing the card, including its first strike ability.


Why does random player not play grizzly bears sooner?
----------------------------------------

With this command, on turn on 84 the first grizzly gets played:

    time cargo run --release --bin mtg -- tui test_decks/simple_bolt.dck test_decks/grizzly_bears.dck --p1=random --p2=random --seed=41 -v2

If we have options to "pass" or "play grizzly bear", we should be
choosing to play the grizzly bear at least 50% of the time. With
multiple grizzly bears and lands in the hand the odds would be
slightly different but still should be more likely than on turn 84.

That is especially true because creatures should be playable in main1
AND main2, so we should be asked at both times if we want to play the
grizzly bears


Heaptrack analysis doesn't print the lines of code that are the culprits
----------------------------------------

Our scripts/analyze_heapprofile.sh does part of what it should. It
creates a nice breakdown of the top 10 allocation sites:

 1.   77,378 calls,      304.54K
    └─> src/game/game_loop.rs:819
        ├─> src/game/game_loop.rs:222
        ├─> src/bin/profile.rs:85

 2.   45,274 calls,        1.39K
    └─> src/game/game_loop.rs:222
        ├─> src/bin/profile.rs:85
        ├─> src/bin/profile.rs:95

But it doesn't actually print the lines of code at indicated locations which are the culprits. E.g.:

 - game_loop.rs:819
 - src/game/game_loop.rs:222

If we want to see at a glance our top culprits, a single `make
heapprofile` action should give us all this information in one go.

Further, this profiling (as well as `make profile`) pollute the top
level repo directory.

Modify the setup so that `make heapprofile` and `make heapprofile` put
all their temporary files in a directory called `./experiment_results`
that is added to the .gitignore.

-------

Wait, I led you in a bad direction on the last change. We don't need
to do so much work to print the function names. If we have debug
symbols properly working, we will see the function names right in the
output of `heaptrack_print`, like this:

```
8800 calls with 0B peak consumption from:
    mtg_forge_rs::game::game_loop::GameLoop::end_combat_step::hdae80d8ae53fe57a
      at src/game/game_loop.rs:819
      in /home/newton/work/testing-mtg-forge-rs/target/release/profile
    mtg_forge_rs::game::game_loop::GameLoop::execute_step::h43d9d1f35a9c2825
      at /home/newton/work/testing-mtg-forge-rs/src/game/game_loop.rs:453
    mtg_forge_rs::game::game_loop::GameLoop::run_turn::hfb8f5b09a2e0c4eb
      at src/game/game_loop.rs:22
```

In that particular case it's pretty obvious that the `clear()`
function is allocating new maps. That's silly. At the very least it
should be calling the `.clear()` methods on these collections to zero
them out NOT dropping them and reallocating.

It is sufficient to just print out snippets like the above with the
top few lines of context, without additional post-processing.

---

Then we can use our function printing to ALSO ...

Implement simple caching scheme for `make validate`
----------------------------------------

Let's tweak the `make validate` target so that rather than depending
on `fmt-check clippy test examples` targets, it wraps a little more
setup around a recursive call to make.

- first it checks if the current working copy is clean, with no modified tracked files
- select an output log: `experiment_results/validate_<COMMITHASH>.log` if
  the working copy is clean and `experiment_results/validate_<COMMITHASH>_DIRTY.log`
  if it is not.

- if we are clean, then before we run the recursive make (to do the
  work of fmt-check clippy test examples), first we check if the file
  in question exists already, in which case we return immediately.

- if the file does not exist, we do the work of validating and `tee` the output to the log.

- we can make the commit of the log atomic by FIRST writing to a `.log.wip` file and then atomically renaming it to `.log` only once validation completes.

This scheme will prevent us from running validate twice in a row on the same commit.


Make random, not zero, the default agent behavior
----------------------------------------

That's a better default for --p1 and --p2 agent control.


Focus on expanding e2e tests
----------------------------------------

All our recently added card abilities have been tested with unit tests
that load individual cards.  It's also important to keep expanding our
e2e tests, especially by playing full random games between different
decks. For example, we recently added vigilance tests, but let's also
find a deck with Serra angel in it like this one:

```
cat "forge-java/forge-gui/res/adventure/Shandalar Old Border/decks/standard/armadillo.dck"
cargo run --release --bin mtg -- tui test_decks/grizzly_bears.dck --p1=random --p2=random "forge-java/forge-gui/res/adventure/Shandalar Old Border/decks/standard/armadillo.dck"
```

Right now I can play games with that deck but they seem fishy. Player1 does not seem to be taking the random actions they should. They do not play any forests or grizzly bears.

Add at least one new e2e test (like the above) with assertions on what happens during the game.


Focus on getting the undo log to work
----------------------------------------

Now that you've added at least one new e2e test, let's focus on
getting the undo log working.  The `lightning_bolt_game.rs` example
still works and demonstrates basic undo functioning.

Let's expand that to a full (random) game between two decks. If we run
a game that takes 88 turns:

- let's rewind 50% and replay the second half (44 turns)
- then let's rewind 100% to the beginning of the entire game and play it again

That will complete our e2e undo system test. It will demonstrate the
integration of the UndoLog with the GameState, where we are
successfully able to roll back the entire gamestate to a playable
earlier point.


Flesh out battlefield printing
----------------------------------------

This game state / battlefield printing is very basic:

    ========================================
    Turn 6 - Player 2's turn
      Life: 20
      Hand: 2 cards
      Battlefield: 2 cards

Please refer back to the output format of the Java TUI game. It should show which cards are on each player's side of the battlefield, which cards are tapped, and the size of the graveyards / exile zones.


Log each random choice made by the agent
----------------------------------------

Each time the agent is asked to make a random decision, have it
produce a line of output:

    >>> RANDOM chose 3 out of choices 0-9.


done: Fix issues with undo e2e test
----------------------------------------

Setting verbosity to normal and running this undo_e2e test shows that
significant problems remain that need to be fixed.

```
cargo test -- undo --nocapture
```

Just from the snippet below we see:

- The halfway rewind didn't play forward.. It did nothing.
- The controllers should be fairly stateless and ready to continue play from any point
  - If there are any extra requirements for this to hold, we can
    formalize a stateless-playercontroller trait and that should certainly at least hold for randomcontroller!
- When we did replay after the 100% rewind the turn counter kept INCREASING... "turn 89", when it should be set
  backwards by undoing.

    Game completed!
      Winner: Some(1)
      Turns played: 88
      End reason: PlayerDeath(0)
      Undo log size: 72

    === Phase 2: Rewinding 50% of actions ===
    Rewinding 36 out of 72 actions
    After rewind, undo log size: 36

    === Phase 3: Replaying from 50% point ===

    === Phase 4: Rewinding 100% to beginning ===
    Rewinding all 36 remaining actions
    After full rewind, undo log size: 0

    Game state after full rewind:
      P1 life: 20 (initial: 20)
      P2 life: 20 (initial: 20)

    === Phase 5: Replaying entire game ===

    ========================================
    Turn 89 - Player 1's turn
      Life: 20
      Hand: 10 cards
      Battlefield: 0 cards

To be extra strict we should take a SNAPSHOT of the GameState before
turn 1, and then when we do the full rewind we can do a deep
comparison of the rewound gamestate to the snapshot of the original.

---

This represents progress, but it is still NOT playing the game FORWARD after rewinding in phase 2 and phase 3.
That is a critical part of what we want working here. It's not just that we can rewind but that we can play forward after rewinding, and then repeat that process as much as we want.

--

Ok, that's again progress. But this final result makes no sense, where playing forward in phase 4 takes ZERO turns:

```
=== Phase 4: Play forward to completion from beginning ===
Replay completed!
  Winner: Some(0)
  Turns played: 0
  End reason: Decking(1)
Note: Replay completed with 0 turns (may differ from original due to RNG reset)
```

And that the game ended from decking seems to indicate that rewinding is NOT reversing draw steps and restoring cards to the deck, and thus it becomes exhausted.

---

 Wait a second, explain to me why this is a problem. Undo SHOULD be permanent. The goal of the test is to play forward 100%, rewind 50%, then play forward till the end of the game in a
  NEW SECOND HALF -- a brand new evaluation which adds new actions to the undo log. When we are back up to 100% execution at the end of phase2, the first ~half of the undolog should be old
  (from phase1) and the second half should be brand new. That's still a fine starting state for phase3 which will unroll the entire undo log back to the starting point.


moved: Elide random choices with one option only
----------------------------------------

done: Fix validate script
----------------------------------------

The `make validate` implementation has a critical bug where validaiton
will FAIL but it will still cache the result and think it succeeded
the next time.

Let's factor out the validate logic into `./scripts/validate.sh` and
be more careful with it.

- The core validate-impl will stay the same and stay in the Makefile for now
- The validate logs files will move to `./validate_logs/` which will be added to .gitignore.
- The files will be called `validate_HASH[_dirty].log[.wip]`
- We failed to implement the ATOMIC nature of adding validation log and will do that now.
  - During validation runs, log to `.log.wip` and then atomically move it to `.log`
    ONLY once validation succeeds.
- The last successful run will also by symlinked to `validate_latest.log`.


done: validate script upgrade
----------------------------------------

Let's make it more robust to trivial changes in TODO.md or other
documentation/markdown files that don't affect program behavior.

IF there is a dirty working copy, then go ahead and commit a temporary
commit with "wip" as the message. Continue to name it [_DIRTY], and
link it as latest. Uncommit when validation completes, irrespective of
whether it succeeds or fails.

Before we run the real validation, perform a `git diff --stat -r HASH`
to compare (1) the current working copy state to (2) commit
represented by (the target of) `validate_latest.log`.

IF there are no differences, or the only differing files are "*.md",
then let's consider it a cache hit.

We can symlink the `validate_OLDHASH[_dirty].log` to our
`validate_NEWHASH[_dirty].log` to represent the cache hit and the
coverage of the new commit.


done: Migrate to beads for issue tracking
----------------------------------------

Read `bd quickstart` and get ready to migrate our TODO.md tracking to beads.
I've run `bd init -p mtg` to initialize the database.

- create issue mtg-1 should be the OVERALL tracking issue, which primarily will reference other tracking issues
  and document some of these conventions. It will be priority 0. We want to keep it pretty short.

- other tracking issues will have priority 1, and we will move away from "phases" as the main organization
  and instead use a few major topics:
  - Optimization tracking: migrate and remove from the relevant section in OPTIMIZATION.md
  - MTG feature completeness: supporting keywords/abilities/complex mana and effects.
  - Gameplay feautures: like an actual TUI to play as a human.
  - Cross-cutting codebase issues: APIs (player, controller, etc), testing coverage and methodology.

- tracking issues refer to granular issues by name in their text, e.g. "mtg-42"
- all other granular issues will have priority 3 or 4 unless they are seen as a critical bug, which will bump them to priority 1.

- issues labeled "human" are created by me and will always have 0 priority
- you can ignore the "completed work" in TODO.md in this migration
- delete TODO.md when you're sure everything we need has been copied over

### CONVENTIONS:

Our TODO file had TRANSIENT information in it that quickly gets out of
date, like benchmark results. Moving forward we want to have a notion
of timestamp for any transient information. Specificallyy
`./scripts/gitdepth.sh` prints out the number of commits in the repo,
and we can use "commit#161(387498cecf)" as a shorthand that gives us a
rough sense of time (in the repository) that information dates from.

Start adopting the convention of referencing issues from todo comments:

```
// TODO(mtg-13): brief summary here
```


done: make bench showing RANDOM choices it should not
----------------------------------------

These messages should go through the normal logging interface and be at verbosity=2

    >>> RANDOM chose creature 0 to attack (50% probability) out of 3 available creatures

An invariant we want is that make bench should produce O(1) logging output and not spend much time in printing os we can benchmark the real game logic.


--

Let's try out another approach for that. We should have a centralized
object for logging during the game, which INTERNALLY remembers the
verbosity level.  Different parts of the code that need to log should
retain a handle on this logger.  Try to make the controllers follow
that same pattern.  Maybe the gamestate reference could even provide access to the logger.


done: random choices of 1 option still present.
----------------------------------------

Look at an e2e game:

```
$ cargo run --release --bin mtg -- tui test_decks/simple_bolt.dck test_decks/grizzly_bears.dck --p1=random --p2=random --seed=41 -v2

>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose creature 0 to attack (50% probability) out of 3 available creatures
>>> RANDOM chose creature 2 to attack (50% probability) out of 3 available creatures
  Player 2 declares Grizzly Bears (85) (2/2) as attacker
  Player 2 declares Grizzly Bears (83) (2/2) as attacker
  Grizzly Bears (83) deals 2 damage to Player 1
  Grizzly Bears (85) deals 2 damage to Player 1
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
>>> RANDOM chose to pass priority (no available actions)
```

I still see all these choices where the ONLY option is pass
priority. These should NOT be logged as a choice, but should be
suppressed (the player should not be asked the choice).

Our guiding principle here is to only invoke the PlayerController
 - when a choice is needed
 - presenting VALID options only

When `get_available_spell_abilities` returns only a single action
(PassPriority), then we don't need to call the PlayerController.


---

Make an additional change. Whenever `bd show` is going to print multiple issues, present them in order of the IDs: issue-1, issue-2, etc.


done: Task fixing and dedup
----------------------------------------

Note that we have multiple duplicate tasks.
There are especially a set of "error messages in card loader"
tasks which are both duplicate AND polluting priority 0.

Review CLAUDE.md and the conventions for beads issues. Merge the
duplicate issues into one and put it at the appropriate priority.


done: aggressive random undo testing
----------------------------------------


done: Use --numeric-choices when comparing against Java
----------------------------------------

If you do this, you will see the choices to attack or block in a numeric-only choice format:

```
/headless.sh tui `pwd`/forge-headless/test_decks/grizzly_bears.dck `pwd`/forge-headless/test_decks/grizzly_bears.dck --p1=random --p2=ai --numeric-choices
```

For our own TUI, we can follow a simiar design. Declaring blockers for example can be done much more quickly with a rich interactive format ("0 blocks 1" "1 blocks 3" "done" is what Java uses). But if we pass `--numeric-choices` we want to only run the choice oracle in the simples mode that pics 0-N with a prompt.

When this is working well, proceed to start adding more test_decks. You can either sample whole decks from the thousands in forge-java/, or you can make more custom decks that demonstrate more choices.
For example, a deck with royal assasins vs the grizzly bear deck should have the option to tap to kill a tapped grizzly bear in response to its attack.



moved: Begin or progress work on simple interactive TUI
------------------------------------------------------------

The basic `mtg tui --p1=tui` will be very similar to the Java version. Instead
of making a random choice for every `Enter choice (0-N)` prompt, we will present
the choice to the user and wait for input on stdin.

Later we will add a fancy TUI with the ratatui library, so we should keep an eye
on how to make our Controller interface generic. It will present the battlefield
and the choices in a different way.


Overhaul ingest_inbox script
--------------------------------

The script .beads/ingest_inbox.sh is not working to successfully import the entries
in .beads/inbox/*.md into .beads/done/.

It also doesn't present an error message when bd fails, and when it succeeds it doesn't print anything confirming the issue id of the task(s) created.

```
==================================
Processing 1 file(s) from inbox
===================================

Processing: tasks.md
  → Creating issue: Begin or progress work on simple interactive TUI
```

Port this script to python and improve it to fix these deficiencies.

Check if `bd -h` reports that it takes the `--no-db` option, and if it does,
prefer that.

If bd does NOT take --no-db, and bd reports `Error: no beads database found`, then 
the script can switch to `.beads/..` and run `bd init -p mtg` with the appropriate prefix for this project.


TODO: Optimized mode to cut down on choices
------------------------------------------------------------

Following the full magic rules, we will give players priority at every opportunity.
If we are doing aggressive filtering of valid actions, then many of these priorities
should be skipped (only one option).

- I.e. if they don't have an instant or don't have mana to play an instant.
- We won't count mana-actions, i.e. we'll never interrupt the player to ask if they want to tap a land when they technically have priority but no other actions.

In addition to these "always safe" simplifications of priority passing, we will add a `--priority=simple` flag (in contrast with the default `--priority=full`). The simplifying assumption is this:

- With reactive acions you only want to react TO something your opponent does (or triggered abilities).
- With proactive actions (play a permanent, a sorcery/instant on your turn) there are certain canonical times to play them "play a creature during main1". "Play an instant at the end step of my opponents turn."

So from the moment we draw on our turn, we can directly be presented with a set of "fast forward" options:

- pass priority / fast forward to end of turn
- play a creature in your main1
- play a spell in your main1
- attack with creature(s)

If we pick one action, we can of course pick a second and third after.  But if we advance to attacking, then we will only be able to play spells in main2.

Anything else we want to do at other steps in our turn would be a reaction to our opponent casting a spell, declaring a blocker, or a triggered action on our upkeep/beginning-of-combat/etc. So if we want to *react*, we can pass priority and wait to get it back when the relevant events happens, knowing that control will not actually reach the end of our turn.

The benefit is that in very simple games, with only a single simple action or two per turn (draw, play land, play creature), we can greatly reduce the depth of the choice tree.

First let me know if you can find any exceptions to these assumptions based on your understanding of the rules. Then, let's come up with a plan for the simple priority mode.





TODO: Separate seed for initial shuffle vs subsequent game
----------------------------------------

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



MCP server for the agent to play the game
-----


TODO: Abstract the logging framework to redirect logs
----------------------------------------

Sometimes it may help us to capture the logs in memory, for exmaple
during testing.



TODO: fix Mana pools not being emptied
----------------------------------------


TODO: fix BOGUS "Fill in missing targets for effects" in action.rs
----------------------------------------

These should be choices for the player not arbitrary heuristic hacks.


TODO: Port the Java mana system
----------------------------------------




TODO: switch to bump allocator for temporary storage
--------------------------------------------




TODO: Bring back layered design, DecisionMaker
----------------------------------------

In commit e1a819a587e35dd27123087ba0afaf070e03342d, you deleted the
DecisionMaker and DecisionTreeAdapter.  Bring back the DecisionMaker:

```
pub trait DecisionMaker {
    /// Get the player ID this decision maker is responsible for
    fn player_id(&self) -> crate::core::PlayerId;

    /// Make a choice from available options
    ///
    /// # Arguments
    /// * `prompt` - Description of what decision is being made
    /// * `options` - Descriptions of each option (indexed 0 to N-1)
    ///
    /// # Returns
    /// Index of the chosen option (0 to options.len()-1)
    fn make_choice(&mut self, prompt: &str, options: &[&str]) -> usize;
}
```

This trait was correct, but you had it backwards before.  We don't need this:

```
impl<C: PlayerController> DecisionMaker for DecisionTreeAdapter<C>
```

We need the opposite, where, given a DecisionMaker, we can wrap it and
turn it into a `PlayerController`.  The idea is that for each choice
it is asked to make, like choosing a spell/ability to play, it will
package the choice in the format expected by the `make_choice` method.
For example, it will set the prompt as "Pick 1 card to discard." and
the options will correspond to the names of cards.

TODO: Prioritize mana sources
----------------------------------------
AI should prioritize mana sources:

  Colorless-only lands first (save flexible ones)
  Single-color lands
  Dual lands
  Any-color sources (Command Tower, etc.)
  Creatures (might need them for combat)
  Sources with costs/drawbacks last



TODO: Bad choice tree - combinatorial explosion of blocker/attackers
--------------------------------------------

It is silly to enumerate ALL possible (blocker,attackers) mappings. If
there are 10 attackers and 10 blockers this would quickly grow to many
possibilities.

First of all, why does `DeclareBlocker` allow multiple attackers? In MTG more than one blocker can block an attacker, but a single blocker cannot block MULTIPLE attackers.  Unless there is some special card with this ability. Is there? If so what is it called.

Look at how the Java TUI structures combat as a tree of choices. For each declared attacker, you can assign 0 blockers or you can add one additional blocker. After you add one, you can assign a second or be done (always option 0) and leave it at a single blocker. Then the process repeats for the second attacker.







TODO: Make a test deck with grizzly bears
----------------------------------------



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


