"""
Type stubs for AgentKern Python SDK.

Provides type hints for the native Rust extension module.
"""

from typing import Optional

__version__: str

def version() -> str:
    """Get the SDK version."""
    ...

class Agent:
    """
    Agent - The core identity unit in AgentKern.
    
    Holds an Ed25519 keypair and can:
    - Sign arbitrary data
    - Create Liability Proofs
    - Verify other agents' proofs
    """
    
    @property
    def id(self) -> str:
        """Agent's unique ID (DID format)."""
        ...
    
    @property
    def name(self) -> str:
        """Agent's name."""
        ...
    
    @property
    def public_key(self) -> str:
        """Agent's public key (base64url encoded)."""
        ...
    
    @property
    def seed(self) -> str:
        """Keypair seed for persistence (base64url encoded) - SENSITIVE."""
        ...
    
    @staticmethod
    def generate(name: str) -> "Agent":
        """Generate a new Agent with a random Ed25519 keypair."""
        ...
    
    @staticmethod
    def generate_with_config(
        name: str,
        proof_expiry_seconds: Optional[int] = None,
        issuer: Optional[str] = None,
    ) -> "Agent":
        """Generate a new Agent with custom configuration."""
        ...
    
    @staticmethod
    def from_seed(name: str, seed_base64: str) -> "Agent":
        """Restore an Agent from an existing keypair seed (base64url encoded)."""
        ...
    
    @staticmethod
    def verify_proof(proof: "LiabilityProof") -> bool:
        """Verify a Liability Proof."""
        ...
    
    def sign(self, data: bytes) -> str:
        """Sign arbitrary data bytes (returns base64url signature)."""
        ...
    
    def sign_message(self, message: str) -> str:
        """Sign a string message (returns base64url signature)."""
        ...
    
    def create_proof(self, action: str) -> "LiabilityProof":
        """Create a Liability Proof for an action."""
        ...


class LiabilityProof:
    """Liability Proof - A signed JWT proving authorization."""
    
    @property
    def issuer(self) -> str:
        """Issuer (DID)."""
        ...
    
    @property
    def subject(self) -> str:
        """Subject (agent DID)."""
        ...
    
    @property
    def action(self) -> str:
        """Authorized action."""
        ...
    
    @property
    def issued_at(self) -> int:
        """Issued at (Unix timestamp)."""
        ...
    
    @property
    def expires_at(self) -> int:
        """Expiration (Unix timestamp)."""
        ...
    
    @property
    def jti(self) -> str:
        """JWT ID."""
        ...
    
    @property
    def jwt(self) -> str:
        """Full raw JWT string."""
        ...
    
    def is_expired(self) -> bool:
        """Check if this proof is expired."""
        ...


def parse_proof(jwt: str) -> LiabilityProof:
    """Parse a JWT string into a LiabilityProof."""
    ...


def create_a2a_request(from_: str, to: str, payload: str) -> str:
    """Create an A2A request message (payload as JSON string)."""
    ...


def create_a2a_notification(from_: str, to: str, payload: str) -> str:
    """Create an A2A notification message (payload as JSON string)."""
    ...
