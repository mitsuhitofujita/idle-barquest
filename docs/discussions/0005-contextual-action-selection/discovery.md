---
date: 2026-08-29
tags: [tui, interaction, action-selection]
status: proposed
worklogs: []
decisions: []
---

選択方法を見直して。
選択したTargetによってActionは変更されるので、選択肢は選択によってインタラクティブに生まれるようにして。

ターゲット選択時↓
```
|> Target:
| a) Hero
| b) Adventurer
| c) Farmer
```

`2`を選択後↓
```
|  Target:       |> Action:
|    Hero        | a) Forest Exploration
|    Adventurer <|
|    Farmer      |
```
