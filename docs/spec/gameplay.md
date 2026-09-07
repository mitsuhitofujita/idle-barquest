# Gameplay and world state

## Content and live state

A `Catalog` owns immutable templates for targets, settlements, locations,
actions, resources, and Location/Action reward tables. Each kind is addressed
by a stable string id. Templates are stored in registration order, which is
also the display order used by menus and materials.

`GameState` owns the current settlement, target instances, unlocked locations
and actions, and resource stacks. Target instances refer to their template by
id and each can hold at most one task. New known target instances can be spawned
with a unique id; known locations and actions can be unlocked idempotently.

The built-in world starts at Awakening Shore with one Hero. It unlocks First
Shore, Nearby Woods, and Nearby Hill, plus Gather, Fish, and Hunt. Each action
takes ten seconds. The catalog contains Pebble, Twig, Grass, Vine, Small Fish,
Seaweed Fragment, Small Fang, Awful Meat, and Tiny Magic Stone.

## Assignment and progress

A task combines a target instance, a location, an action, and tick-based
progress. Assignment succeeds only when all referenced content exists, the
location and action are unlocked, both target and location support the action,
the target is idle, and the Location/Action pair has a valid non-empty reward
table whose entries reference known resources.

Available actions are the unlocked actions supported by both the selected target
and location, in unlock order. Progress advances toward a clamped nonzero goal
and saturates when complete. Multiple target instances can therefore run one
task each during the same state advance.

## Completion and rewards

Each reward-table entry independently awards a positive resource amount at its
integer percentage chance. Guaranteed entries do not consume randomness.
Successful duplicate resource entries are combined with saturating addition
while preserving the first successful entry's order.

When a task completes, every awarded resource is added to inventory with
saturating addition. The task is removed and core emits a completion event
containing the target, location, action, and ordered aggregated reward list.
An empty list represents no reward.

Inventory storage follows first-acquisition order, while the acquired-resource
projection used by the UI follows catalog order. The presence of a stack marks
a resource as acquired even when its amount is zero.

## Built-in compatibility and rewards

The Hero supports all three actions. First Shore supports Gather and Fish;
Nearby Woods and Nearby Hill support Gather and Hunt. Each supported combination
has a reward table:

| Location | Action | Independent rewards |
| --- | --- | --- |
| First Shore | Gather | Seaweed Fragment 60%; Pebble 100% |
| First Shore | Fish | Small Fish 30%; Seaweed Fragment 100% |
| Nearby Woods | Gather | Vine 60%; Twig 100% |
| Nearby Woods | Hunt | Awful Meat 100%; Tiny Magic Stone 10% |
| Nearby Hill | Gather | Grass 100%; Pebble 50% |
| Nearby Hill | Hunt | Awful Meat 100%; Small Fang 60% |

Every successful entry currently awards one unit.
