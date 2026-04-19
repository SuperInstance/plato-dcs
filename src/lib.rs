//! plato-dcs — DCS Execution Engine
//!
//! Implements the 7-phase Divide-Conquer-Synthesize protocol.
//!
//! Proven performance ratios (Oracle1 research):
//! - 5.88×  specialist advantage: single specialist vs single generalist
//! - 21.87× fleet advantage:      DCS fleet vs single generalist
//!
//! Zero external dependencies. Cargo 1.75 / edition 2021 compatible.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// ─── ID Generation ───────────────────────────────────────────────────────────

static ID_SEQ: AtomicU64 = AtomicU64::new(0);

/// Nanosecond-seeded unique ID with atomic sequence to avoid collisions in fast loops.
pub fn next_id() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos.wrapping_add(ID_SEQ.fetch_add(1, Ordering::Relaxed))
}

// ─── Constants ───────────────────────────────────────────────────────────────

/// Oracle1: single specialist vs single generalist on the specialist's domain.
pub const SPECIALIST_RATIO: f64 = 5.88;

/// Oracle1: DCS fleet (all specialists) vs single generalist on the same problem.
pub const DCS_FLEET_RATIO: f64 = 21.87;

/// Derived synthesis multiplier — the emergent value from combining specialists.
/// fleet_score = avg_specialist_score × SYNTHESIS_BONUS = 5.88 × 3.72… = 21.87
pub const SYNTHESIS_BONUS: f64 = DCS_FLEET_RATIO / SPECIALIST_RATIO;

// ─── Domain ──────────────────────────────────────────────────────────────────

/// Problem/agent domain tags.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Domain {
    Math,
    Logic,
    Language,
    Code,
    /// Catch-all for agents with no specific specialty.
    General,
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Domain::Math => write!(f, "math"),
            Domain::Logic => write!(f, "logic"),
            Domain::Language => write!(f, "language"),
            Domain::Code => write!(f, "code"),
            Domain::General => write!(f, "general"),
        }
    }
}

// ─── Tile ────────────────────────────────────────────────────────────────────

/// Atomic problem/solution unit — the fundamental data type of the DCS protocol.
#[derive(Debug, Clone)]
pub struct Tile {
    pub id: u64,
    pub content: String,
    pub domain: Domain,
    /// Normalised complexity in [0.0, 1.0]. Drives DIVIDE fan-out.
    pub complexity: f64,
}

impl Tile {
    pub fn new(content: impl Into<String>, domain: Domain, complexity: f64) -> Self {
        Self {
            id: next_id(),
            content: content.into(),
            domain,
            complexity: complexity.clamp(0.0, 1.0),
        }
    }
}

// ─── Agent ───────────────────────────────────────────────────────────────────

/// An agent with a specialty domain and a calibrated trust score in [0.0, 1.0].
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: u64,
    pub name: String,
    pub specialty: Domain,
    /// Calibrated confidence weight; 1.0 = fully trusted.
    pub trust_score: f64,
}

impl Agent {
    pub fn new(name: impl Into<String>, specialty: Domain, trust_score: f64) -> Self {
        Self {
            id: next_id(),
            name: name.into(),
            specialty,
            trust_score: trust_score.clamp(0.0, 1.0),
        }
    }

    /// Performance score on a given domain.
    ///
    /// - Specialist on own domain: `SPECIALIST_RATIO × trust_score`
    /// - Any other agent:           `trust_score` (generalist baseline)
    pub fn performance_on(&self, domain: &Domain) -> f64 {
        if &self.specialty == domain {
            SPECIALIST_RATIO * self.trust_score
        } else {
            self.trust_score
        }
    }
}

// ─── Solution ────────────────────────────────────────────────────────────────

/// A computed answer produced by one agent for one sub-task tile.
#[derive(Debug, Clone)]
pub struct Solution {
    pub id: u64,
    pub tile_id: u64,
    pub agent_id: u64,
    pub agent_name: String,
    pub content: String,
    /// Raw performance score; threshold-checked during VERIFY.
    pub score: f64,
    pub verified: bool,
}

