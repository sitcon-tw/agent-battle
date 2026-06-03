# PromptOps Arena Backend Implementation Plan

> Backend-only implementation plan for the PromptOps Arena game server.
>
> This document defines the architecture, module boundaries, domain model, WebSocket protocol, persistence abstraction, simulation engine, testing strategy, and step-by-step implementation milestones.

---

## 1. Project Goal

The backend is a **game server** for a web-based `.io`-style prompt-controlled tactical game.

The server must support:

- Anonymous players joining a room with a chosen display name.
- Players claiming an agent slot and submitting a prompt.
- Admins monitoring and controlling rooms in real time.
- A room owning one child match.
- A match running a deterministic game simulation.
- Replayable game events and state snapshots.
- Memory-based persistence for MVP.
- A future database-backed persistence layer without changing game logic.
- A Rust implementation with strong domain types and thorough tests.

The server should not know anything about external event flow such as “round 1”, “round 2”, “camp activity phase”, or “reflection stage”.

Those are usage patterns outside the game server.

---

## 2. Core Design Principles

### 2.1 Room is the Admin-Facing Game Container

A **Room** is the main object that admins and players interact with.

A room contains:

- Players
- Teams
- Agent slots
- Room status
- Room configuration
- One child match when the game starts

Admins create, monitor, lock, start, reset, and close rooms.

Players join rooms, claim slots, and edit prompts while the room is open.

---

### 2.2 Match is the Room-Owned Simulation Runtime

A **Match** is created from a locked snapshot of a room.

The match owns:

- Immutable match configuration
- Turn state
- Game events
- State snapshots
- Agent decision logs
- Score
- Final result

Once a match is created, player presence no longer affects the simulation.

The match continues using the locked prompt snapshot even if a player leaves.

---

### 2.3 Player is Not a User Account

There are no user accounts.

A **Player** is an anonymous, temporary participant inside a room.

A player has:

- A server-generated player ID
- A chosen display name
- A temporary player token
- Optionally one claimed agent slot

The server does not need to know who the player is in real life.

---

### 2.4 Admin Controls Room Lifecycle

There is no host player.

Players cannot create, lock, start, reset, or delete rooms.

Only admin APIs can control room lifecycle.

This is important for workshop, classroom, and event use cases where staff must monitor and control all game content.

---

### 2.5 WebSocket is the Primary API

The room system is real-time.

Admin dashboards must receive room updates immediately.

Therefore:

- REST is only used for bootstrap, health checks, and optional auth.
- WebSocket is used for room control, player actions, admin monitoring, and live match updates.

---

### 2.6 Persistence is an Interface

The backend must not depend directly on MongoDB or any database.

The server depends on a persistence abstraction.

For MVP:

- Use an in-memory persistence layer.
- Store all game-related data in process-local maps.
- Data is lost when the server restarts.
- Multi-instance deployment is not supported.

Future persistence implementations, such as MongoDB, should be replaceable without changing room services, match services, simulation logic, or WebSocket protocol.

---

## 3. High-Level Architecture

```text
Backend Game Server
├── WebSocket API
│   ├── Admin Socket
│   ├── Room Socket
│   └── Observer Socket
│
├── Services
│   ├── Room Service
│   ├── Match Service
│   └── Admin Service
│
├── Simulation
│   ├── Simulation Engine
│   ├── Rules Engine
│   ├── Movement Engine
│   ├── Combat Engine
│   ├── Visibility Engine
│   ├── Scoring Engine
│   └── Decision Providers
│
├── Persistence Layer
│   ├── Persistence Traits
│   ├── Memory Persistence
│   └── Future Mongo Persistence
│
├── Event Bus
│   └── Runtime WebSocket Broadcast
│
└── Domain Model
    ├── Room
    ├── Player
    ├── Team
    ├── AgentSlot
    ├── Match
    ├── GameState
    ├── GameEvent
    └── AgentDecisionLog
```

---

## 4. Domain Relationship Overview

```text
Room
├── Players
├── Teams
├── AgentSlots
└── Match?
    ├── MatchConfigSnapshot
    ├── GameState
    ├── GameEvents
    ├── GameStateSnapshots
    ├── AgentDecisionLogs
    └── MatchResult
```

### Relationship Rules

- A room is the root object.
- A player only exists inside a room.
- A player may claim at most one agent slot in that room.
- An agent slot belongs to a team and has a role.
- A room can have at most one active child match.
- A match is created from the locked room snapshot.
- A match does not depend on live player connections.
- A match result is reflected back into the room status.

---

## 5. Room Lifecycle

```text
open
→ locked
→ running
→ finished
→ closed
```

### 5.1 `open`

Players can:

- Join the room.
- Leave the room.
- Claim slots.
- Release slots.
- Edit their own prompt.

Admins can:

- Monitor the room.
- Lock the room.
- Reset the room.
- Close the room.

---

### 5.2 `locked`

The game input is fixed.

Players can no longer:

- Claim slots.
- Release slots.
- Edit prompts.

Admin can:

- Unlock the room.
- Start the room.
- Close the room.

When a room is locked:

- Every slot receives a `lockedPrompt`.
- If a slot has no prompt, use the role default prompt.
- If a slot has no player, it can still be filled by a default controller.

---

### 5.3 `running`

The child match is running.

Player presence no longer matters.

The match uses the locked snapshot.

Admins receive real-time match progress events.

---

### 5.4 `finished`

The match has ended.

The room has:

- Match result
- Replay data
- Final score
- Game event history

Admins may reset or close the room.

---

### 5.5 `closed`

The room is no longer active.

Players cannot join.

Admins may keep the room for replay or delete it later.

---

## 6. Match Lifecycle

```text
created
→ running
→ finished

created
→ cancelled

created/running
→ failed
```

### 6.1 `created`

The match exists but has not started simulation.

It already contains an immutable room snapshot.

---

### 6.2 `running`

The simulation engine is actively resolving turns.

