# HOWTO PLAY MTG AND BUILD REPRODUCERS FOR AGENTS

It's important that you have the experience of playing the MTG game we're developing so you can see actual issues with game play and compare to expected behavior given MTG rules. Testing in code is insufficient.

## How to play the game
You're the agent and you can't directly play our MTG game through interactive text user interface (TUI) on stdin. But you can effectively play it through:
* Fixed input controllers
* Stop/go snapshot and resume execution

When using `--stop-when-fixed-exhausted` or `--stop-on-choice`, the game will automatically print choice prompts (attackers, blockers, discard, spell/ability selection) before saving the snapshot and exiting. This makes it easy to see exactly what choice was about to be made, helping you decide what input to provide when resuming.

```
cargo run --bin mtg -- tui DECK1.dck DECK2.dck \
    --seed=100 \
    --stop-when-fixed-exhausted \
    --p1=fixed --p1-fixed-inputs="${P1_INPUTS}" \
    --p2=fixed --p2-fixed-inputs="${P2_INPUTS}"
```



## How to build a reproducer
When I report a buggy behavior, or when you see it in the middle of a larger game or stress test, it helps to build a minimal reproducer. You can do this incrementally:

1. Start with empty or minimal fixed inputs
2. Run with `--stop-when-fixed-exhausted`
3. The game will print the available choices before stopping
4. Add the next choice to your fixed inputs and repeat

Example workflow:
```bash
# First run - see what choices are available at the start
cargo run --bin mtg -- tui DECK1.dck DECK2.dck \
    --seed=100 \
    --stop-when-fixed-exhausted \
    --p1=fixed --p1-fixed-inputs="" \
    --p2=random \
    --verbosity=verbose

# You'll see a prompt like:
#   Alice available actions:
#     [0] Play land: Forest
#     [1] Cast spell: Grizzly Bears
#
# Add choice "0" to play the Forest, then repeat...

cargo run --bin mtg -- tui DECK1.dck DECK2.dck \
    --seed=100 \
    --stop-when-fixed-exhausted \
    --p1=fixed --p1-fixed-inputs="0" \
    --p2=random \
    --verbosity=verbose
```

Note: Fixed inputs are separated by semicolons (`;`), not commas or spaces.
Example: `--p1-fixed-inputs="0;1;2;0"`

