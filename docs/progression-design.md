# Progression Design

- Status: Draft
- Date: 2026-06-22

This document restructures the early progression notes into an implementation
reference for agents. It describes only the currently stated early-game flow.
It is not a complete progression, balance, or content specification.

## Naming Rules

In-game names should use ASCII-friendly English names. The source notes may use
Japanese design labels, but implementation content should use the English names
listed here or close equivalents.

Quality tiers, item quality, and rare gathering results are expected extension
points. They should not be required for the first implementation of this early
progression.

Example future extension:

| Base item | Possible rare item | Notes |
| --- | --- | --- |
| `Stone` | `Sharp Stone` | Reserved for later tuning or crafting depth. |

## Opening State

The player starts as a single hero who wakes on a shore.

There should be no explicit in-game explanation for the opening situation. The
available areas, actions, and rewards should communicate the starting state.

Initial actor:

| Actor | Role |
| --- | --- |
| `Hero` | The only available actor at the start of the game. |

Initial explorable areas:

| Area id | Display name | Purpose |
| --- | --- | --- |
| `first_shore` | `First Shore` | Shore gathering source for food-adjacent materials and stone. |
| `nearby_woods` | `Nearby Woods` | Wooded gathering source for berries, mushrooms, twigs, and monster material. |
| `nearby_hill` | `Nearby Hill` | Hill gathering source for herbs and monster material. |

## Stage 1: Hero Gathering

At the start of Stage 1, the `Hero` can explore `First Shore`,
`Nearby Woods`, and `Nearby Hill`.

### Gathering Rewards

| Area | Resource id | Display name | Source label | Early purpose |
| --- | --- | --- | --- | --- |
| `First Shore` | `small_fish` | `Small Fish` | Small fish | Deferred. Food-adjacent early resource. |
| `First Shore` | `seaweed` | `Seaweed` | Seaweed | Can become `Food` after `Campfire` unlock. |
| `First Shore` | `stone` | `Stone` | Stone | Unlocks `Stone Workbench`. |
| `Nearby Woods` | `berry` | `Berry` | Tree nut / berry | Can become `Food` after `Campfire` unlock. |
| `Nearby Woods` | `mushroom` | `Mushroom` | Mushroom | Deferred early resource. |
| `Nearby Woods` | `twig` | `Twig` | Small branch | Crafts `Bark` and unlocks early technologies. |
| `Nearby Hill` | `herb` | `Herb` | Medicinal herb | Deferred early resource. |

### Monster Rewards

Monsters currently do not affect exploration success, exploration duration, or
area access. For now, they are treated as post-exploration resource sources.

| Area | Monster id | Display name | Reward |
| --- | --- | --- | --- |
| `Nearby Woods` | `biter` | `Biter` | `Small Fang` |
| `Nearby Hill` | `crawler` | `Crawler` | `Small Fang` |

The exact combat, defeat, and success rules are intentionally not specified at
this stage. Implementations may model these monsters as simple completion
rewards until combat rules exist.

## Stage 1.1: First Technologies And Crafting

Early technologies are unlocked from gathered resources. Each technology adds a
small new capability and should make the next progression step easier to reach.

### Technology Unlocks

| Technology id | Display name | Unlock cost | Unlock effect |
| --- | --- | --- | --- |
| `stone_workbench` | `Stone Workbench` | `Stone` x40 | Enables `Bark` crafting from `Twig`. |
| `campfire` | `Campfire` | `Twig` x10, `Bark` x10 | Enables basic `Food` crafting. |
| `simple_bed` | `Simple Bed` | `Twig` x20, `Bark` x20 | Adds the `Rest` action. |

### Crafting Recipes

| Recipe id | Output | Input | Required technology |
| --- | --- | --- | --- |
| `craft_bark_from_twig` | `Bark` x5 | `Twig` x20 | `Stone Workbench` |
| `craft_food_from_berry` | `Food` x1 | `Berry` x20 | `Campfire` |
| `craft_food_from_seaweed` | `Food` x1 | `Seaweed` x20 | `Campfire` |

### Progression Notes

`Stone Workbench` is the first crafting foundation. It converts gathered
`Twig` into `Bark`, which then supports the `Campfire` and `Simple Bed`
unlocks.

`Campfire` starts basic food production. Both `Berry` and `Seaweed` can be
converted into `Food`, so early exploration of both `Nearby Woods` and
`First Shore` contributes to survival progression.

`Simple Bed` adds the `Rest` action. The effect, duration, cost, and conditions
for `Rest` are intentionally deferred.

## Deferred Decisions

The following gaps are known and accepted. They should remain undecided until
the surrounding systems need them.

| Topic | Deferred question |
| --- | --- |
| `Mushroom` | What is its early-game use? |
| `Herb` | What is its early-game use? |
| `Small Fish` | Does it become `Food`, remain raw food, or serve another role? |
| `Small Fang` | What crafting, upgrade, or trade role should it have? |
| `Rest` | What are its effects, duration, resource costs, and requirements? |
| Monsters | What conditions determine defeat, success, or reward generation? |

## Implementation Guidance

Use the tables in this document as content seeds, not final balance data.

Recommended first implementation scope:

1. Add the three initial areas.
2. Add the listed resources and monster rewards.
3. Add the three early technologies.
4. Add the three crafting recipes.
5. Add `Rest` as an unlocked action only after `Simple Bed`, with behavior left
   minimal until the rest system is designed.

Avoid blocking Stage 1 or Stage 1.1 progression on item quality, rare resource
drops, or full combat behavior. Those systems are planned extension points.