---

### 6.3 `finished`

The match has completed normally.

---

### 6.4 `cancelled`

The admin cancelled the match.

---

### 6.5 `failed`

The match crashed or encountered an unrecoverable internal error.

Invalid agent actions should not cause match failure.

Invalid actions should be replaced with fallback actions.

---

## 7. Player Model

### 7.1 Player Definition

A player is an anonymous participant inside a room.

```rust
pub struct Player {
    pub id: PlayerId,
    pub display_name: String,
    pub token_hash: String,
    pub joined_at: Timestamp,
    pub last_seen_at: Timestamp,
}
```

### 7.2 Display Name Rules

The player chooses their own display name.

Rules:

- Trim leading and trailing whitespace.
- Length: 1–20 characters.
- Unicode is allowed.
- Control characters are rejected.
- Display names are not unique.
- Internal identity must always use `PlayerId`, never display name.

Rendering layers should escape display names before showing them in HTML.

---

### 7.3 Leaving and Disconnecting

For MVP, do not distinguish between `left` and `disconnected` in the lobby stage.

If a player is gone before the room is locked:

- Remove the player from the room.
- Release their slot.

If a player is gone after the room is locked:

- Do not alter the locked prompt.
- Do not alter the match.
- The match continues normally.

A short reconnect grace period may be implemented later, but it is not required for the first version.

---

### 7.4 Player Token

When a player joins, the server returns a temporary token.

The token is used only to prove:

> “I am this anonymous player inside this room.”

It is not a login account.

The server stores only a hash of the token.

---

## 8. Team and Slot Model

### 8.1 Team

```rust
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub side: TeamSide,
}
```

```rust
pub enum TeamSide {
    A,
    B,
}
```

MVP has two teams.

Each team has six role slots.

---

### 8.2 AgentSlot

```rust
pub struct AgentSlot {
    pub id: SlotId,
    pub team_id: TeamId,
    pub role: RoleName,
    pub player_id: Option<PlayerId>,
    pub prompt_draft: Option<String>,
    pub locked_prompt: Option<String>,
}
```

A slot represents a role seat in a room.

A player may claim one slot.

The slot is later converted into a match agent when the room starts.

---

### 8.3 Role Names

```rust
pub enum RoleName {
    Vanguard,
    Striker,
    Medic,
    Guardian,
    Scout,
    Engineer,
}
```

Each team has exactly one slot for each role in MVP.

---

## 9. Room Model

```rust
pub struct Room {
    pub id: RoomId,
    pub code: RoomCode,
    pub status: RoomStatus,
    pub config: RoomConfig,

    pub players: Vec<Player>,
    pub teams: Vec<Team>,
    pub slots: Vec<AgentSlot>,

    pub match_id: Option<MatchId>,

    pub version: u64,

    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
```

### 9.1 Room Config

```rust
pub struct RoomConfig {
    pub mode: GameMode,
    pub map_id: MapId,
    pub max_turns: u32,
}
```

```rust
pub enum GameMode {
    PromptOpsArena,
}
```

MVP default:

```text
mode: PromptOpsArena
map: default-15x9
maxTurns: 8
```

---

## 10. Match Model

```rust
pub struct Match {
    pub id: MatchId,
    pub room_id: RoomId,
    pub status: MatchStatus,
    pub config_snapshot: MatchConfigSnapshot,
    pub current_turn: u32,
    pub result: Option<MatchResult>,
    pub created_at: Timestamp,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
}
```

### 10.1 Match Config Snapshot

```rust
pub struct MatchConfigSnapshot {
    pub map_id: MapId,
    pub max_turns: u32,
    pub teams: Vec<MatchTeamConfig>,
}
```

```rust
pub struct MatchTeamConfig {
    pub id: TeamId,
    pub name: String,
    pub side: TeamSide,
    pub agents: Vec<MatchAgentConfig>,
}
```

```rust
pub struct MatchAgentConfig {
    pub agent_id: AgentId,
    pub role: RoleName,
    pub display_name: Option<String>,
    pub source_player_id: Option<PlayerId>,
    pub controller: AgentControllerConfig,
}
```

```rust
pub enum AgentControllerConfig {
    Llm { prompt: String },
    Scripted { strategy_id: String },
    Random,
}
```

MVP should first implement `Scripted` and `Random`.

`Llm` should be added after deterministic simulation is stable.

---

## 11. Game Model

### 11.1 GameState

```rust
pub struct GameState {
    pub match_id: MatchId,
    pub turn: u32,
    pub map: GameMap,
    pub teams: Vec<GameTeam>,
    pub agents: Vec<GameAgent>,
    pub nodes: Vec<ControlNode>,
    pub score: HashMap<TeamId, i32>,
}
```

### 11.2 GameAgent

```rust
pub struct GameAgent {
    pub id: AgentId,
    pub team_id: TeamId,
    pub role: RoleName,
    pub display_name: Option<String>,

    pub hp: i32,
    pub max_hp: i32,
    pub position: Position,
    pub status: AgentStatus,

    pub marked: bool,
    pub shield: i32,
    pub cooldowns: HashMap<SkillName, u32>,
}
```

### 11.3 AgentStatus

```rust
pub enum AgentStatus {
    Active,
    Downed { turns_remaining_before_respawn: u32 },
}
```

### 11.4 Role Stats

```rust
pub struct RoleStats {
    pub hp: i32,
    pub movement: u32,
    pub attack_range: u32,
    pub attack_damage: i32,
}
```

MVP stats:

| Role | HP | Movement | Attack Range | Attack Damage |
|---|---:|---:|---:|---:|
| Vanguard | 14 | 2 | 1 | 2 |
| Striker | 9 | 3 | 3 | 3 |
| Medic | 8 | 3 | 2 | 1 |
| Guardian | 12 | 2 | 2 | 2 |
| Scout | 8 | 4 | 2 | 1 |
| Engineer | 10 | 2 | 3 | 2 |

