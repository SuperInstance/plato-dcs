"""Dynamic Consensus System for multi-agent coordination."""

import time
from dataclasses import dataclass, field
from typing import Optional

@dataclass
class BeliefScore:
    topic: str
    confidence: float
    trust: float = 0.5
    relevance: float = 0.5
    source: str = ""
    timestamp: float = field(default_factory=time.time)

class BeliefStore:
    def __init__(self, max_beliefs: int = 10000):
        self.max_beliefs = max_beliefs
        self._beliefs: dict[str, list[BeliefScore]] = {}

    def record(self, belief: BeliefScore):
        if belief.topic not in self._beliefs:
            self._beliefs[belief.topic] = []
        self._beliefs[belief.topic].append(belief)
        if len(self._beliefs[belief.topic]) > 100:
            self._beliefs[belief.topic] = self._beliefs[belief.topic][-100:]

    def get_belief(self, topic: str) -> Optional[BeliefScore]:
        records = self._beliefs.get(topic, [])
        return records[-1] if records else None

    def weighted_belief(self, topics: list[str]) -> float:
        scores = [self._get_topic_score(t) for t in topics if self._beliefs.get(t)]
        if not scores: return 0.0
        return sum(scores) / len(scores)

    def _get_topic_score(self, topic: str) -> float:
        records = self._beliefs.get(topic, [])
        if not records: return 0.0
        total, weight = 0.0, 0.0
        for r in records:
            w = 0.99 ** ((time.time() - r.timestamp) / 3600)
            total += (r.confidence * r.trust * r.relevance) * w
            weight += w
        return total / max(weight, 1e-9)

    def decay(self, rate: float = 0.96):
        for topic in self._beliefs:
            for b in self._beliefs[topic]:
                b.confidence = 0.5 + (b.confidence - 0.5) * rate
                b.trust = 0.5 + (b.trust - 0.5) * rate

    @property
    def stats(self) -> dict:
        return {"topics": len(self._beliefs),
                "total_records": sum(len(v) for v in self._beliefs.values())}

@dataclass
class Lock:
    trigger: str
    opcode: str
    constraint: str
    strength: float
    acquired_at: float = field(default_factory=time.time)

class LockAccumulator:
    def __init__(self, critical_mass_n: int = 7):
        self.critical_mass_n = critical_mass_n
        self._locks: list[Lock] = []

    def add_lock(self, trigger: str, opcode: str, constraint: str, strength: float = 0.5) -> Lock:
        lock = Lock(trigger=trigger, opcode=opcode, constraint=constraint, strength=min(strength, 1.0))
        self._locks.append(lock)
        return lock

    def total_strength(self) -> float:
        return sum(l.strength for l in self._locks)

    def check_critical_mass(self) -> bool:
        return len(self._locks) >= self.critical_mass_n and self.total_strength() >= 1.0

    def dominant_opcode(self) -> Optional[str]:
        if not self._locks: return None
        counts: dict[str, int] = {}
        for l in self._locks:
            counts[l.opcode] = counts.get(l.opcode, 0) + 1
        return max(counts, key=counts.get)

    def release(self, trigger: str) -> int:
        before = len(self._locks)
        self._locks = [l for l in self._locks if l.trigger != trigger]
        return before - len(self._locks)

    @property
    def stats(self) -> dict:
        opcodes = {}
        for l in self._locks:
            opcodes[l.opcode] = opcodes.get(l.opcode, 0) + 1
        return {"locks": len(self._locks), "total_strength": self.total_strength(),
                "critical_mass": self.check_critical_mass(), "opcodes": opcodes}

class ConsensusRound:
    def __init__(self, participants: int = 3, threshold: float = 0.7):
        self.participants = participants
        self.threshold = threshold
        self._submissions: dict[str, dict] = {}

    def submit(self, participant_id: str, belief: float, lock_count: int = 0):
        self._submissions[participant_id] = {"belief": belief, "lock_count": lock_count}

    def compute_consensus(self) -> Optional[float]:
        if not self.is_quorum(): return None
        total_belief, total_locks = 0.0, 0
        for s in self._submissions.values():
            w = 1 + s["lock_count"] * 0.1
            total_belief += s["belief"] * w
            total_locks += w
        return total_belief / total_locks

    def is_quorum(self) -> bool:
        return len(self._submissions) >= self.participants

    @property
    def stats(self) -> dict:
        return {"participants": len(self._submissions), "quorum": self.is_quorum(),
                "consensus": self.compute_consensus()}

class DCS:
    def __init__(self, critical_mass_n: int = 7, participants: int = 3):
        self.beliefs = BeliefStore()
        self.locks = LockAccumulator(critical_mass_n)
        self.consensus = ConsensusRound(participants)

    def process_event(self, topic: str, confidence: float, trust: float = 0.5,
                      relevance: float = 0.5, trigger: str = "", opcode: str = "",
                      constraint: str = "") -> dict:
        self.beliefs.record(BeliefScore(topic=topic, confidence=confidence,
                                         trust=trust, relevance=relevance))
        if trigger and opcode:
            self.locks.add_lock(trigger, opcode, constraint, strength=confidence)
        return {"belief_score": self.beliefs.weighted_belief([topic]),
                "lock_mass": self.locks.total_strength(),
                "critical_mass": self.locks.check_critical_mass(),
                "dominant_opcode": self.locks.dominant_opcode()}

    def health_score(self) -> float:
        belief_health = min(self.beliefs.stats["topics"] / 10, 1.0)
        lock_health = min(self.locks.total_strength() / 3.0, 1.0)
        consensus_health = 1.0 if self.consensus.is_quorum() else 0.3
        return (belief_health * 0.4 + lock_health * 0.3 + consensus_health * 0.3)

    @property
    def stats(self) -> dict:
        return {"beliefs": self.beliefs.stats, "locks": self.locks.stats,
                "consensus": self.consensus.stats, "health": self.health_score()}
