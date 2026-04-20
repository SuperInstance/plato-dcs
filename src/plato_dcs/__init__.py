"""Dynamic Consensus System — multi-agent belief, lock accumulation, consensus rounds.
Part of the PLATO framework."""
from .dcs import BeliefScore, BeliefStore, Lock, LockAccumulator, ConsensusRound, DCS
__version__ = "0.1.0"
__all__ = ["BeliefScore", "BeliefStore", "Lock", "LockAccumulator", "ConsensusRound", "DCS"]
