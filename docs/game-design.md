# Game Design

- Status: Draft
- Date: 2026-06-20

This document describes the intended game rules and content shape in a form
that implementation agents can use as a reference. It is not a full balance
specification. Values such as durations, yields, success rates, and efficiency
differences are intentionally left for later tuning.

## High-Level Concept

Idle Barquest is a Rust TUI idle RPG. The player selects a `Target`, selects an
`Action`, waits for a progress bar to complete, and then receives resources,
crafted items, discoveries, or other outcomes.

The game starts with a single hero exploring a forest. Over time, the player
develops a village and an adventurers' guild. New targets such as adventurers,
villagers, smiths, farms, and livestock facilities allow multiple tasks to run
in parallel.

## Core Loop

1. The player chooses a target.
2. The player chooses an available action for that target.
3. The action starts a timed progress task.
4. On completion, the game resolves rewards, discoveries, resource depletion,
   and any required state changes.
5. New resources, facilities, areas, or targets may unlock additional actions.

The loop should support both short, repeatable work and longer expeditions.
Simple gathering actions may be short and reliable, but their exact stability
and efficiency are balance decisions rather than fixed rules.

## Canonical Concepts

### Target

A `Target` is an actor or facility that can be assigned an action.

Targets are expected to have role-based strengths, but efficiency modifiers are
not yet final. The current design intent is:

| Target | Role |
| --- | --- |
| Hero | Central player character; can perform broad adventure actions. |
| Adventurer | Guild-managed worker for exploration, monster hunting, and dungeons. |
| Villager | Supports basic village labor such as gathering and logging. |
| Smith | Crafts weapons, armor, and tools from minerals and monster materials. |
| Adventurers' Guild | Manages adventurer recruitment, requests, and expedition support. |
| Farm | Village facility that produces food and plant materials over time. |
| Livestock | Village facility that produces food and animal materials over time. |

### Action

An `Action` is timed work assigned to a target. Each action may define:

- Required target types or facilities.
- Required resources or equipment.
- Duration.
- Completion rewards.
- Discovery chances.
- Depletion effects.
- Failure or partial-success rules, if needed later.

Initial action set:

| Action | Intent |
| --- | --- |
| Forest exploration | Gain wood, herbs, monster materials, and possible area discoveries. |
| Monster hunting | Gain monster materials, rewards, and possibly edible monster food. |
| Fishing | Gain fish as a food source from rivers or lakes. |
| Animal hunting | Gain meat, hides, bones, and related animal materials. |
| Dungeon expedition | Gain treasure, rare materials, and equipment from a long task. |
| Logging | Gain wood from forest resources. |
| Farming | Produce crops, food, and plant materials at farms. |
| Livestock work | Produce meat, hides, bones, and related materials from livestock. |
| Armor crafting | Consume wood, ore, and monster materials to create armor. |
| Enchantment | Add special effects to equipment using rare or magical materials. |

### Resource

Resources are grouped by gameplay purpose:

| Resource group | Examples | Purpose |
| --- | --- | --- |
| Natural resources | Wood, herbs | Early materials for village growth and crafting. |
| Minerals | Iron ore | Smithing, tools, equipment, and facility upgrades. |
| Food | Fish, meat, crops, edible monster parts | Sustains activity and may gate long actions. |
| Monster materials | Claws, hides, cores, edible parts | Crafting, enchantment, rewards, and survival. |
| Equipment | Weapons, armor, tools | Improves target performance and unlocks harder tasks. |

Food is the main survival resource. It may be consumed to maintain workers,
start long expeditions, or support repeated activity. The strictness of food
management should be tuned to preserve the idle game's pacing.

## Area and Depletion Rules

Areas are sources of actions and resources.

Forests and mines are finite field resources. Once a forest or mine is
depleted, it should not naturally recover. This creates pressure to discover new
areas or develop renewable village production.

Dungeons are renewable adventure resources. After a dungeon is cleared or
depleted, it should recover after a period of time and become available again.
Dungeons are the primary repeatable source for rare materials, treasure, and
combat rewards.

The design intent is to make resource routes change over time:

- Early game: gather from nearby finite resources.
- Mid game: discover new areas and unlock village production.
- Late game: combine renewable dungeons, farms, livestock, and unexplored areas
  to maintain supply.

## Survival Layer

The survival layer adds pressure without turning the game into a high-friction
survival simulation.

Food sources include:

- Fishing.
- Animal hunting.
- Edible monsters.
- Farms.
- Livestock.

Food should be important enough to influence planning, especially for long
tasks, but not so strict that idle progression stalls too easily.

## Progression Outline

### Early Game

The hero explores the forest, gathers wood and materials, and discovers the
first expansion paths. Food comes primarily from fishing and hunting. Wood is
used to establish basic village functions and unlock targets such as villagers
or the smith.

### Mid Game

The adventurers' guild expands the number of available workers. Adventurers can
handle exploration, monster hunting, and dungeon expeditions while villagers
support resource gathering. Forest exploration may reveal mines, dungeons, and
special gathering locations.

Village growth unlocks farms and livestock. These facilities provide renewable
food and some renewable materials, reducing dependence on finite forests and
mines.

### Late Game

Smithing and enchantment improve equipment, enabling harder monsters and deeper
dungeons. As forests and mines become depleted, the player maintains supply
through renewable dungeons, village production, and newly discovered areas.

Rare dungeon materials feed back into stronger equipment and facility upgrades,
increasing efficiency and opening harder content.

## TUI Implications

The TUI should make the following state easy to inspect:

- Available targets and their current task.
- Available actions for the selected target.
- Progress bars for running tasks.
- Remaining time or progress percentage.
- Resource inventory.
- Discoveries and completion results.
- Area depletion or recovery state.
- Food supply and food-related requirements.

When a task completes, the log should show the meaningful outcome: gained
resources, discovered areas, crafted equipment, depletion changes, or dungeon
recovery state.

## Balance Notes

The following are design directions, not fixed balance rules:

- Targets may have role-based strengths.
- The hero may be flexible but less efficient than specialized targets.
- Simple gathering may be short and reliable.
- Exploration and dungeon tasks may take longer but provide discoveries and rare
  rewards.
- Food should matter, especially for long actions, without overwhelming the idle
  loop.

## Open Questions

- How many concurrent tasks should be available at each stage?
- Should food be consumed per action, per target over time, or only for long
  expeditions?
- Should depleted forests and mines remain visible as exhausted areas, or be
  removed from available actions?
- Should dungeon recovery be real-time, tick-based, or tied to completed tasks?
- Which rewards should be deterministic, probabilistic, or both?
