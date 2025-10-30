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
To do that play with fixed inputs and have the game stop when they are exhausted:

```
./target/release/mtg tui DECK1.dck DECK2.dck \
    --log-tail 50 --stop-when-fixed-exhausted \
    --p1=fixed --p1-fixed-inputs="" \
    --p2=fixed --p2-fixed-inputs=""
```



## How to build a reproducer
When I report a buggy behavior, or when you see it in the middle of a larger game or stress test, it helps to be able to 
```
cargo run --bin mtg -- tui DECK1.dck DECK2.dck
    --seed=100 \
    --stop-when-fixed-exhausted \
    --p1=fixed --p1-fixed-inputs="${P1_INPUTS}"
    --p2=fixed --p2-fixed-inputs="${P2_INPUTS}"
```