---

## 12. Map Model

### 12.1 Coordinates

Use zero-based coordinates internally.

```text
x: 0..14
y: 0..8
```

Do not use one-based row/column coordinates internally.

---

### 12.2 Position

```rust
pub struct Position {
    pub x: i32,
    pub y: i32,
}
```

---

### 12.3 Tile

```rust
pub enum Tile {
    Normal,
    Wall,
    Cover,
    Spawn { team_side: TeamSide },
    Node { node_id: NodeId },
}
```

---

### 12.4 Default Map

MVP uses a fixed 15×9 map.

```text
01  .  .  .  #  .  .  o  N  o  .  .  #  .  .  .
02  A  A  .  #  .  .  .  .  .  .  .  #  .  B  B
03  A  A  .  .  .  #  o  .  o  #  .  .  .  B  B
04  .  .  .  o  .  .  .  .  .  .  .  o  .  .  .
05  .  A  .  .  .  o  .  C  .  o  .  .  .  B  .
06  .  .  .  o  .  .  .  .  .  .  .  o  .  .  .
07  A  A  .  .  .  #  o  .  o  #  .  .  .  B  B
08  A  A  .  #  .  .  .  .  .  .  .  #  .  B  B
09  .  .  .  #  .  .  o  S  o  .  .  #  .  .  .
```

Internal map config should use strings or structured tile data, but the parser must validate:

- Width is 15.
- Height is 9.
- All rows have equal length.
- Exactly three control nodes exist.
- Both teams have spawn locations.

---

## 13. Control Nodes

MVP has three nodes:

| Node | Position | Score Per Turn |
|---|---:|---:|
| North Relay | `(7, 0)` | 1 |
| Core Terminal | `(7, 4)` | 2 |
| South Relay | `(7, 8)` | 1 |

### 13.1 Node Control Rule

At the end of each turn:

1. Count active agents within Manhattan distance `<= 1` from each node.
2. Downed agents do not count.
3. The team with more valid agents controls the node.
4. If tied, previous controller remains.
5. The controlling team gains the node score.

---

## 14. Action Model

Agent decisions must use structured data.

```rust
pub struct AgentDecision {
    pub intent: Option<String>,
    pub movement: MovementDecision,
    pub action: ActionDecision,
}
```

### 14.1 MovementDecision

```rust
pub enum MovementDecision {
    Stay,
    MoveTo(Position),
}
```

### 14.2 ActionDecision

```rust
pub enum ActionDecision {
    Wait,
    Attack { target: AgentId },
    Defend,
    UseSkill { skill: SkillName, target: SkillTarget },
    CaptureOrReinforceNode { node_id: NodeId },
}
```

The `intent` field is only for replay and debugging.

It must not affect game rules.

---

## 15. Validation and Fallback

Agent decisions are suggestions.

They never directly mutate game state.

Every decision goes through:

- Movement validation
- Action validation
- Skill validation

If an action is invalid:

1. Record an `INVALID_ACTION_REPLACED` event.
2. Replace the decision with fallback.
3. Continue simulation.

MVP fallback:

```text
movement = Stay
action = Wait
```

Invalid actions should not crash a match.

---

## 16. Turn Resolution Pipeline

Each turn follows the same deterministic pipeline.

```text
1. Emit TURN_STARTED.
2. Build observation for each active agent.
3. Request decision from each agent.
4. Validate decisions.
5. Resolve movement phase.
6. Resolve action phase.
7. Resolve damage, healing, and skill effects.
8. Resolve downed and respawn states.
9. Resolve node control.
10. Apply scoring.
11. Save game events.
12. Save state snapshot.
13. Save agent decision logs.
14. Broadcast match.turnResolved.
15. Advance turn.
```

### 16.1 Simultaneous Decision Rule

All agents decide based on the state at the beginning of the turn.

Do not resolve one agent and then let the next agent observe the changed state.

This avoids turn-order advantage.

---

### 16.2 Movement Conflict Rule

MVP should keep this simple.

If multiple agents attempt to move into the same tile:

1. Resolve by deterministic agent order.
2. The first valid move succeeds.
3. Later conflicting moves are replaced with `Stay`.
4. Record movement conflict events if useful.

The deterministic order should be stable, such as sorting by `AgentId`.

---

## 17. Combat Rules

### 17.1 Attack

An attack is valid if:

- The attacker is active.
- The target exists.
- The target is an enemy.
- The target is active.
- The target is within attack range.
- The target is visible or otherwise legally targetable.
- Line of sight is not blocked by a wall.

### 17.2 Cover

If a target is in cover or protected by cover, ranged damage is reduced.

MVP may use a simple rule:

```text
ranged damage against a target on cover tile is reduced by 1, minimum 1.
```

### 17.3 Downed

When HP reaches 0:

- Agent becomes downed.
- Agent cannot move.
- Agent cannot attack.
- Agent cannot use skills.
- Agent cannot control nodes.

After 2 turns without revival:

- Agent respawns at team spawn.
- Agent returns with half HP.

---

## 18. MVP Skills

Implement skills in two batches.

### 18.1 Core Skill Batch

These should be implemented first.

| Role | Skill | MVP Effect |
|---|---|---|
| Vanguard | Push | Push adjacent enemy 1 tile and deal small damage |
| Striker | Focus Shot | Higher damage attack; bonus if target is marked |
| Medic | Heal | Restore HP to nearby ally |
| Guardian | Hold Position | Reduce incoming damage and improve node control |
| Scout | Mark | Mark a visible enemy |
| Engineer | Hack | Reinforce or quickly control a nearby node |

These six skills are enough to make role identity meaningful.

---

### 18.2 Extended Skill Batch

Implement after core simulation is stable.

| Role | Skill |
|---|---|
| Vanguard | Guard |
| Striker | Reposition |
| Medic | Revive |
| Medic | Shield Patch |
| Guardian | Barrier |
| Scout | Scan |
| Scout | Slip |
| Engineer | Turret |
| Engineer | Repair |

