# plato-dcs

[![crates.io](https://img.shields.io/crates/v/plato-dcs)](https://crates.io/crates/plato-dcs) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)


Dynamic Consensus System — multi-agent belief, lock accumulation, consensus.

When agents disagree, how do they reach consensus? Plato DCS implements a dynamic consensus system where agents accumulate locks on beliefs until a threshold is reached.

## How It Works

1. **Propose** — An agent proposes a belief
2. **Accumulate** — Other agents lock their agreement
3. **Threshold** — When enough agents agree, the belief is accepted
4. **Dynamic** — Thresholds adjust based on confidence and agent expertise

## What It Does

- **Multi-agent belief tracking** — Each agent's beliefs are recorded
- **Lock accumulation** — Agreement is collected, not voted
- **Dynamic thresholds** — Consensus requirements adjust based on stakes
- **Conflict resolution** — When agents disagree, evidence wins

## Installation

```bash
pip install plato-dcs
```

## Part of the Cocapn Fleet

Enables democratic decision-making across the fleet without a central authority.

## License

MIT