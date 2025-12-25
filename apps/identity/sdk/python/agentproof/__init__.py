"""
AgentProof Python SDK

Zero-config embedded verification for AI agents.

Installation:
    pip install agentproof

Usage:
    from agentproof import AgentProof
    
    proof = await AgentProof.create_proof(
        principal={"id": "user-123", "credential_id": "cred-456"},
        agent={"id": "my-agent", "name": "My Agent", "version": "1.0.0"},
        intent={
            "action": "transfer",
            "target": {"service": "api.bank.com", "endpoint": "/transfer", "method": "POST"}
        }
    )
    
    # Add to request
    import httpx
    response = await httpx.post(url, headers={"X-AgentProof": proof.header})
"""

import asyncio
import httpx
from typing import Optional, Dict, Any, List
from dataclasses import dataclass
from functools import wraps


@dataclass
class Principal:
    id: str
    credential_id: str
    device_attestation: Optional[str] = None


@dataclass
class Agent:
    id: str
    name: str
    version: str


@dataclass
class IntentTarget:
    service: str
    endpoint: str
    method: str


@dataclass
class Intent:
    action: str
    target: IntentTarget
    parameters: Optional[Dict[str, Any]] = None


@dataclass
class Constraints:
    max_amount: Optional[float] = None
    allowed_recipients: Optional[List[str]] = None
    geo_fence: Optional[List[str]] = None
    valid_hours: Optional[Dict[str, int]] = None
    require_confirmation_above: Optional[float] = None
    single_use: bool = False


@dataclass
class ProofResult:
    header: str
    proof_id: str
    expires_at: str


@dataclass
class VerifyResult:
    valid: bool
    proof_id: Optional[str] = None
    principal_id: Optional[str] = None
    agent_id: Optional[str] = None
    intent: Optional[Dict[str, str]] = None
    liability_accepted_by: Optional[str] = None
    errors: Optional[List[str]] = None


@dataclass 
class TrustResolution:
    trusted: bool
    trust_score: int
    ttl: int
    revoked: bool


class AgentProofClient:
    """AgentProof Python Client"""
    
    def __init__(
        self,
        server_url: str = "http://localhost:5002",
        timeout: float = 5.0,
    ):
        self.server_url = server_url
        self.timeout = timeout
        self._client = httpx.AsyncClient(timeout=timeout)
    
    async def create_proof(
        self,
        principal: Dict[str, str],
        agent: Dict[str, str],
        intent: Dict[str, Any],
        constraints: Optional[Dict[str, Any]] = None,
        expires_in_seconds: int = 300,
    ) -> ProofResult:
        """Create a signed Liability Proof"""
        response = await self._client.post(
            f"{self.server_url}/api/v1/proof/create",
            json={
                "principal": {
                    "id": principal["id"],
                    "credentialId": principal.get("credential_id", principal.get("credentialId")),
                },
                "agent": agent,
                "intent": intent,
                "constraints": constraints,
                "expiresInSeconds": expires_in_seconds,
            },
        )
        response.raise_for_status()
        data = response.json()
        return ProofResult(
            header=data["header"],
            proof_id=data["proofId"],
            expires_at=data["expiresAt"],
        )
    
    async def verify_proof(self, header: str) -> VerifyResult:
        """Verify a Liability Proof"""
        response = await self._client.post(
            f"{self.server_url}/api/v1/proof/verify",
            json={"proof": header},
        )
        response.raise_for_status()
        data = response.json()
        return VerifyResult(
            valid=data["valid"],
            proof_id=data.get("proofId"),
            principal_id=data.get("principalId"),
            agent_id=data.get("agentId"),
            intent=data.get("intent"),
            liability_accepted_by=data.get("liabilityAcceptedBy"),
            errors=data.get("errors"),
        )
    
    async def resolve_trust(self, agent_id: str, principal_id: str) -> TrustResolution:
        """Resolve trust for an agent-principal pair"""
        response = await self._client.get(
            f"{self.server_url}/api/v1/dns/resolve",
            params={"agentId": agent_id, "principalId": principal_id},
        )
        response.raise_for_status()
        data = response.json()
        return TrustResolution(
            trusted=data["trusted"],
            trust_score=data["trustScore"],
            ttl=data["ttl"],
            revoked=data["revoked"],
        )
    
    async def register_trust(
        self,
        agent_id: str,
        principal_id: str,
        agent_name: Optional[str] = None,
        agent_version: Optional[str] = None,
    ) -> None:
        """Register a trust relationship"""
        response = await self._client.post(
            f"{self.server_url}/api/v1/dns/register",
            json={
                "agentId": agent_id,
                "principalId": principal_id,
                "agentName": agent_name,
                "agentVersion": agent_version,
            },
        )
        response.raise_for_status()
    
    async def revoke_trust(self, agent_id: str, principal_id: str, reason: str) -> None:
        """Revoke trust"""
        response = await self._client.post(
            f"{self.server_url}/api/v1/dns/revoke",
            json={"agentId": agent_id, "principalId": principal_id, "reason": reason},
        )
        response.raise_for_status()
    
    async def close(self):
        """Close the HTTP client"""
        await self._client.aclose()