---

## 19. Decision Providers

### 19.1 DecisionProvider Trait

```rust
#[async_trait]
pub trait DecisionProvider: Send + Sync {
    async fn decide(
        &self,
        agent: &GameAgent,
        observation: AgentObservation,
    ) -> Result<AgentDecision, DecisionError>;
}
```

### 19.2 Implementations

Implement in this order:

1. `ScriptedDecisionProvider`
2. `RandomDecisionProvider`
3. `LlmDecisionProvider`

Do not start with LLM.

Rules engine bugs are much easier to debug with deterministic scripted decisions.

---

### 19.3 LLM Decision Provider

Add only after deterministic simulation works.

LLM provider responsibilities:

- Build agent observation.
- Compose model input.
- Include player prompt snapshot.
- Request structured JSON output.
- Parse output.
- Validate decision.
- Timeout and fallback.
- Save decision log.

The LLM must never directly mutate game state.

---

## 20. Observation Model

Each agent receives an observation, not the full internal game state.

```rust
pub struct AgentObservation {
    pub turn: u32,
    pub self_agent: PublicAgentState,
    pub visible_allies: Vec<PublicAgentState>,
    pub visible_enemies: Vec<PublicAgentState>,
    pub known_nodes: Vec<PublicNodeState>,
    pub score: HashMap<TeamId, i32>,
    pub map_summary: PublicMapSummary,
    pub legal_action_hints: Vec<String>,
}
```

Do not include:

- Enemy prompts.
- Hidden enemies.
- Future decisions.
- Admin-only debug data.
- Internal player tokens.

---

## 21. Visibility Rules

MVP visibility:

```text
Normal agents can see enemies within Manhattan distance <= 4.
Scout-related skills may reveal additional enemies later.
Walls do not need to block visibility in MVP.
Walls do block ranged attacks.
```

This keeps first-version rules understandable.

---

## 22. Replay and Logging

Replay is a game server feature.

It is not specific to a camp activity.

---

### 22.1 GameEvent

```rust
pub struct GameEvent {
    pub id: EventId,
    pub match_id: MatchId,
    pub turn: u32,
    pub sequence: u64,
    pub event_type: GameEventType,
    pub actor_agent_id: Option<AgentId>,
    pub payload: serde_json::Value,
    pub created_at: Timestamp,
}
```

### 22.2 GameEvent Types

```text
MATCH_STARTED
TURN_STARTED
AGENT_DECISION_RECEIVED
INVALID_ACTION_REPLACED
AGENT_MOVED
MOVEMENT_CONFLICT
AGENT_ATTACKED
SKILL_USED
DAMAGE_DEALT
AGENT_HEALED
AGENT_DOWNED
AGENT_RESPAWNED
NODE_CONTROL_CHANGED
SCORE_CHANGED
TURN_FINISHED
MATCH_FINISHED
MATCH_FAILED
```

---

### 22.3 AgentDecisionLog

```rust
pub struct AgentDecisionLog {
    pub match_id: MatchId,
    pub turn: u32,
    pub agent_id: AgentId,

    pub observation: AgentObservation,
    pub prompt_snapshot: Option<String>,

    pub raw_output: Option<String>,
    pub parsed_decision: Option<AgentDecision>,
    pub validated_decision: AgentDecision,

    pub was_fallback_used: bool,
    pub error: Option<String>,
    pub latency_ms: Option<u64>,
}
```

This is essential for debugging why an agent behaved a certain way.

---

### 22.4 GameStateSnapshot

```rust
pub struct GameStateSnapshot {
    pub match_id: MatchId,
    pub turn: u32,
    pub state: PublicGameState,
    pub created_at: Timestamp,
}
```

Save one snapshot at the end of each turn.

---

## 23. Persistence Layer

### 23.1 Persistence Responsibility

Persistence must store all game-related data:

- Rooms
- Players
- Slots
- Matches
- Game events
- State snapshots
- Agent decision logs
- Admin audit logs

---

### 23.2 Persistence Traits

```rust
pub trait Persistence: Send + Sync {
    fn rooms(&self) -> &dyn RoomRepository;
    fn matches(&self) -> &dyn MatchRepository;
    fn events(&self) -> &dyn GameEventRepository;
    fn states(&self) -> &dyn GameStateRepository;
    fn decisions(&self) -> &dyn AgentDecisionRepository;
    fn audit_logs(&self) -> &dyn AdminAuditLogRepository;
}
```

Use `async_trait` for async repository methods in MVP.

---

### 23.3 RoomRepository

```rust
#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn create_room(&self, input: CreateRoomInput) -> Result<Room, RepoError>;
    async fn get_room(&self, room_id: &RoomId) -> Result<Option<Room>, RepoError>;
    async fn get_room_by_code(&self, code: &RoomCode) -> Result<Option<Room>, RepoError>;
    async fn list_rooms(&self) -> Result<Vec<Room>, RepoError>;

    async fn add_player(&self, input: AddPlayerInput) -> Result<(Room, Player), RepoError>;
    async fn remove_player(&self, input: RemovePlayerInput) -> Result<Room, RepoError>;

    async fn claim_slot(&self, input: ClaimSlotInput) -> Result<Room, RepoError>;
    async fn release_slot(&self, input: ReleaseSlotInput) -> Result<Room, RepoError>;
    async fn update_slot_prompt(&self, input: UpdateSlotPromptInput) -> Result<Room, RepoError>;

    async fn lock_room(&self, input: LockRoomInput) -> Result<Room, RepoError>;
    async fn unlock_room(&self, room_id: &RoomId) -> Result<Room, RepoError>;

    async fn attach_match(&self, room_id: &RoomId, match_id: &MatchId) -> Result<Room, RepoError>;
    async fn set_status(&self, room_id: &RoomId, status: RoomStatus) -> Result<Room, RepoError>;
}
```

