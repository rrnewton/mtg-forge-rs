---
title: Randomized stress tests with invariants for snapshot resume
status: open
priority: 2
issue_type: task
created_at: "2025-10-27T09:12:20Z"
updated_at: "2025-10-27T09:13:08Z"
---

# Description

Now you can see how to play some number of turns, and stop and resume randomly.
Build an e2e test script (under tests/) which stresses the system. This script can be in python and we can extend the e2e test script runner to run both .py and .sh files in that directory.

For a list of test decks (initially just grizzly bears and royal assassin):
 For both random/random and heuristic/heuristic modes:
 - Play a game with the deterministic seed and count the turns,
   choices, and log of choices made by P1/P2.
 - Play the same game stop-and-go, with players switched to fixed controllers
    - advance a random count of choices, 1-5, passing in fixed inputs
    - snapshot, resume, repeat until game end

 - Examine the collected logs of both the original and stop-and-go runs.
   - Filtering for relevant game actions (draw card, spell resolves, etc),
     the logs should match EXACTLY. The differences are only extra messages around stopping/resuming.
   - Make sure the final game 

If this works, you can make the test go even deeper by adding a `--save-final-gamestate=file` flag which will save the end-of-game state of play
to a snapshot file. When both run modes produce a final file, we can do a
deep comparison to make sure they match. Perhaps we can get the serialized text files to EXACTLY match, but there may be good reasons to ignore certain bits of state in the comparison instead.

