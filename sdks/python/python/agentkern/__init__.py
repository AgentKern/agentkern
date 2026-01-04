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
    # Version
    __version__,
    version,
    # Classes
    Agent,
    LiabilityProof,
    # Functions
    parse_proof,
    create_a2a_request,
    create_a2a_notification,
)

__all__ = [
    "__version__",
    "version",
    "Agent",
    "LiabilityProof",
    "parse_proof",
    "create_a2a_request",
    "create_a2a_notification",
]
