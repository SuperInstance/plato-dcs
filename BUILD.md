# BUILD plato-dcs — DCS Execution Engine

## What to Build
A Rust crate implementing the 7-phase Divide-Conquer-Synthesize (DCS) protocol.

## The 7 Phases
1. DIVIDE: Decompose problem into sub-tasks
2. ASSIGN: Match sub-tasks to specialist agents
3. COMPUTE: Each specialist solves their sub-task
4. VERIFY: Check specialist solutions against constraints
5. SYNTHESIZE: Merge specialist solutions into one
6. VALIDATE: Test synthesized solution against original problem
7. INTEGRATE: Commit or rollback

## Key Numbers (from Oracle1 research)
- 5.88× specialist advantage (single specialist vs generalist)
- 21.87× generalist advantage (DCS fleet vs single generalist)
- These MUST be asserted in tests — if the ratios fail, the engine is wrong

## Design
- State machine: Phase enum + transitions
- Zero external deps (cargo 1.75 compatible)
- Uses plato-tile-spec Tile type for problem/solution representation
- Agent pool: Vec of agent structs with specialty + trust score
- Each phase is a pure function taking state → producing new state

## Test Requirements
- 7 individual phase tests (each phase works in isolation)
- 3 end-to-end cycle tests (full divide→integrate)
- Specialist vs generalist ratio assertions
- Edge cases: empty agent pool, single agent, all agents fail verification

## Cargo.toml
```toml
[package]
name = "plato-dcs"
version = "0.1.0"
edition = "2021"
description = "DCS execution engine — Divide-Conquer-Synthesize protocol, 5.88× specialist, 21.87× generalist"
```

BUILD IT NOW. Write Cargo.toml, src/lib.rs with all 7 phases, comprehensive tests.
No uuid crate (cargo 1.75). Use nanosecond-based IDs. Zero external dependencies.
Push to GitHub when tests pass.
