---
title: SVar resolution (DB$ sub-abilities)
status: open
priority: 3
issue_type: feature
created_at: "2025-10-30T05:28:25Z"
updated_at: "2025-10-30T05:28:25Z"
---

# Description

Implement SVar (Script Variable) resolution for DB$ sub-abilities.
SVars allow card scripts to define reusable sub-abilities and parameters.

Example from Lightning Bolt:
SVar:DBDamage:DB$ DealDamage | NumDmg$ 3 | ValidTgts$ Creature,Player

Requires parser for SVar definitions and resolution mechanism.
