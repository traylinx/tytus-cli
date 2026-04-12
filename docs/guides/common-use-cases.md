# Common Use Cases

> Real-world scenarios with copy-paste commands.

---

## "I just want to code with AI"

```bash
tytus setup              # one-time: login + connect + test
tytus link .             # inject AI integration into your project
claude                   # start coding
```

Or use the tray icon: click **T** > **Open in** > **Claude Code**.

---

## "I want to use my pod from Python"

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://10.42.42.1:18080/v1",
    api_key="sk-tytus-user-..."  # run: tytus env
)

response = client.chat.completions.create(
    model="ail-compound",
    messages=[{"role": "user", "content": "Explain quantum computing in 3 sentences"}]
)
print(response.choices[0].message.content)
```

Get your API key:
```bash
tytus env
```

---

## "I want my tunnel to survive reboots"

```bash
tytus autostart install
```

Done. Your tunnel reconnects automatically every time you log in. Your tools keep working with the same URL and key.

Verify it's installed:
```bash
tytus autostart status
```

---

## "I want to switch from nemoclaw to hermes"

```bash
# See what's running
tytus status

# Free the current pod (DESTRUCTIVE)
tytus revoke 02

# Allocate with hermes
tytus connect --agent hermes

# Test it
tytus test
```

Your stable URL and API key stay the same. Tools configured with those values don't need updating.

---

## "I want every AI CLI on my machine to use my pod"

Set the env vars globally in your shell profile:

```bash
# Add to ~/.zshrc or ~/.bashrc
eval "$(tytus env --export)"
```

Now every new terminal has `OPENAI_API_KEY` and `OPENAI_BASE_URL` set. Any tool that reads these (Claude Code, OpenCode, Aider, Codex, Vibe) will route through your pod.

---

## "I want to run a command inside my pod"

```bash
# List files in the workspace
tytus exec "ls /workspace"

# Check what agent is running
tytus exec "cat /etc/agent-type"

# Install a package
tytus exec "pip install pandas"
```

Commands run inside the agent container with a 30-second default timeout (max 120s):
```bash
tytus exec --timeout 60 "pip install torch"
```

---

## "I want to generate an image"

```bash
eval "$(tytus env --export)"
curl -sS "$OPENAI_BASE_URL/images/generations" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"ail-image","prompt":"a lobster wearing a top hat, digital art","n":1}'
```

---

## "I want to use embeddings for RAG"

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://10.42.42.1:18080/v1",
    api_key="sk-tytus-user-..."
)

response = client.embeddings.create(
    model="ail-embed",
    input="What is the meaning of life?"
)
vector = response.data[0].embedding
print(f"Embedding dimension: {len(vector)}")
```

---

## "I want to diagnose why my connection is broken"

```bash
# Quick check
tytus status

# Full diagnostic (checks 8 things)
tytus doctor

# See the daemon log
cat /tmp/tytus/autostart.log

# See the tunnel daemon log
cat /tmp/tytus/tunnel-02.log

# Nuclear option: disconnect + reconnect
tytus disconnect && tytus connect && tytus test
```

---

## "I want to share my pod setup with a team member"

You can't share pods (each user gets their own key). But you can share the setup process:

```bash
# They run:
curl -fsSL https://get.traylinx.com/install.sh | bash
tytus setup
```

Each team member gets their own stable URL + key pair. The URL (`10.42.42.1:18080`) is the same for everyone, but the API key is per-user.

---

## "I want to use Tytus from a CI/CD pipeline"

Tytus is designed for interactive use, but headless mode works for CI:

```bash
# In CI, set TYTUS_HEADLESS=1 to prevent browser prompts
export TYTUS_HEADLESS=1

# Login must happen interactively first (on your machine)
# Then the refresh token persists and CI can use it:
tytus connect --headless
eval "$(tytus env --export)"
curl "$OPENAI_BASE_URL/chat/completions" ...
```

**Important:** The CI machine needs the same `state.json` file (or a pre-authenticated token). Tytus is not designed for headless-first CI — it's a developer tool.