Important rule:

`claim_slot` must be one repository operation.

Do not implement it as `get → modify → save` at the service level.

This allows a future database implementation to make slot claiming atomic.

---

### 23.4 MatchRepository

```rust
#[async_trait]
pub trait MatchRepository: Send + Sync {
    async fn create_match(&self, input: CreateMatchInput) -> Result<Match, RepoError>;
    async fn get_match(&self, match_id: &MatchId) -> Result<Option<Match>, RepoError>;
    async fn get_match_by_room_id(&self, room_id: &RoomId) -> Result<Option<Match>, RepoError>;

    async fn set_running(&self, match_id: &MatchId) -> Result<Match, RepoError>;
    async fn set_finished(&self, match_id: &MatchId, result: MatchResult) -> Result<Match, RepoError>;
    async fn set_failed(&self, match_id: &MatchId, error: MatchError) -> Result<Match, RepoError>;

    async fn update_current_turn(&self, match_id: &MatchId, turn: u32) -> Result<Match, RepoError>;
}
```

---

### 23.5 GameEventRepository

```rust
#[async_trait]
pub trait GameEventRepository: Send + Sync {
    async fn append_event(&self, event: GameEvent) -> Result<(), RepoError>;
    async fn append_events(&self, events: Vec<GameEvent>) -> Result<(), RepoError>;

    async fn list_events_by_match(&self, match_id: &MatchId) -> Result<Vec<GameEvent>, RepoError>;
    async fn list_events_by_turn(&self, match_id: &MatchId, turn: u32) -> Result<Vec<GameEvent>, RepoError>;
}
```

---

### 23.6 GameStateRepository

```rust
#[async_trait]
pub trait GameStateRepository: Send + Sync {
    async fn save_snapshot(&self, snapshot: GameStateSnapshot) -> Result<(), RepoError>;

    async fn get_snapshot(
        &self,
        match_id: &MatchId,
        turn: u32,
    ) -> Result<Option<GameStateSnapshot>, RepoError>;

    async fn list_snapshots(&self, match_id: &MatchId) -> Result<Vec<GameStateSnapshot>, RepoError>;
}
```

---

### 23.7 AgentDecisionRepository

```rust
#[async_trait]
pub trait AgentDecisionRepository: Send + Sync {
    async fn save_decision(&self, decision: AgentDecisionLog) -> Result<(), RepoError>;

    async fn list_decisions_by_match(
        &self,
        match_id: &MatchId,
    ) -> Result<Vec<AgentDecisionLog>, RepoError>;

    async fn list_decisions_by_agent(
        &self,
        match_id: &MatchId,
        agent_id: &AgentId,
    ) -> Result<Vec<AgentDecisionLog>, RepoError>;
}
```

---

## 24. Memory Persistence

### 24.1 Memory Store

```rust
pub struct MemoryPersistence {
    rooms: RwLock<HashMap<RoomId, Room>>,
    rooms_by_code: RwLock<HashMap<RoomCode, RoomId>>,

    matches: RwLock<HashMap<MatchId, Match>>,

    game_events: RwLock<HashMap<MatchId, Vec<GameEvent>>>,
    game_states: RwLock<HashMap<MatchId, Vec<GameStateSnapshot>>>,
    agent_decisions: RwLock<HashMap<MatchId, Vec<AgentDecisionLog>>>,

    admin_audit_logs: RwLock<Vec<AdminAuditLog>>,
}
```

Use `tokio::sync::RwLock` if repositories are async.

---

### 24.2 Memory Persistence Limitations

The memory persistence layer is:

- Volatile
- Process-local
- Lost on server restart
- Not suitable for multiple backend instances
- Intended for MVP and local development

This is acceptable for first implementation.

---

## 25. Runtime Event Bus

The runtime event bus is for WebSocket broadcasting.

It is not the same thing as persisted game events.

```rust
pub struct EventBus {
    // implementation can use tokio::sync::broadcast
}
```

Topics:

```text
admin
room:{room_id}
```

### 25.1 Event Bus vs GameEventRepository

| System | Purpose | Persistence |
|---|---|---|
| EventBus | Live WebSocket updates | No |
| GameEventRepository | Replay and match history | Yes / memory-stored in MVP |

---

## 26. WebSocket API

### 26.1 Endpoints

```text
/ws/admin
/ws/rooms/:room_code
/ws/rooms/:room_id/watch
```

### 26.2 REST Endpoints

Keep REST minimal.

```text
GET  /health
GET  /config
POST /admin/auth
GET  /rooms/:room_code/bootstrap
```

Everything else should happen over WebSocket.

---

## 27. WebSocket Message Protocol

### 27.1 Client Command

```rust
pub struct ClientCommand {
    pub id: CommandId,
    pub r#type: String,
    pub payload: serde_json::Value,
}
```

Example:

```json
{
  "id": "cmd_001",
  "type": "slot.claim",
  "payload": {
    "roomId": "room_123",
    "slotId": "slot_a_medic"
  }
}
```

---

### 27.2 Server Message

```rust
pub enum ServerMessage {
    Ack(CommandAck),
    Event(ServerEvent),
    Snapshot(SnapshotMessage),
    Error(ServerErrorMessage),
}
```

### 27.3 Command Ack

```rust
pub struct CommandAck {
    pub command_id: CommandId,
    pub ok: bool,
    pub payload: Option<serde_json::Value>,
    pub error: Option<CommandError>,
}
```

Every command must receive an ack.

---

## 28. Admin WebSocket

Endpoint:

```text
/ws/admin
```

Admin WebSocket requires admin authentication.

MVP can use a simple `ADMIN_API_KEY`.

---

### 28.1 Admin Commands

```text
admin.subscribe.rooms
admin.subscribe.room

admin.room.create
admin.room.lock
admin.room.unlock
admin.room.start
admin.room.cancel
admin.room.reset
admin.room.close
admin.room.delete

admin.slot.clear
admin.player.kick
```

