---
title: Fix prompts not appearing before stop conditions
status: closed
priority: 2
issue_type: bug
created_at: "2025-10-30T05:28:59Z"
updated_at: "2025-10-30T05:29:05Z"
closed_at: "2025-10-30T05:29:05Z"
---

# Description

When using --stop-when-fixed-exhausted or --stop-on-choice flags, the game would save snapshots and exit without showing what choice was about to be made. This was confusing for debugging.

Fixed by:
1. Adding formatting functions to controller.rs for all choice types
2. Modifying game_loop.rs to print prompts BEFORE check_stop_conditions()
3. Removing duplicate prompts from InteractiveController
4. Using block scoping for borrow checker constraints

Completed in commit c57956f.
