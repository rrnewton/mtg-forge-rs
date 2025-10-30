# HOWTO PLAY MTG AND BUILD REPRODUCERS FOR AGENTS

It's important that you have the experience of playing the MTG game we're developing so you can see actual issues with game play and compare to expected behavior given MTG rules. Testing in code is insufficient.

## How to play the game
You're the agent and you can't directly play our MTG  game through interactive text user interface (TUI) on stdin. But you can effectively play it through:
* Fixed input controllers
* Stop/go snapshot and resume execution


```
cargo run --bin mtg -- tui DECK1.dck DECK2.dck
    --seed=100 \
    --stop-when-fixed-exhausted \
    --p1=fixed --p1-fixed-inputs="${P1_INPUTS}"
    --p2=fixed --p2-fixed-inputs="${P2_INPUTS}"
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

