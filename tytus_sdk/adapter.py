"""Core adapter protocol shared by every Tytus agent (OpenClaw, Hermes, …)."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Iterator, List, Literal, Optional, Protocol, runtime_checkable

Role = Literal["system", "user", "assistant"]
NotifyType = Literal["info", "warn", "decision", "ask"]


@dataclass
class AgentIdentity:
    """Stable identity the registry and consumers use to address an adapter."""

    pod_id: str
    agent_type: str
    display_name: str
    capabilities: list[str] = field(default_factory=list)
    model: str | None = None
    stable_endpoint: str | None = None


@dataclass
class AgentMessage:
    role: Role
    content: str

    def to_openai(self) -> dict:
        return {"role": self.role, "content": self.content}


@runtime_checkable
class AgentAdapter(Protocol):
    """Protocol every adapter implements. Lope, Harvey, etc. depend on *this*,
    not on the concrete OpenClaw/Hermes classes."""

    def identify(self) -> AgentIdentity: ...

    def ask(self, messages: list[AgentMessage], timeout_s: int = 120) -> str: ...

    def stream(
        self, messages: list[AgentMessage], timeout_s: int = 300
    ) -> Iterator[str]:
        """Default impl: call ask() and yield the whole reply once.
        Concrete adapters override for real token-level streaming."""
        yield self.ask(messages, timeout_s=timeout_s)

    def notify(
        self,
        message: str,
        typ: NotifyType = "info",
        details: dict | None = None,
    ) -> None:
        """Reverse channel: agent → human. Default is a no-op; the HarveyBridge
        client overrides this once Phase 4 lands."""
        return None

    def close(self) -> None: ...
