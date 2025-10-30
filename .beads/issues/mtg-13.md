---
title: Arena allocation for per-turn temporaries
status: open
priority: 4
issue_type: feature
created_at: "2025-10-30T05:28:25Z"
updated_at: "2025-10-30T05:28:25Z"
---

# Description

Use arena allocators (bumpalo or typed-arena) for per-turn allocations.
Benefits: faster allocation (pointer increment), bulk deallocation, better cache locality.