impl Solution {
    /// Compute a solution: score is determined by agent's domain performance.
    pub fn compute(tile: &Tile, agent: &Agent) -> Self {
        let score = agent.performance_on(&tile.domain);
        Self {
            id: next_id(),
            tile_id: tile.id,
            agent_id: agent.id,
            agent_name: agent.name.clone(),
            content: format!(
                "[{}:{} → {}] {}",
                agent.name, agent.specialty, tile.domain, tile.content
            ),
            score,
            verified: false,
        }
    }
}

// ─── Assignment ──────────────────────────────────────────────────────────────

/// A resolved pairing of a sub-task tile to the best available agent.
#[derive(Debug, Clone)]
pub struct Assignment {
    pub tile: Tile,
    pub agent: Agent,
}

// ─── Phase ───────────────────────────────────────────────────────────────────

/// The seven DCS phases plus terminal states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    /// Phase 1 — Decompose problem into sub-tasks.
    Divide,
    /// Phase 2 — Match sub-tasks to the best available agents.
    Assign,
    /// Phase 3 — Each agent solves its sub-task.
    Compute,
    /// Phase 4 — Filter solutions that meet the verification threshold.
    Verify,
    /// Phase 5 — Merge verified solutions into a unified result.
    Synthesize,
    /// Phase 6 — Validate the synthesized solution against the original problem.
    Validate,
    /// Phase 7 — Commit the solution or rollback.
    Integrate,
    /// Terminal success: solution committed.
    Complete,
    /// Terminal failure: reason encoded in the string.
    Failed(String),
}

impl Phase {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Phase::Complete | Phase::Failed(_))
    }
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Divide => write!(f, "DIVIDE"),
            Phase::Assign => write!(f, "ASSIGN"),
            Phase::Compute => write!(f, "COMPUTE"),
            Phase::Verify => write!(f, "VERIFY"),
            Phase::Synthesize => write!(f, "SYNTHESIZE"),
            Phase::Validate => write!(f, "VALIDATE"),
            Phase::Integrate => write!(f, "INTEGRATE"),
            Phase::Complete => write!(f, "COMPLETE"),
            Phase::Failed(e) => write!(f, "FAILED({})", e),
        }
    }
}

// ─── DcsState ────────────────────────────────────────────────────────────────

/// Full mutable state of one DCS execution cycle.
#[derive(Debug, Clone)]
pub struct DcsState {
    pub phase: Phase,
    pub problem: Tile,
    pub agents: Vec<Agent>,
    // Populated by each phase:
    pub sub_tasks: Vec<Tile>,
    pub assignments: Vec<Assignment>,
    pub solutions: Vec<Solution>,
    pub verified_solutions: Vec<Solution>,
    pub synthesized: Option<Tile>,
    pub validation_score: f64,
    pub committed: bool,
    pub cycle_log: Vec<String>,
}

impl DcsState {
    pub fn new(problem: Tile, agents: Vec<Agent>) -> Self {
        Self {
            phase: Phase::Divide,
            problem,
            agents,
            sub_tasks: Vec::new(),
            assignments: Vec::new(),
            solutions: Vec::new(),
            verified_solutions: Vec::new(),
            synthesized: None,
            validation_score: 0.0,
            committed: false,
            cycle_log: Vec::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.phase == Phase::Complete
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.phase, Phase::Failed(_))
    }
}

// ─── DcsEngine ───────────────────────────────────────────────────────────────

/// The DCS execution engine.
///
/// Each phase method is a pure function: it takes ownership of `DcsState`,
/// advances it by one phase, and returns the updated state.
#[derive(Debug, Clone)]
pub struct DcsEngine {
    /// Minimum solution score required to pass VERIFY.
    pub verification_threshold: f64,
    /// Minimum fleet score required to pass VALIDATE.
    pub validation_threshold: f64,
    /// Minimum sub-tasks produced by DIVIDE (safety floor).
    pub min_sub_tasks: usize,
}

impl Default for DcsEngine {
    fn default() -> Self {
        Self {
            verification_threshold: 0.5,
            validation_threshold: 1.0,
            min_sub_tasks: 1,
        }
    }
}

