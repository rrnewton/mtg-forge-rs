


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

Workflow: Tasks and Commits
========================================

Commit to git as described in the PROJECT_VISION.

We track work in Beads instead of Markdown. Run `bd quickstart` to see how.

There is an initial issue (mtg-1) which will serve as our primary tracking issue for what we're currently working on. Every time we do a git commit let's make sure the tracking issue tracks our medium-term TODO list, so that if we lose our current conversation history we can start again from there.

You should periodically do documentation work, usually before committing, to make sure the issues are up-to-date.
