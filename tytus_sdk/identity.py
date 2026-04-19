"""Ed25519 device identity for the OpenClaw WS handshake.

The adapter generates one keypair once, stored at ~/.tytus/openclaw/device.json
(0600), and reuses it across every pod in the account. Per-pod keys would bloat
OpenClaw's `known_devices` list without buying us any real isolation since
every pod belongs to the same human anyway.
"""

from __future__ import annotations

import base64
import hashlib
import json
import os
import secrets
import stat
from dataclasses import dataclass
from pathlib import Path

from cryptography.hazmat.primitives.asymmetric.ed25519 import (
    Ed25519PrivateKey,
    Ed25519PublicKey,
)
from cryptography.hazmat.primitives.serialization import (
    Encoding,
    NoEncryption,
    PrivateFormat,
    PublicFormat,
)

IDENTITY_FORMAT_VERSION = 1


def _b64url(raw: bytes) -> str:
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii")


def _b64url_decode(s: str) -> bytes:
    s = s + "=" * (-len(s) % 4)
    return base64.urlsafe_b64decode(s.encode("ascii"))


@dataclass
class DeviceIdentity:
    device_id: str
    private_key: Ed25519PrivateKey
    public_key: Ed25519PublicKey

    @property
    def public_key_b64url(self) -> str:
        raw = self.public_key.public_bytes(Encoding.Raw, PublicFormat.Raw)
        return _b64url(raw)

    def sign_b64url(self, payload: str | bytes) -> str:
        if isinstance(payload, str):
            payload = payload.encode("utf-8")
        return _b64url(self.private_key.sign(payload))


def default_identity_path() -> Path:
    return Path.home() / ".tytus" / "openclaw" / "device.json"


def load_or_create_identity(path: Path | None = None) -> DeviceIdentity:
    """Load an existing identity or mint a new one. Idempotent."""
    path = path or default_identity_path()
    if path.exists():
        return _load(path)
    return _create(path)


def _load(path: Path) -> DeviceIdentity:
    data = json.loads(path.read_text())
    if data.get("version") != IDENTITY_FORMAT_VERSION:
        raise ValueError(
            f"Unsupported tytus device identity format version: {data.get('version')}"
        )
    seed = _b64url_decode(data["private_key_seed"])
    priv = Ed25519PrivateKey.from_private_bytes(seed)
    pub = priv.public_key()
    return DeviceIdentity(device_id=data["device_id"], private_key=priv, public_key=pub)


def _create(path: Path) -> DeviceIdentity:
    priv = Ed25519PrivateKey.generate()
    pub = priv.public_key()
    seed = priv.private_bytes(Encoding.Raw, PrivateFormat.Raw, NoEncryption())
    # device_id = sha256(pub_raw).hex() — OpenClaw's server-side derivation
    # (src/infra/device-identity.ts:146 `deriveDeviceIdFromPublicKey`). Must
    # be the FULL 64-char hex or the gateway rejects with DEVICE_ID_MISMATCH.
    pub_raw = pub.public_bytes(Encoding.Raw, PublicFormat.Raw)
    device_id = hashlib.sha256(pub_raw).hexdigest()
    payload = {
        "version": IDENTITY_FORMAT_VERSION,
        "device_id": device_id,
        "private_key_seed": _b64url(seed),
        "public_key": _b64url(pub_raw),
        "created_at_ms": int(os.path.getmtime(path.parent) * 1000) if path.parent.exists() else None,
    }
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    path.write_text(json.dumps(payload, indent=2))
    os.chmod(path, stat.S_IRUSR | stat.S_IWUSR)  # 0600
    return DeviceIdentity(device_id=device_id, private_key=priv, public_key=pub)


def fresh_nonce() -> str:
    """Only used client-side when we want a nonce the server didn't give us
    (i.e., never for connect — the server always sends connect.challenge).
    Kept here for consistency."""
    return secrets.token_urlsafe(16)