# Default singleton
_default_client: Optional[AgentProofClient] = None


def get_client(server_url: str = "http://localhost:5002") -> AgentProofClient:
    """Get or create the default client"""
    global _default_client
    if _default_client is None:
        _default_client = AgentProofClient(server_url=server_url)
    return _default_client


class AgentProof:
    """Static interface for AgentProof SDK"""
    
    @staticmethod
    async def create_proof(
        principal: Dict[str, str],
        agent: Dict[str, str],
        intent: Dict[str, Any],
        constraints: Optional[Dict[str, Any]] = None,
        expires_in_seconds: int = 300,
        server_url: str = "http://localhost:5002",
    ) -> ProofResult:
        """Create a signed Liability Proof"""
        client = get_client(server_url)
        return await client.create_proof(
            principal=principal,
            agent=agent,
            intent=intent,
            constraints=constraints,
            expires_in_seconds=expires_in_seconds,
        )
    
    @staticmethod
    async def verify_proof(header: str, server_url: str = "http://localhost:5002") -> VerifyResult:
        """Verify a Liability Proof"""
        client = get_client(server_url)
        return await client.verify_proof(header)
    
    @staticmethod
    async def resolve_trust(
        agent_id: str,
        principal_id: str,
        server_url: str = "http://localhost:5002",
    ) -> TrustResolution:
        """Resolve trust for an agent-principal pair"""
        client = get_client(server_url)
        return await client.resolve_trust(agent_id, principal_id)


def require_agentproof(principal: Dict[str, str], agent: Dict[str, str]):
    """Decorator to add AgentProof to async functions that make HTTP calls"""
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            # Create proof before execution
            proof = await AgentProof.create_proof(
                principal=principal,
                agent=agent,
                intent={
                    "action": func.__name__,
                    "target": {"service": "function", "endpoint": f"/{func.__name__}", "method": "POST"},
                },
            )
            
            # Add proof to kwargs for the function to use
            kwargs["_agentproof_header"] = proof.header
            
            return await func(*args, **kwargs)
        return wrapper
    return decorator


# CrewAI integration
class AgentProofCrewAITool:
    """Base class for CrewAI tools with AgentProof"""
    
    def __init__(
        self,
        principal: Dict[str, str],
        agent: Dict[str, str],
        server_url: str = "http://localhost:5002",
    ):
        self.principal = principal
        self.agent = agent
        self.client = AgentProofClient(server_url=server_url)
    
    async def get_proof_header(self, action: str, parameters: Dict[str, Any]) -> str:
        """Generate proof header for a tool action"""
        proof = await self.client.create_proof(
            principal=self.principal,
            agent=self.agent,
            intent={
                "action": action,
                "target": {"service": "crewai-tool", "endpoint": f"/{action}", "method": "POST"},
                "parameters": parameters,
            },
        )
        return proof.header


# AutoGPT integration  
class AgentProofAutoGPTPlugin:
    """AutoGPT plugin for AgentProof"""
    
    def __init__(
        self,
        principal: Dict[str, str],
        agent: Dict[str, str],
        server_url: str = "http://localhost:5002",
    ):
        self.principal = principal
        self.agent = agent
        self.client = AgentProofClient(server_url=server_url)
    
    async def wrap_command(self, command_name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        """Wrap an AutoGPT command with AgentProof"""
        proof = await self.client.create_proof(
            principal=self.principal,
            agent=self.agent,
            intent={
                "action": command_name,
                "target": {"service": "autogpt", "endpoint": f"/commands/{command_name}", "method": "POST"},
                "parameters": arguments,
            },
        )
        
        return {
            "original_arguments": arguments,
            "agentproof_header": proof.header,
            "proof_id": proof.proof_id,
        }
