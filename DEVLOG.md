
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



Use real cards and make card loading fully async
----------------------------------------------


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




TODO: async card loading
----------------------------------------




Comparison to transcript of Java forge-headless games
--------------------------------------------------------------------------------

```
sudo dnf install 
```


