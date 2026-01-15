"""
AgentKern SDK for Python

Native Ed25519 cryptography and Liability Proof creation.

Example:
    >>> from agentkern import Agent
    >>> 
    >>> # Generate a new agent
    >>> agent = Agent.generate("my-agent")
    >>> 
    >>> # Create a liability proof
    >>> proof = agent.create_proof("payment:transfer:100")
    >>> 
    >>> # Verify a proof
    >>> is_valid = Agent.verify_proof(proof)
"""

from agentkern._native import (
    # Gate / Prompt Guard
    PromptGuard,
    GateEngine,
    ThreatLevel,
    PromptAction,
    PromptAnalysis,
)

__version__ = "0.1.0"

__all__ = [
    "__version__",
    "PromptGuard",
    "GateEngine",
    "ThreatLevel",
    "PromptAction",
    "PromptAnalysis",
]
