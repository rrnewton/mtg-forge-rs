


An experiment in persistence
========================================

Read the PROJECT_VISION.md document.

This project is an experiment in largely-unattended longer time-horizon development with Claude. That means you should be continuing to work toward the project vision, implementing new features iteratively, and not stopping for approval. 

If you become stuck with an issue you cannot debug, you can file an issue for it and leave it to work on other topics. Of course, the tests should be always passing before each commit and achieve reasonably good code coverage.

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

Read the PROJECT_VISION description of coding conventions we should follow for high-performance Rust (unboxing, minimizing allocation, etc).

Workflow: Tasks and Commits
========================================

Commit to git as described in the PROJECT_VISION.

We track work in TODO.md at the repository root. This file contains:
- Current status and latest commit
- Completed features organized by phase
- Next priorities with checkboxes
- Known issues
- Progress summary

Every time we do a git commit, update TODO.md to reflect:
- What was just completed (check off items, move to completed section)
- What's next (update priorities)
- Any new issues discovered

The TODO.md serves as our primary tracking document, so if we lose conversation history we can start again from there.

You should periodically do documentation work, usually before committing, to make sure TODO.md is up-to-date.

Make sure you ACTUALLY run `cargo test` before any commit. Include a Test Results Summary section in every commit message like this:

```
## Test Results Summary

    $ cargo test 2>&1  | grep -E '(test result: |Running |Doc-tests )'
         Running unittests src/lib.rs (target/debug/deps/mtg_forge_rs-dbfccebd340e260f)
    test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
         Running unittests src/main.rs (target/debug/deps/mtg_forge_rs-d765597e7833fdac)
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
       Doc-tests mtg_forge_rs
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


Finally, also before committing reanalyze the relationship between (1) what you built and (2) the existing Java implementation, and summarize it. It's ok for the Rust and Java versions to deviate, but there should be a reason for it and we should document it in these commit messages.

```
## Relationship to Java Forge

- this Rust reimplementation does X
- the upstream Java version does Y
```
