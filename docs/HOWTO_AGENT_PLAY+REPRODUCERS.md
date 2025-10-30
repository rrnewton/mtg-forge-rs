# HOWTO PLAY MTG AND BUILD REPRODUCERS FOR AGENTS

It's important that you have the experience of playing the MTG game we're developing so you can see actual issues with game play and compare to expected behavior given MTG rules. Testing in code is insufficient.

## How to play the game
You're the agent and you can't directly play our MTG  game through interactive text user interface (TUI) on stdin. But you can effectively play it through:
* Fixed input controllers
* Stop/go snapshot and resume execution

Your most basic tool is to run random games in a loo until their logs demonstrate the thing you want:

```
cargo build --release
./target/release/mtg tui DECK1.dck DECK2.dck --seed=100 --p1=random --p2=fixed
```

But a more targetted approach is for you to play the game yourself turn by turn.
To do that play with fixed inputs and have the game stop when they are exhausted.

```
./target/release/mtg tui decks/grizzly_bears.dck decks/royal_assassin.dck \
    --p1-name=P1 --p2-name=P2 \
    --log-tail 100 --stop-when-fixed-exhausted \
    --p1=fixed --p1-fixed-inputs="" \
    --p2=fixed --p2-fixed-inputs=""
...
  ==> Priority P1: life 20, Main1

Available actions:
  [0] Pass
  [1] Play Swamp
```

With this command you will see the last 100 lines of the game with the choice you need to make next. You can then add "1" to your `--p1-fixed-inputs` and advance to the next move.
You can pass numbers separated by semi-colons, `--p1-fixed-inputs=1;1;1` OR if you know what action you want to do you can use the command syntax `--p1-fixed-inputs=Play Swamp;Play Swamp`, which is robust to option ordering.

## How to build a reproducer
When I report a buggy behavior, or when you see it in the middle of a larger game or stress test, it helps to be able to create a more minimal reproducer. Using the above method for playnig the game, you can:

- add `--draw-card="Card 1;Card 2;...` to seed the hand with the cards you need from the deck to reproduce the examples,
- control both P1/P2 to play lands and then play the cards needed for the reproduce,
- controll P1/P2 choices so at the right time they activate / attack / block respond to create the desired situation.

For more complex situations, it can be helpful to instead create a `.pzl` file and pass it in with `--start-state`. That way you can start at a specific phase with battlefields and hands already configured in the desired way.


