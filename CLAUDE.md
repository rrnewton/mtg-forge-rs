
A performant Magic The Gathering engine for AI research
=======================================================

Read the PROJECT_VISION.md document.

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

SAFETY! This is a safe-rust project. We will not introduce the `unsafe` keyword unless we have a VERY good reason and with significant advanced planning.

Documentation and Analysis
========================================

When creating analysis documents, specifications, or other AI-generated documentation, place them in the `ai_docs/` directory. This keeps the top-level clean and makes it clear which documents are AI-generated analysis (and may become outdated) versus core project documentation.

Workflow: Task tracking
========================================

We use "beads" to track our issues locally under version control. Review `bd quickstart` to learn how to use it. 

Every time we do a git commit, update our beads issues to reflect:
- What was just completed (check off items in lists, close completed task(s))
- What's next (update the in tracking issues that track the granular issues)
- Mention in the commit any new issues created to document bugs found or future work.

The beads database is our primary tracking mechanism, so if we lose conversation history we can start again from there.  You should periodically do documentation work, usually before committing, to make sure information in the issues is up-to-date.

### Beads CONVENTIONS for this project

Do NOT read or modify files inside the `./.beads/` private database, except when fixing merge conflicts in markdown files that you can read.

Prefer the MCP client to the CLI tool. ALWAYS `bd update` existing issues, never introduce duplicates with spurious `bd create`.

The issue prefix may be customized (`foobar-1`, `foobar-2`), but here we will refer `bd-1` as example issue names

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

We often record transient information, like benchmark results, that quickly gets out of date. We want to label such information so we can tell how old it is. In addition to YYYY-MM-DD, our convention is to use:
  `git rev-list --count HEAD`
which prints out the number of commits in the repo (or equivalently the ./gitdepth.sh script), and then format the timestamp as `YYYY-MM-DD_#DEPTH(387498cecf)` e.g. `2025-10-22_#161(387498cecf)`. That's our full timestamp
for any transient information that derives from a specific commit.
Sometimes this requires us to split our commits into (1) functionality and then (2) documentation-update.

#### Reference issues in code TODO

We don't want TODO items to be in floating code alone. For anything but the most trivial TODOs, we adopt the convention of referencing issues that tracks the TODO:

```
// TODO(mtg-13): brief summary here
```

Then, the commit that fixes the issue both removes the comment and closes the issue in beads.

#### Use description field only, not notes

When creating or updating issues with `bd`, always put ALL content in the description field. Do NOT use the --notes field, as it creates duplication and confusion between what's in description vs notes. Keep all issue information consolidated in the description field only.


Workflow: Commits and Version Control
================================================================================

Commit to git as described in the PROJECT_VISION.

Clean Start: Before beginning work on a task
--------------------------------------------

Make sure we start in a clean state. Check that we have no uncommitted changes in our working copy. Perform `git pull origin <BRANCH>` to make sure we are starting with the latest version on our branch. Check that `make validate` passes in our starting state.

If github MCP is configured and github actions workflows exist for this project, check the github actions CI status for the most recent commit and make sure it not red (if it's still pending, ignore and proceed). If
there's a CI failure, then fixing THAT becomes our task. Finally, check that `make validate` passes locally in our starting state.

Pre-Commit: checks before committing to git
--------------------------------------------

Run `make validate` and ensure that it passes or fix any problems before committing.

Also include a `Test Results Summary` section in every commit message that summarizes how many tests passed of what kind.

If you validate some changes with a new manual or temporary test, that test should be added to either the unit tests, examples, or e2e tests and it should be called consistently from both `make validate` and Github CI.

NEVER add binary files or large serialized artifact to version control without explicit permission. Always carefully review what you are adding with `git add`, and update `.gitignore` as needed.

Branches and pushing
----------------------------------------

You may push after validation and can check CI status with github MCP. Don't force push unless you're asked to or ask permission.

We merge feature branches into main when they're completed and validating. ARCHIVE completed feature braches. Upon merging a feature branch X, archive it as tag `X.v1` or `X.(N+1)` if that tag is taken.

Commit message documents relationship to original Java version
--------------------------------------------------------------

Finally, also before committing reanalyze the relationship between (1) what you built and (2) the existing Java implementation, and summarize it. It's ok for the Rust and Java versions to deviate, but there should be a reason for it and we should document it in these commit messages.

```
## Relationship to Java Forge

- this Rust reimplementation does X
- the upstream Java version does Y
```

Commit message justifies game play logic with real games
--------------------------------------------------------

Except for purely internal fixes that don't directly affect MTG gameplay, in every commit you will need to justify changes with real gameplay logs. Add a section to the commit message which provides evidence for the correct behavior of the fix in the form of a log snippet from a real `mtg tui` CLI game, ideally with a runnable reproducer CLI command.

- We will reason about the behavior of the game in terms of the log messages of game actions.
- Compare against the rules of MTG (and cite the rule numbers where applicable). Keep an eye out for for missing behaviors, contradictory information, or impossible events.
- In the case of AI, consider whether the player actions make a basic level of sens.

See the file `ai_docs/HOWTO_AGENTPLAY+REPRODUCERS.md` for instructions on playing the game as an agent to observe engine behaviors without writing new code.