---

### 28.2 Admin Live Updates

Admin must receive real-time updates for:

- Room created
- Room updated
- Room closed
- Player joined
- Player left
- Slot claimed
- Slot released
- Prompt updated
- Room locked
- Room started
- Match turn resolved
- Match finished
- Match failed

---

## 29. Room WebSocket

Endpoint:

```text
/ws/rooms/:room_code
```

### 29.1 Player Commands

```text
room.join
room.leave
room.subscribe
room.rename

slot.claim
slot.release
slot.updatePrompt
```

---

### 29.2 Player Join Flow

```text
1. Client connects to /ws/rooms/:room_code.
2. Client sends room.join with displayName.
3. Server validates room is open.
4. Server creates Player.
5. Server returns playerId and playerToken in ack.
6. Server sends full room snapshot.
7. Server broadcasts player.joined to room and admin subscribers.
```

---

### 29.3 Slot Claim Flow

```text
1. Player sends slot.claim.
2. Server authenticates player token.
3. Server requires room.status == open.
4. Server requires slot is empty.
5. Server requires player has no other slot.
6. Repository claims slot atomically.
7. Server acks success.
8. Server broadcasts slot.claimed to room and admin subscribers.
```

---

### 29.4 Prompt Update Flow

```text
1. Player sends slot.updatePrompt.
2. Server authenticates player token.
3. Server requires room.status == open.
4. Server requires slot belongs to player.
5. Server validates prompt length.
6. Server saves prompt draft.
7. Server acks success.
8. Server broadcasts slot.promptUpdated.
```

---

## 30. Room Events

```rust
pub enum RoomEvent {
    RoomCreated { room: RoomSummary },
    RoomUpdated { room_id: RoomId, version: u64 },
    RoomClosed { room_id: RoomId },
    RoomDeleted { room_id: RoomId },

    PlayerJoined { room_id: RoomId, player: PublicPlayer },
    PlayerLeft { room_id: RoomId, player_id: PlayerId },
    PlayerRenamed { room_id: RoomId, player_id: PlayerId, display_name: String },

    SlotClaimed { room_id: RoomId, slot_id: SlotId, player_id: PlayerId },
    SlotReleased { room_id: RoomId, slot_id: SlotId },
    SlotPromptUpdated { room_id: RoomId, slot_id: SlotId, updated_by: PlayerId },

    RoomLocked { room_id: RoomId },
    RoomUnlocked { room_id: RoomId },
    RoomStarted { room_id: RoomId, match_id: MatchId },
    RoomFinished { room_id: RoomId, match_id: MatchId, result: MatchResult },
    RoomReset { room_id: RoomId },
}
```

---

## 31. Match Events Sent Through Room Channel

Because a room owns its child match, room subscribers should receive match updates.

```rust
pub enum MatchLiveEvent {
    MatchCreated { room_id: RoomId, match_id: MatchId },
    MatchStarted { room_id: RoomId, match_id: MatchId },
    MatchTurnStarted { room_id: RoomId, match_id: MatchId, turn: u32 },
    MatchTurnResolved {
        room_id: RoomId,
        match_id: MatchId,
        turn: u32,
        snapshot: PublicGameState,
    },
    MatchScoreUpdated {
        room_id: RoomId,
        match_id: MatchId,
        score: HashMap<TeamId, i32>,
    },
    MatchFinished { room_id: RoomId, match_id: MatchId, result: MatchResult },
    MatchFailed { room_id: RoomId, match_id: MatchId, error: String },
}
```

---

## 32. RoomService

### 32.1 Responsibilities

RoomService owns room-related application logic.

It should not know how WebSocket works.

It should not know whether persistence is memory or database.

It should provide methods that WebSocket handlers call.

---

### 32.2 Methods

```rust
pub struct RoomService<P: Persistence> {
    persistence: Arc<P>,
    event_bus: Arc<EventBus>,
}
```

```text
create_room
join_room
leave_room
rename_player

claim_slot
release_slot
update_prompt

lock_room
unlock_room
start_room
cancel_room
reset_room
close_room

get_room_state
list_rooms
```

---

### 32.3 Lock Room Algorithm

```text
1. Load room.
2. Require room.status == open.
3. For each slot:
   - If promptDraft exists, copy it to lockedPrompt.
   - Otherwise use default prompt for role.
4. Set room.status = locked.
5. Increment room.version.
6. Save room.
7. Publish room.locked event.
8. Return updated room.
```

---

### 32.4 Start Room Algorithm

```text
1. Load room.
2. Require room.status == locked.
3. Build MatchConfigSnapshot from room:
   - mapId
   - maxTurns
   - teams
   - agents from locked slots
4. Create child Match.
5. Attach matchId to room.
6. Set room.status = running.
7. Set match.status = running.
8. Publish room.started and match.started events.
9. Spawn simulation task.
10. On simulation success:
    - Set match.status = finished.
    - Set room.status = finished.
    - Publish room.finished event.
11. On simulation failure:
    - Set match.status = failed.
    - Publish match.failed event.
```

---

## 33. MatchService

### 33.1 Responsibilities

MatchService owns:

- Creating match from room snapshot.
- Starting match simulation.
- Updating match status.
- Saving match result.
- Providing replay data.

MatchService should not handle player join, slot claim, or room lifecycle directly.

---

## 34. SimulationEngine

### 34.1 Responsibilities

SimulationEngine owns the actual game simulation.

It should be deterministic when given:

- Same match config
- Same decisions
- Same random seed, if randomness is later added

MVP should avoid gameplay randomness.

---

### 34.2 Simulation Methods

```rust
pub async fn run_match(&self, match_id: MatchId) -> Result<MatchResult, SimulationError>;

pub async fn simulate_next_turn(
    &self,
    state: GameState,
) -> Result<TurnResult, SimulationError>;
```

---

### 34.3 TurnResult

