---
title: Randomized stress tests with invariants for snapshot resume
status: open
priority: 0
issue_type: task
created_at: "2025-10-27T09:12:20Z"
updated_at: "2025-10-28T03:33:26-07:00"
closed_at: "2025-10-28T00:55:30Z"
---

# Description

We have a partial implementation of the `--stop-every`/`--stop-from` suspend resume mechanism.
You can use this to test the engine yourself and write e2e tests.

But to make sure it is rock solid we need a STRICT DETERMINISM GUARANTEE. We are
achieving this by rigorous stress testing. Continue to improve
`./tests/snapshot_stress_test.py` until it's fully deterministic as described
below.

Design
==============================================================================

Don't update this first section, but update the Tracking section below to track
progress on this issue.

## Basic stress test design

For a growable list of test decks:
 For both random/random and heuristic/heuristic modes:
 - Play a game with the deterministic seed and count the turns,
   choices, and log of choices made by P1/P2.
 - Play the same game stop-and-go, with players switched to fixed controllers
    - advance a random count of choices, 1-5, passing in fixed inputs
    - snapshot, resume, repeat until game end

 - Examine the collected logs of both the original and stop-and-go runs.
   - Filtering for relevant game actions (draw card, spell resolves, etc),
     the logs should match EXACTLY. The differences are only extra messages around stopping/resuming.
   - Make sure the final outcome matches.

If this works, you can make the test go even deeper by adding a
`--save-final-gamestate=file` flag which will save the end-of-game state of play
to a snapshot file. When both run modes produce a final file, we can do a deep
comparison to make sure they match. Perhaps we can get the serialized text files
to EXACTLY match, but there may be good reasons to ignore certain bits of state
in the comparison instead.

You can choose whatever mechanism you like to collect the choices from the first
(normal) game run. You can either standardize the choice output in the logs
enough that it can be extracted from the logs OR you can have a flag that
activates logging of just the [p1/p2] choices.

## Principle of independent objects for gamestate / controllers

For this stop/go setup to work, and the game to remain deterministic, we need
SEPARATION between the game state and the controllers. These should be viewed as
separate, interacting systems. One key place where this appears is in the
handling of RNG seeding. We will add flags to control seeding of systems separately:

```
--seed         # master seed to which the others default
--seed-shuffle # affects only the initial shuffle
--seed-engine  # affects game engine evolution
--seed-p1      # affects P1 controller only
--seed-p2
```

Irrespective of whether we control them from the CLI, the important thing is the 
non-interference of these different RNGs during play.

The master seed can be used by COPYING it to each of the per-system seeds (not
by using it to generate a random number, which mutates it). A constant inside
each system can be used to add salt to the random seed, so that, e.g. P1 P2
don't see the identical stream of random numbers.

When a stop/resume occurs, the state of the engine is serialized and resumed.
But it is independent from the RNG of the controllers. When we resume we 
may carry on with the controller from the snapshot, or change it to a new controller.
These are two different scenarios that both need to be tested.
If the controller is reinitialized, then the CLI args determine its state, but it
remains completely unentangled from the game engine's state.


## CRITICAL: Criteria for closing this task

Only close this task when we at least three decks can fully pass the test with
exact matching game game actions between the normal and stop-and-go run.
- royal_assassin.dck
- white_aggro_4ed.dck
- moonred.dck

This INCLUDES the deep comparison of final gamestate. Until we have total fidelity between original runs (random and heuristic) and replays, we are not done with this task.


Tracking - Implementation Progress
==============================================================================

### Phase 9: Implemented Controller State Serialization (2025-10-27 commit ec4540c)

**MAJOR ARCHITECTURAL FIX - Controller State Preservation:**

The core issue identified: FixedScriptController state (current_index) was not preserved across snapshot/resume, causing the controller to restart from index 0 in each segment.

**Solutions Implemented:**

1. ✅ **Added get_snapshot_state() to PlayerController trait**
   - New optional method returns serializable controller state
   - Default implementation returns None (no state to preserve)
   - FixedScriptController serializes entire state (script + current_index)

2. ✅ **Made FixedScriptController fully serializable**  
   - Added serde derives for serialization
   - Made current_index public
   - Controller state saved in GameSnapshot struct

3. ✅ **Fixed main.rs to capture ACTUAL controller state**
   - Previous approach: cloned controller at creation (always index 0)
   - New approach: call get_snapshot_state() AFTER game runs
   - Captures real current_index after choices consumed

4. ✅ **Fixed ChoicePoint synchronization bug**
   - GameLoop logs ChoicePoint for EVERY controller method call
   - Fixed controller was only consuming script for SOME calls
   - Now ALL controller methods call next_choice() for synchronization:
     - choose_targets (even single target)
     - choose_mana_sources_to_pay  
     - choose_damage_assignment_order
     - choose_cards_to_discard

**Test Results:**
- ✅ All 244 unit tests PASSING
- ⚠️ Stress tests still failing (NEW ROOT CAUSE identified)

**NEW ISSUE IDENTIFIED - Test Methodology Flaw:**

The stress test has a fundamental architectural problem:

1. **What test does:**
   - Extracts choices by parsing log lines (">>> RANDOM:")
   - Only some controller calls generate log lines
   - Creates Fixed controller script from these extracted choices

2. **What actually happens:**
   - GameLoop logs ChoicePoint for EVERY controller method call
   - Stop condition counts ALL ChoicePoints (not just logged ones)
   - Fixed controller now consumes script entry for EVERY call

3. **The mismatch:**
   - Test extracts M logged choices (e.g., 127 choices)
   - Actual game makes N ChoicePoints (e.g., 150+ total)
   - Fixed controller runs out of script entries early
   - Defaults to passing priority, causing gameplay divergence

**Example:**
- Random controller chooses single target → no log line, but ChoicePoint logged
- Test doesn't extract this choice
- Fixed controller needs entry for this but script is missing it
- Fixed controller defaults to index 0, wrong decision made

**Solution Required:**

Test must be refactored to extract choices from GameAction::ChoicePoint log, not from parsed log lines. Options:

A. Add --dump-choices flag that outputs all ChoicePoints to separate file
B. Parse ChoicePoint data from snapshot's intra_turn_choices
C. Redesign test to use ReplayController instead of Fixed
D. Have GameLoop export ChoicePoint log as choices file

**Overall Progress:**
- ✅ Snapshot/resume architecture complete
- ✅ Controller state serialization working  
- ✅ Turn order determinism achieved
- ✅ ChoicePoint synchronization fixed
- ⚠️ Test methodology needs refactor to match engine architecture
- ⚠️ Once test fixed, expect full determinism

Blocks (2):
  ← mtg-103: Make snapshot stress test run NUM_REPLAYS=3 per deck [P3]
  ← mtg-104: Split snapshot stress test into per-deck script and parallel runner [P3]