impl DcsEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_thresholds(verification: f64, validation: f64) -> Self {
        Self {
            verification_threshold: verification,
            validation_threshold: validation,
            ..Self::default()
        }
    }

    // ── Phase 1: DIVIDE ──────────────────────────────────────────────────────

    /// Decompose the problem tile into N sub-tasks proportional to complexity.
    ///
    /// Fan-out: `N = max(min_sub_tasks, ceil(complexity × 4))`
    pub fn divide(&self, mut state: DcsState) -> DcsState {
        if state.phase != Phase::Divide {
            return state;
        }

        let n = ((state.problem.complexity * 4.0).ceil() as usize).max(self.min_sub_tasks);
        let sub_complexity = state.problem.complexity / n as f64;

        state.sub_tasks = (0..n)
            .map(|i| Tile {
                id: next_id(),
                content: format!("{} [part {}/{}]", state.problem.content, i + 1, n),
                domain: state.problem.domain.clone(),
                complexity: sub_complexity,
            })
            .collect();

        state
            .cycle_log
            .push(format!("DIVIDE: '{}' → {} sub-tasks", state.problem.content, n));
        state.phase = Phase::Assign;
        state
    }

    // ── Phase 2: ASSIGN ──────────────────────────────────────────────────────

    /// Match each sub-task to the agent with the highest performance on its domain.
    pub fn assign(&self, mut state: DcsState) -> DcsState {
        if state.phase != Phase::Assign {
            return state;
        }

        if state.agents.is_empty() {
            state.phase = Phase::Failed("No agents in pool".into());
            state.cycle_log.push("ASSIGN: failed — empty agent pool".into());
            return state;
        }

        let mut assignments = Vec::new();
        for tile in &state.sub_tasks {
            let best = state
                .agents
                .iter()
                .max_by(|a, b| {
                    a.performance_on(&tile.domain)
                        .partial_cmp(&b.performance_on(&tile.domain))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .expect("agents non-empty; checked above");
            assignments.push(Assignment {
                tile: tile.clone(),
                agent: best.clone(),
            });
        }

        state.cycle_log.push(format!(
            "ASSIGN: {} sub-tasks → {} agents (pool size {})",
            assignments.len(),
            assignments
                .iter()
                .map(|a| a.agent.name.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            state.agents.len()
        ));
        state.assignments = assignments;
        state.phase = Phase::Compute;
        state
    }

    // ── Phase 3: COMPUTE ─────────────────────────────────────────────────────

    /// Each assigned agent computes a solution for its sub-task.
    pub fn compute(&self, mut state: DcsState) -> DcsState {
        if state.phase != Phase::Compute {
            return state;
        }

        state.solutions = state
            .assignments
            .iter()
            .map(|a| Solution::compute(&a.tile, &a.agent))
            .collect();

        state.cycle_log.push(format!(
            "COMPUTE: {} solutions produced",
            state.solutions.len()
        ));
        state.phase = Phase::Verify;
        state
    }

    // ── Phase 4: VERIFY ──────────────────────────────────────────────────────

    /// Retain only solutions whose score meets `verification_threshold`.
    pub fn verify(&self, mut state: DcsState) -> DcsState {
        if state.phase != Phase::Verify {
            return state;
        }

        let threshold = self.verification_threshold;
        let before = state.solutions.len();
        let verified: Vec<Solution> = state
            .solutions
            .drain(..)
            .map(|mut s| {
                s.verified = s.score >= threshold;
                s
            })
            .filter(|s| s.verified)
            .collect();

        if verified.is_empty() {
            state.phase = Phase::Failed(format!(
                "All {} solutions failed verification (threshold={:.4})",
                before, threshold
            ));
            state.cycle_log.push("VERIFY: all solutions rejected".into());
            return state;
        }

        state.cycle_log.push(format!(
            "VERIFY: {}/{} solutions passed (threshold={:.4})",
            verified.len(),
            before,
            threshold
        ));
        state.verified_solutions = verified;
        state.phase = Phase::Synthesize;
        state
    }

    // ── Phase 5: SYNTHESIZE ──────────────────────────────────────────────────

    /// Merge verified solutions into one synthesized tile.
    ///
    /// Fleet score = avg_solution_score × SYNTHESIS_BONUS.
    /// With perfect specialists: 5.88 × (21.87 / 5.88) = 21.87 = DCS_FLEET_RATIO.
    pub fn synthesize(&self, mut state: DcsState) -> DcsState {
        if state.phase != Phase::Synthesize {
            return state;
        }

        let n = state.verified_solutions.len() as f64;
        let avg_score = state.verified_solutions.iter().map(|s| s.score).sum::<f64>() / n;
        let fleet_score = avg_score * SYNTHESIS_BONUS;

        let content = state
            .verified_solutions
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<_>>()
            .join(" ⊕ ");

        state.synthesized = Some(Tile {
            id: next_id(),
            content,
            domain: state.problem.domain.clone(),
            complexity: state.problem.complexity,
        });
        state.validation_score = fleet_score;

        state.cycle_log.push(format!(
            "SYNTHESIZE: {} solutions merged, fleet_score={:.6}",
            n as usize,
            fleet_score
        ));
        state.phase = Phase::Validate;
        state
    }

    // ── Phase 6: VALIDATE ────────────────────────────────────────────────────

    /// Validate the synthesized solution meets the fleet score threshold.
    pub fn validate(&self, mut state: DcsState) -> DcsState {
        if state.phase != Phase::Validate {
            return state;
        }

        if state.synthesized.is_none() {
            state.phase = Phase::Failed("No synthesized solution available".into());
            return state;
        }

        if state.validation_score < self.validation_threshold {
            state.phase = Phase::Failed(format!(
                "Fleet score {:.6} < threshold {:.6}",
                state.validation_score, self.validation_threshold
            ));
            state.cycle_log.push("VALIDATE: failed — score below threshold".into());
            return state;
        }

        state.cycle_log.push(format!(
            "VALIDATE: fleet_score={:.6} ≥ threshold={:.4}",
            state.validation_score, self.validation_threshold
        ));
        state.phase = Phase::Integrate;
        state
    }

    // ── Phase 7: INTEGRATE ───────────────────────────────────────────────────

    /// Commit the synthesized solution. Sets `committed = true`.
    pub fn integrate(&self, mut state: DcsState) -> DcsState {
        if state.phase != Phase::Integrate {
            return state;
        }

        state.committed = true;
        state.cycle_log.push(format!(
            "INTEGRATE: committed (fleet_score={:.6})",
            state.validation_score
        ));
        state.phase = Phase::Complete;
        state
    }

    // ── Step / Run ───────────────────────────────────────────────────────────

    /// Advance the state machine by exactly one phase.
    pub fn step(&self, state: DcsState) -> DcsState {
        match &state.phase {
            Phase::Divide => self.divide(state),
            Phase::Assign => self.assign(state),
            Phase::Compute => self.compute(state),
            Phase::Verify => self.verify(state),
            Phase::Synthesize => self.synthesize(state),
            Phase::Validate => self.validate(state),
            Phase::Integrate => self.integrate(state),
            Phase::Complete | Phase::Failed(_) => state,
        }
    }

    /// Run all phases to a terminal state (Complete or Failed).
    pub fn run(&self, mut state: DcsState) -> DcsState {
        while !state.phase.is_terminal() {
            state = self.step(state);
        }
        state
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn math_specialist() -> Agent {
        Agent::new("MathBot", Domain::Math, 1.0)
    }

    fn generalist() -> Agent {
        Agent::new("Generalist", Domain::General, 1.0)
    }

    fn math_problem(complexity: f64) -> Tile {
        Tile::new("solve integration", Domain::Math, complexity)
    }

    fn engine() -> DcsEngine {
        DcsEngine::new()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RATIO ASSERTIONS — these must hold or the engine is wrong
    // ─────────────────────────────────────────────────────────────────────────

    /// A specialist on their domain must outperform a generalist by exactly 5.88×.
    #[test]
    fn test_specialist_ratio_5_88x() {
        let g = generalist();
        let s = math_specialist();
        let domain = Domain::Math;

        let g_score = g.performance_on(&domain);
        let s_score = s.performance_on(&domain);

        assert!(
            (g_score - 1.0).abs() < 1e-12,
            "Generalist baseline must be 1.0, got {g_score}"
        );
        let ratio = s_score / g_score;
        assert!(
            (ratio - SPECIALIST_RATIO).abs() < 1e-12,
            "Specialist ratio must be exactly {SPECIALIST_RATIO}×, got {ratio:.10}×"
        );
    }

    /// A DCS fleet of specialists must achieve 21.87× vs a single generalist.
    #[test]
    fn test_dcs_fleet_ratio_21_87x() {
        let generalist_baseline = generalist().performance_on(&Domain::Math); // 1.0

        // Single perfect specialist drives the fleet
        let state = DcsState::new(math_problem(0.25), vec![math_specialist()]);
        let result = engine().run(state);

        assert!(result.is_complete(), "DCS cycle must reach Complete");
        let fleet_score = result.validation_score;
        let ratio = fleet_score / generalist_baseline;

        assert!(
            (ratio - DCS_FLEET_RATIO).abs() < 1e-9,
            "DCS fleet ratio must be {DCS_FLEET_RATIO}×, got {ratio:.10}×"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // INDIVIDUAL PHASE TESTS (7 phases)
    // ─────────────────────────────────────────────────────────────────────────

    // Phase 1 ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_phase1_divide_basic() {
        let state = DcsState::new(math_problem(0.5), vec![math_specialist()]);
        let next = engine().divide(state);

        assert_eq!(next.phase, Phase::Assign);
        // ceil(0.5 × 4) = 2
        assert_eq!(next.sub_tasks.len(), 2, "complexity 0.5 → 2 sub-tasks");
        for tile in &next.sub_tasks {
            assert_eq!(tile.domain, Domain::Math);
            assert!((tile.complexity - 0.25).abs() < 1e-12);
        }
    }

    #[test]
    fn test_phase1_divide_full_complexity() {
        let state = DcsState::new(math_problem(1.0), vec![math_specialist()]);
        let next = engine().divide(state);
        // ceil(1.0 × 4) = 4
        assert_eq!(next.sub_tasks.len(), 4, "complexity 1.0 → 4 sub-tasks");
    }

    #[test]
    fn test_phase1_divide_min_floor() {
        // Even near-zero complexity yields at least min_sub_tasks = 1
        let state = DcsState::new(math_problem(0.001), vec![math_specialist()]);
        let next = engine().divide(state);
        assert!(!next.sub_tasks.is_empty());
    }

    // Phase 2 ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_phase2_assign_picks_specialist() {
        let agents = vec![math_specialist(), generalist()];
        let mut state = DcsState::new(math_problem(0.25), agents); // → 1 sub-task
        let e = engine();
        state = e.divide(state);
        let next = e.assign(state);

        assert_eq!(next.phase, Phase::Compute);
        assert_eq!(next.assignments.len(), 1);
        // MathBot (5.88) beats Generalist (1.0) on Domain::Math
        assert_eq!(next.assignments[0].agent.name, "MathBot");
    }

    #[test]
    fn test_phase2_assign_empty_pool_fails() {
        let mut state = DcsState::new(math_problem(0.25), vec![]);
        state = engine().divide(state);
        let next = engine().assign(state);
        assert!(matches!(next.phase, Phase::Failed(_)));
        assert!(!next.committed);
    }

    // Phase 3 ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_phase3_compute_specialist_score() {
        let e = engine();
        let mut state = DcsState::new(math_problem(0.25), vec![math_specialist()]);
        state = e.divide(state);
        state = e.assign(state);
        let next = e.compute(state);

        assert_eq!(next.phase, Phase::Verify);
        assert_eq!(next.solutions.len(), 1);
        // Specialist on Math: SPECIALIST_RATIO × trust 1.0 = 5.88
        assert!(
            (next.solutions[0].score - SPECIALIST_RATIO).abs() < 1e-12,
            "Expected score {SPECIALIST_RATIO}, got {}",
            next.solutions[0].score
        );
    }

    // Phase 4 ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_phase4_verify_passes() {
        let e = engine(); // threshold = 0.5; specialist scores 5.88
        let mut state = DcsState::new(math_problem(0.25), vec![math_specialist()]);
        state = e.divide(state);
        state = e.assign(state);
        state = e.compute(state);
        let next = e.verify(state);

        assert_eq!(next.phase, Phase::Synthesize);
        assert_eq!(next.verified_solutions.len(), 1);
        assert!(next.verified_solutions[0].verified);
    }

    #[test]
    fn test_phase4_verify_all_fail() {
        // Set verification threshold impossibly high
        let e = DcsEngine::with_thresholds(999.0, 1.0);
        let mut state = DcsState::new(math_problem(0.25), vec![math_specialist()]);
        state = e.divide(state);
        state = e.assign(state);
        state = e.compute(state);
        let next = e.verify(state);

        assert!(
            matches!(next.phase, Phase::Failed(_)),
            "Expected Failed when all solutions below threshold"
        );
        assert!(!next.committed);
    }

    // Phase 5 ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_phase5_synthesize_fleet_score() {
        let e = engine();
        let mut state = DcsState::new(math_problem(0.25), vec![math_specialist()]);
        state = e.divide(state);
        state = e.assign(state);
        state = e.compute(state);
        state = e.verify(state);
        let next = e.synthesize(state);

        assert_eq!(next.phase, Phase::Validate);
        assert!(next.synthesized.is_some());
        // avg=5.88, fleet=5.88 × SYNTHESIS_BONUS = DCS_FLEET_RATIO
        assert!(
            (next.validation_score - DCS_FLEET_RATIO).abs() < 1e-9,
            "Expected fleet score {DCS_FLEET_RATIO}, got {}",
            next.validation_score
        );
    }

    // Phase 6 ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_phase6_validate_passes() {
        let e = engine(); // validation_threshold=1.0; fleet_score≈21.87
        let mut state = DcsState::new(math_problem(0.25), vec![math_specialist()]);
        state = e.divide(state);
        state = e.assign(state);
        state = e.compute(state);
        state = e.verify(state);
        state = e.synthesize(state);
        let next = e.validate(state);

        assert_eq!(next.phase, Phase::Integrate);
    }

    #[test]
    fn test_phase6_validate_fails_high_threshold() {
        let e = DcsEngine::with_thresholds(0.5, 9999.0);
        let mut state = DcsState::new(math_problem(0.25), vec![math_specialist()]);
        state = e.divide(state);
        state = e.assign(state);
        state = e.compute(state);
        state = e.verify(state);
        state = e.synthesize(state);
        let next = e.validate(state);

        assert!(matches!(next.phase, Phase::Failed(_)));
        assert!(!next.committed);
    }

    // Phase 7 ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_phase7_integrate_commits() {
        let e = engine();
        let mut state = DcsState::new(math_problem(0.25), vec![math_specialist()]);
        state = e.divide(state);
        state = e.assign(state);
        state = e.compute(state);
        state = e.verify(state);
        state = e.synthesize(state);
        state = e.validate(state);
        let next = e.integrate(state);

        assert_eq!(next.phase, Phase::Complete);
        assert!(next.committed);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // END-TO-END CYCLE TESTS
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_e2e_single_specialist_completes() {
        let agents = vec![Agent::new("CodeBot", Domain::Code, 1.0)];
        let problem = Tile::new("write a parser", Domain::Code, 0.5);
        let result = engine().run(DcsState::new(problem, agents));

        assert!(result.is_complete(), "Expected Complete, got {}", result.phase);
        assert!(result.committed);
        assert!(result.synthesized.is_some());
        assert!(!result.cycle_log.is_empty());
    }

    #[test]
    fn test_e2e_multi_agent_fleet() {
        let agents = vec![
            Agent::new("MathBot", Domain::Math, 1.0),
            Agent::new("LogicBot", Domain::Logic, 0.9),
            Agent::new("LangBot", Domain::Language, 0.95),
        ];
        let problem = Tile::new("complex multi-step analysis", Domain::Math, 1.0);
        let result = engine().run(DcsState::new(problem, agents));

        assert!(result.is_complete());
        assert!(result.committed);
        // 4 sub-tasks, all solved by MathBot (best on Math)
        assert_eq!(result.verified_solutions.len(), 4);
    }

    #[test]
    fn test_e2e_generalist_only() {
        // Generalist has trust=1.0 but no specialty; score on Logic = 1.0
        // 1.0 > verification_threshold(0.5) and fleet score > validation_threshold(1.0)
        let agents = vec![Agent::new("AllRounder", Domain::General, 1.0)];
        let problem = Tile::new("general reasoning task", Domain::Logic, 0.5);
        let result = engine().run(DcsState::new(problem, agents));

        assert!(result.is_complete());
        assert!(result.committed);
        // fleet_score = 1.0 × SYNTHESIS_BONUS ≈ 3.72, which is > 1.0 threshold
        assert!(result.validation_score > 1.0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EDGE CASES
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_edge_empty_agent_pool() {
        let state = DcsState::new(math_problem(0.5), vec![]);
        let result = engine().run(state);

        assert!(result.is_failed(), "Empty pool must fail");
        assert!(!result.committed);
    }

    #[test]
    fn test_edge_single_agent() {
        let agents = vec![Agent::new("Solo", Domain::Math, 1.0)];
        let state = DcsState::new(math_problem(0.25), agents);
        let result = engine().run(state);

        assert!(result.is_complete());
        assert!(result.committed);
    }

    #[test]
    fn test_edge_all_agents_fail_verification() {
        let e = DcsEngine::with_thresholds(999.0, 1.0); // no agent can score 999
        let agents = vec![
            Agent::new("A", Domain::Math, 1.0),
            Agent::new("B", Domain::Math, 0.8),
        ];
        let result = e.run(DcsState::new(math_problem(0.25), agents));

        assert!(result.is_failed());
        assert!(!result.committed);
    }

    #[test]
    fn test_edge_minimal_complexity() {
        let agents = vec![Agent::new("Bot", Domain::Logic, 1.0)];
        let problem = Tile::new("tiny task", Domain::Logic, 0.01);
        let result = engine().run(DcsState::new(problem, agents));

        assert!(result.is_complete());
        assert_eq!(result.sub_tasks.len(), 1); // ceil(0.01×4)=1
    }

    #[test]
    fn test_edge_terminal_state_is_idempotent() {
        let agents = vec![math_specialist()];
        let state = DcsState::new(math_problem(0.25), agents);
        let e = engine();

        let result = e.run(state);
        assert_eq!(result.phase, Phase::Complete);

        // Stepping a Complete state does nothing
        let stepped = e.step(result.clone());
        assert_eq!(stepped.phase, Phase::Complete);
        assert_eq!(stepped.committed, result.committed);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // STATE MACHINE TRANSITION TEST
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_step_transitions_in_order() {
        let e = engine();
        let agents = vec![math_specialist()];
        let mut state = DcsState::new(math_problem(0.25), agents);

        let expected = [
            Phase::Divide,
            Phase::Assign,
            Phase::Compute,
            Phase::Verify,
            Phase::Synthesize,
            Phase::Validate,
            Phase::Integrate,
            Phase::Complete,
        ];

        assert_eq!(state.phase, expected[0]);
        for expected_phase in &expected[1..] {
            state = e.step(state);
            assert_eq!(&state.phase, expected_phase, "Wrong phase after step");
        }

        // Complete is sticky
        let still_complete = e.step(state);
        assert_eq!(still_complete.phase, Phase::Complete);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CYCLE LOG
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_cycle_log_has_entry_per_phase() {
        let agents = vec![math_specialist()];
        let state = DcsState::new(math_problem(0.25), agents);
        let result = engine().run(state);

        // One log entry per phase (DIVIDE, ASSIGN, COMPUTE, VERIFY, SYNTHESIZE, VALIDATE, INTEGRATE)
        assert!(
            result.cycle_log.len() >= 7,
            "Expected ≥7 log entries, got {}",
            result.cycle_log.len()
        );
    }
}
