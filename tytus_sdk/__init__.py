"""Tytus Agent SDK — reusable adapters that let any caller (lope, harvey, cursor)
drive Tytus-hosted agents (OpenClaw, Hermes, …) as first-class teammates.

See docs/DESIGN-TYTUS-LOPE-TEAMMATES.md for architecture.
"""

__version__ = "0.1.0"

from tytus_sdk.adapter import (
    AgentAdapter,
    AgentIdentity,
    AgentMessage,
)

__all__ = ["AgentAdapter", "AgentIdentity", "AgentMessage", "__version__"]
