


An experiment in persistence
========================================

Read the PROJECT_VISION.md document.

This project is an experiment in largely-unattended longer time-horizon development with Claude. That means you should be continuing to work toward the project vision, implementing new features iteratively, and not stopping for approval. 

If you become stuck with an issue you cannot debug, you can file an issue for it and leave it to work on other topics. Of course, the tests should be always passing before each commit and achieve reasonably good code coverage as described below.

References
========================================

Refer to the MTG (Magic the Gathering) rules in the `./rules` directory.

 - 01_full_official_MagicCompRules_20250919
 - 02_mtg_rules_condensed_medium_length_gemini.md

You should mainly use the second, condensed summary for understanding the basic operation of the MTG game. When necessary, refer to the long rules list in the official MTG rules (the first document above).

Coding conventions
========================================

PREFER STRONG TYPES. Do not use "u32" or "String" where you can have a more specific type or at least a type alias. "String" makes it very unclear which values are legal. We want explicit Enums to lock down the possibilities for our state, and we want separate types for numerical IDs and distinct, non-overlapping uses of basic integers.

Delete trailing spaces. Don't leave empty lines that consist only of whitespace. (Double newline is fine.)

Add README.md files for every major subdirectory/subsystem.  For example `src/core`, `src/game`, etc.

Read the PROJECT_VISION description of coding conventions we should follow for high-performance Rust (unboxing, minimizing allocation, etc). In particular, adhere to the below programming patterns / avoid anti-patterns, which generally fall under the principle of "zero copy":

- Avoid clone: instead take a temporary reference to the object and manage lifetimes appropriately.
- Avoid collect: instead take an iterator with references to the original collection without copying.

Read OPTIMIZATION.md for more details.

Workflow: Tasks and Commits
========================================

Commit to git as described in the PROJECT_VISION.

Task Tracking
----------------------------------------

We use "beads" to track our issues locally. Review `bd quickstart` to learn how to use it.

Every time we do a git commit, update our beads issues to reflect:
- What was just completed (check off items in lists, close completed task(s))
- What's next (update the in tracking issues that track the granular issues)
- Any individual new issues discovered become new tasks with references from trackig issues

The beads database is our primary tracking mechanism, so if we lose conversation history we can start again from there.  You should periodically do documentation work, usually before committing, to make sure information in the issues is up-to-date.

### Beads CONVENTIONS for this project

#### Tracking issues and Priorities

Warning: Be careful to EDIT tracking issues (`bd update`) and not just
file a new duplicate issue with `bd create`.

- Issues labeled "human" are created by me and will always have 0 priority.
- Issue mtg-1, at priority 0, is the OVERALL tracking issue. It primarily references other tracking issues
  and reiterate some of these conventions. We want to keep it pretty short.

- The next tracking issues, e.g. mtg-2 and on have priority 1 and are topic-specific trackers:
  - Optimization tracking
  - MTG feature completeness: supporting keywords/abilities/complex mana and effects.
  - Gameplay feautures: like an actual TUI to play as a human.
  - Cross-cutting codebase issues: APIs (player, controller, etc), testing coverage and methodology.

 - All tracking issues refer to granular issues by name in their text, e.g. "mtg-42"
 - All other granular issues will have priority 3 to 4 unless they are seen as a critical bug, which will bump them to priority 2.

#### Mark transient information

We often record transient information, like benchmark results, that quickly gets out of date. We want to label such information so we can tell how old it is. Rather than a realtime timestamp, our convention is to use `./scripts/gitdepth.sh` which prints out the number of commits in the repo, and then the shorthand `commit#161(387498cecf)` becomes our timestamp for information that came from a particular commit. Sometimes this requires us to split our commits into (1) functionality and then (2) documentation-update.

#### Reference issues in code TODO

We don't want TODO items to be in floating code alone. For anything but the most trivial TODOs, we adopt the convention of referencing issues that tracks the TODO:

```
// TODO(mtg-13): brief summary here
```

Then, the commit that fixes the issue both removes the comment and closes the issue in beads.

Clean Start: Before beginning work on a task
--------------------------------------------

Make sure we start in a clean state. Check that we have no uncommitted changes in our working copy. Perform `git pull origin main` to make sure we are starting with the latest version. Check that `make validate` passes in our starting state. 

Pre-Commit: checks before committing to git
--------------------------------------------

Run `make validate` and ensure that it passes or fix any problems before committing.

Also include a `Test Results Summary` section in every commit message that summarizes how many tests passed of what kind.

If you validate some changes with a new manual or temporary test, that test should be added to either the unit tests, examples, or e2e tests and it should be called consistently from both `make validate` and Github CI.

Commit message documents relationship to original Java version
--------------------------------------------------------------

Finally, also before committing reanalyze the relationship between (1) what you built and (2) the existing Java implementation, and summarize it. It's ok for the Rust and Java versions to deviate, but there should be a reason for it and we should document it in these commit messages.

```
## Relationship to Java Forge

- this Rust reimplementation does X
- the upstream Java version does Y
```