```rust
pub struct TurnResult {
    pub new_state: GameState,
    pub public_snapshot: PublicGameState,
    pub events: Vec<GameEvent>,
    pub decisions: Vec<AgentDecisionLog>,
}
```

---

## 35. Testing Strategy

Tests are a core part of the implementation, not an afterthought.

---

### 35.1 Domain Tests

Test:

- Room status transitions.
- Display name validation.
- Player can claim one slot.
- Player cannot claim two slots.
- Player cannot edit another player's prompt.
- Room cannot start unless locked.
- Locked room cannot accept prompt edits.

---

### 35.2 Persistence Tests

Test MemoryPersistence:

- Create room.
- Get room by ID.
- Get room by code.
- Add player.
- Remove player.
- Claim slot.
- Reject claim if slot is occupied.
- Update prompt.
- Lock room.
- Attach match.
- Save events.
- Save snapshots.
- Save decision logs.

---

### 35.3 RoomService Tests

Test:

- `create_room` creates teams and slots.
- `join_room` returns player and token.
- `claim_slot` enforces ownership rules.
- `update_prompt` enforces slot ownership.
- `lock_room` fills missing prompts.
- `start_room` creates child match.

---

### 35.4 Rules Engine Tests

Test:

- Movement distance.
- Walls block movement.
- Cover is passable.
- Cannot end movement on occupied tile.
- Attack range.
- Line of sight.
- Cover damage reduction.
- Downed state.
- Respawn after 2 turns.
- Node control.
- Node scoring.
- Each core skill.

---

### 35.5 Golden Simulation Tests

Golden tests should run a full match with scripted decisions.

Test:

```text
Given fixed match config
And fixed scripted decisions
When the match runs for 8 turns
Then final score is exactly expected
And event order is stable
And snapshots are saved for every turn
```

Golden simulation tests are the most important protection against accidental game rule regressions.

---

### 35.6 WebSocket Integration Tests

Test:

- Admin connects and receives rooms snapshot.
- Player joins room over WebSocket.
- Admin receives `player.joined`.
- Player claims slot.
- Admin receives `slot.claimed`.
- Player updates prompt.
- Admin receives `slot.promptUpdated`.
- Admin locks room.
- Player can no longer update prompt.
- Admin starts room.
- Admin receives `room.started`.
- Admin receives `match.turnResolved`.
- Admin receives `room.finished`.

---

## 36. Suggested Rust Crates

Use current stable versions at implementation time.

Recommended crates:

```text
tokio
axum
tower
tower-http
serde
serde_json
uuid
thiserror
anyhow
async-trait
tracing
tracing-subscriber
chrono or time
```

Recommended optional crates later:

```text
validator
proptest
insta
schemars
```

Use `thiserror` for domain and service errors.

Use `anyhow` only at application boundaries where exact error type is less important.

Use `tracing` for structured logs.

---

## 37. Project Structure

```text
src/
├── main.rs
├── app.rs
├── config.rs
│
├── domain/
│   ├── mod.rs
│   ├── ids.rs
│   ├── room.rs
│   ├── player.rs
│   ├── team.rs
│   ├── slot.rs
│   ├── match.rs
│   ├── map.rs
│   ├── agent.rs
│   ├── action.rs
│   ├── event.rs
│   └── error.rs
│
├── persistence/
│   ├── mod.rs
│   ├── traits.rs
│   └── memory.rs
│
├── services/
│   ├── mod.rs
│   ├── room_service.rs
│   ├── match_service.rs
│   └── admin_service.rs
│
├── simulation/
│   ├── mod.rs
│   ├── engine.rs
│   ├── rules.rs
│   ├── movement.rs
│   ├── combat.rs
│   ├── scoring.rs
│   ├── visibility.rs
│   ├── skills.rs
│   └── decisions.rs
│
├── websocket/
│   ├── mod.rs
│   ├── protocol.rs
│   ├── gateway.rs
│   ├── admin_socket.rs
│   ├── room_socket.rs
│   └── subscriptions.rs
│
└── http/
    ├── mod.rs
    ├── health.rs
    └── bootstrap.rs
```

---

## 38. Implementation Milestones

### Milestone 1: Rust Project Skeleton

Status: complete as of 2026-06-04.

Build:

- Cargo project
- Module structure
- Config loading
- Health endpoint
- Basic tracing setup

Done when:

- `cargo test` passes.
- `cargo run` starts the server.
- `GET /health` returns OK.

---

### Milestone 2: Domain Types

Status: complete as of 2026-06-04.

Build:

- ID newtypes
- Room
- Player
- Team
- AgentSlot
- Match
- Map
- Agent
- Action
- Event types
- Error types

Done when:

- Domain compiles.
- Basic domain tests pass.

---

### Milestone 3: Persistence Traits and MemoryPersistence

Status: complete as of 2026-06-04.

Build:

- Repository traits
- Memory room repository
- Memory match repository
- Memory event repository
- Memory snapshot repository
- Memory decision repository

Done when:

- Rooms can be created and retrieved.
- Players can be added.
- Slots can be claimed.
- Prompts can be updated.
- Events and snapshots can be stored.
- Persistence tests pass.

---

### Milestone 4: RoomService

Status: complete as of 2026-06-04.

Build:

- Create room
- Join room
- Leave room
- Rename player
- Claim slot
- Release slot
- Update prompt
- Lock room
- Unlock room

Done when:

- Room lifecycle works without WebSocket.
- Unit tests cover all service methods.

---

### Milestone 5: EventBus

Status: complete as of 2026-06-04.

Build:

- Runtime event bus
- Admin topic
- Room topic
- Publish/subscribe abstraction

Done when:

- Service actions can publish events.
- Test subscriber receives room events.

---

### Milestone 6: WebSocket Protocol Skeleton

Build:

- Client command envelope
- Server message envelope
- Ack handling
- Error handling
- Admin socket
- Room socket

Done when:

