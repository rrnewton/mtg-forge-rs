---
title: String allocation optimization
status: open
priority: 4
issue_type: chore
created_at: "2025-10-26T21:06:34Z"
updated_at: "2025-10-26T21:06:34Z"
---

# Description

Card names, player names cloned frequently.
Consider using Arc<str> or &'static str where appropriate.
