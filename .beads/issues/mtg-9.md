---
title: String allocation optimization
status: open
priority: 4
issue_type: chore
created_at: "2025-10-30T05:28:25Z"
updated_at: "2025-10-30T05:28:25Z"
---

# Description

Card names, player names cloned frequently.
Consider using Arc<str> or &'static str where appropriate.