- Admin can connect.
- Player can connect.
- Commands receive acks.
- Invalid commands receive errors.

---

### Milestone 7: Admin Room Commands

Build:

- `admin.subscribe.rooms`
- `admin.room.create`
- `admin.room.lock`
- `admin.room.unlock`
- `admin.room.close`
- `admin.room.reset`

Done when:

- Admin can create and control rooms over WebSocket.
- Admin receives real-time room events.

---

### Milestone 8: Player Room Commands

Build:

- `room.join`
- `room.leave`
- `room.subscribe`
- `room.rename`
- `slot.claim`
- `slot.release`
- `slot.updatePrompt`

Done when:

- Player can join a room with display name.
- Player can claim one slot.
- Player can update their prompt.
- Admin receives all updates in real time.

---

### Milestone 9: Match Creation

Build:

- Build `MatchConfigSnapshot` from locked room.
- Create child match.
- Attach match to room.
- Set room running.
- Set match running.

Done when:

- Admin can start a locked room.
- A child match is created from room snapshot.
- Player changes after match creation do not affect match config.

---

### Milestone 10: Map, Movement, and Node Scoring

Build:

- Default 15×9 map
- Spawn positions
- GameState initialization
- Movement validation
- Node control
- Scoring
- Scripted decision provider
- Full 8-turn simulation without combat

Done when:

- A scripted match can finish.
- Score is calculated.
- Snapshots and events are saved.

---

### Milestone 11: Combat

Build:

- Attack validation
- HP
- Damage
- Downed state
- Respawn timer
- Cover damage reduction
- Line of sight

Done when:

- Agents can damage and down each other.
- Downed agents cannot control nodes.
- Combat tests pass.

---

### Milestone 12: Core Skills

Build:

- Vanguard Push
- Striker Focus Shot
- Medic Heal
- Guardian Hold Position
- Scout Mark
- Engineer Hack

Done when:

- Each role has one meaningful skill.
- Skill events are logged.
- Skill tests pass.

---

### Milestone 13: Match Live Updates

Build:

- `match.started`
- `match.turnStarted`
- `match.turnResolved`
- `match.scoreUpdated`
- `match.finished`
- `match.failed`

Done when:

- Room subscribers receive turn updates.
- Admin receives turn updates.
- Room becomes finished when match finishes.

---

### Milestone 14: Replay and Result Retrieval

Build either WebSocket commands or simple HTTP endpoints:

```text
room.getResult
room.getReplay
```

or

```text
GET /rooms/:room_id/result
GET /rooms/:room_id/replay
```

Done when:

- A finished room can return match result.
- A finished room can return events, snapshots, and decision logs.

---

### Milestone 15: Golden Simulation Tests

Build:

- Fixed scripted strategies
- Full-match expected score tests
- Stable event sequence tests

Done when:

- At least one full 8-turn golden simulation test passes.
- Future rule changes can be checked against expected results.

---

### Milestone 16: LLM Decision Provider

Build:

- Agent observation builder
- LLM prompt composer
- Structured JSON parser
- Timeout
- Fallback
- Decision logging

Done when:

- LLM-controlled agents can play a full match.
- Invalid output does not crash the match.
- Fallback actions are logged.
- Agent decision logs explain each decision.

---

## 39. MVP Definition of Done

The backend MVP is complete when:

- Admin can connect over WebSocket.
- Admin can create a room.
- Admin receives real-time room updates.
- Player can join a room with a display name.
- Player can claim one slot.
- Player can submit prompt draft.
- Admin can lock the room.
- Admin can start the room.
- Room creates one child match.
- Match runs 8 turns.
- Match writes game events.
- Match writes state snapshots.
- Match writes agent decision logs.
- Room becomes finished.
- Admin and room clients receive live match updates.
- Replay and result can be retrieved.
- Persistence is memory-based behind interfaces.
- No MongoDB implementation is required.
- Core rules have unit tests.
- At least one full-match golden simulation test exists.

---

## 40. Non-Goals for MVP

Do not implement these in MVP:

- User accounts
- OAuth login
- Host player
- MongoDB connection
- Multi-instance backend deployment
- Complex reconnect handling
- Complex fog of war
- Equipment system
- Random hit rate
- Critical hits
- Multiple maps
- Tournament bracket
- Camp round 1 / round 2 workflow
- Reflection forms
- Full admin account system

These can be added later if needed.

---

## 41. Final Architecture Summary

```text
Player
- Anonymous room participant.
- Chooses a display name.
- Claims a slot.
- Writes prompt.
- Does not control room lifecycle.

Admin
- Controls rooms.
- Monitors all rooms in real time.
- Starts and resets games.
- Not part of the match itself.

Room
- Admin-facing game container.
- Owns players, teams, slots, and one child match.
- Main WebSocket resource.

Slot
- Room-level role seat.
- May be claimed by a player.
- Holds prompt draft and locked prompt.

Match
- Child runtime of a room.
- Created from locked room snapshot.
- Runs deterministic simulation.
- Owns events, snapshots, decisions, score, and result.

Persistence
- Interface-based.
- MVP uses memory.
- Future database implementation should not change game logic.

WebSocket
- Primary API.
- Used for admin control, player actions, and live updates.

Simulation
- Deterministic rules engine.
- Agent decisions are validated.
- Invalid actions fallback instead of crashing.
- Replay logs are always produced.
```

---

## 42. Recommended First Coding Task

Start with:

```text
Task 1:
Create the Rust project skeleton and implement domain types for Room, Player, Team, AgentSlot, Match, IDs, and RoomStatus.

Requirements:
- Use typed ID wrappers.
- Use serde for serialization.
- Add basic constructors where useful.
- Add validation for display names.
- Add unit tests for Room creation and display name validation.
```

Do not start with WebSocket or LLM.

The safest implementation path is:

```text
Domain
→ Memory Persistence
→ Room Service
→ WebSocket
→ Match Creation
→ Simulation
→ Replay
→ LLM
```
