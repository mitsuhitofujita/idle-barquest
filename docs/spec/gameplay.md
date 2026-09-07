# Gameplay and world state

## Content and live state

A `Catalog` owns immutable templates for targets, settlements, locations,
actions, resources, crafting recipes, and Location/Action reward tables. Each
kind is addressed by a stable string id. Templates are stored in registration
order, which is also the display order used by menus and inventory.

`GameState` owns the current settlement, target instances, unlocked locations
and actions, resource stacks, and completed settlement facilities. Target
instances refer to their template by id and each can hold at most one task. New
known target instances can be spawned with a unique id; known locations and
actions can be unlocked idempotently.

The built-in world starts at Awakening Shore with one Hero. It unlocks First
Shore, Nearby Woods, Nearby Hill, and Base, plus Gather, Fish, Hunt, and Craft.
Gather, Fish, and Hunt take ten seconds. The catalog contains Pebble, Twig,
Grass, Vine, Small Fish, Seaweed Fragment, Small Fang, Awful Meat, Tiny Magic
Stone, and Primitive Fishing Rod.

## Assignment and progress

A task combines a target instance, a location, an action, optional recipe, and
tick-based progress. Ordinary assignment succeeds only when all referenced
content exists, the location and action are unlocked, both target and location
support the action, the target is idle, and the Location/Action pair has a valid
non-empty reward table whose entries reference known resources.

Available actions are the unlocked actions supported by both the selected target
and location, in unlock order. Base supports Craft, whose recipes add a fourth
selection stage. A recipe can start only when its content references are valid,
its prerequisite facilities are complete, and every ingredient is available.
Its ingredients are consumed immediately when work starts. Unique facility
recipes are unavailable while the facility is under construction and disappear
after completion; item recipes remain repeatable.

Progress advances toward a clamped nonzero goal and saturates when complete.
Multiple target instances can therefore run one task each during the same state
advance.

## Completion and rewards

Each reward-table entry independently awards a positive resource amount at its
integer percentage chance. Guaranteed entries do not consume randomness.
Successful duplicate resource entries are combined with saturating addition
while preserving the first successful entry's order.

When an ordinary task completes, every awarded resource is added to inventory
with saturating addition. The task is removed and core emits a completion event
containing the target, location, action, and ordered aggregated reward list. An
empty list represents no reward.

When crafting completes, a facility recipe adds its recipe id to the current
settlement's permanent facilities, while an item recipe adds its output to an
inventory stack. Core emits a separate craft-completion event. Stone Table is
currently the only facility with behavior: it unlocks Primitive Fishing Rod.
Crude Bed, Crude Furnace, and Primitive Fishing Rod otherwise have no gameplay
effect yet.

Inventory storage follows first-acquisition order, while the acquired-resource
projection used by the UI follows catalog order. Materials and stackable items
share this projection. The presence of a stack marks a resource as acquired even
when its amount is zero.

## Built-in compatibility and rewards

The Hero supports all four actions. First Shore supports Gather and Fish;
Nearby Woods and Nearby Hill support Gather and Hunt; Base supports Craft. Each
gathering combination has a reward table:

| Location | Action | Independent rewards |
| --- | --- | --- |
| First Shore | Gather | Seaweed Fragment 60%; Pebble 100% |
| First Shore | Fish | Small Fish 30%; Seaweed Fragment 100% |
| Nearby Woods | Gather | Vine 60%; Twig 100% |
| Nearby Woods | Hunt | Awful Meat 100%; Tiny Magic Stone 10% |
| Nearby Hill | Gather | Grass 100%; Pebble 50% |
| Nearby Hill | Hunt | Awful Meat 100%; Small Fang 60% |

Every successful entry currently awards one unit.

All built-in recipes take twenty seconds:

| Recipe | Output | Ingredients | Requirement |
| --- | --- | --- | --- |
| Stone Table | unique facility | Pebble x20 | none |
| Crude Bed | unique facility | Twig x50; Pebble x50; Grass x50 | none |
| Crude Furnace | unique facility | Pebble x100 | none |
| Primitive Fishing Rod | stackable item x1 | Twig x5; Vine x5; Small Fang x3 | Stone Table |
