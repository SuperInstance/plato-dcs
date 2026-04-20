# plato-dcs

**Divide-Conquer-Synthesize** execution engine. 7-phase protocol for fleet task decomposition.

Proven ratios from fleet research:
- **5.88×** specialist advantage (single specialist vs generalist)
- **21.87×** fleet advantage (DCS fleet vs single generalist)

## Why

One agent can't be good at everything. DCS decomposes a task into specialist subtasks, conquers each independently, then synthesizes results. The synthesis multiplier (3.72×) emerges from combining specialist outputs.

## Usage

```rust
use plato_dcs::DcsEngine;

let engine = DcsEngine::new(specialist_registry);
let result = engine.execute("analyze fleet health").unwrap();
```

Zero dependencies. `cargo add plato-dcs`
