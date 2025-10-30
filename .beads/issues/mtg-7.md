---
title: CardDatabase.get_card() should return references instead of cloning
status: open
priority: 3
issue_type: feature
created_at: "2025-10-30T05:28:25Z"
updated_at: "2025-10-30T05:28:25Z"
---

# Description

Currently clones CardDefinition on every access (database_async.rs:52).
Heaptrack shows this as top allocation site.

Requires adding lifetime parameters to return &CardDefinition.
Would eliminate ~50% of Card struct clones.
