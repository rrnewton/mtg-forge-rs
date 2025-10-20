
Dependencies

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


TODO: Fix github CI
----------------------------------------

The CI is failing on clippy and formatting. Make sure `make validate` runs these and that therefore they get allocated before we make any commit.

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`

--------------------

After the message `Loaded 31438 cards`, print the amount of time it took in milliseconds.


TODO: async card loading
----------------------------------------




Comparison to transcript of Java forge-headless games
--------------------------------------------------------------------------------

```
sudo dnf install 
```
