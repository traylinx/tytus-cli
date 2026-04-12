# Plans, Agents, and Models

> What you're paying for, what runs on your pod, and what models are available.

## Plans

Every Tytus plan comes with a **unit budget** — a fixed number of units you can allocate across pods.

| Plan | Price | Units | What you can run |
|---|---|---|---|
| Explorer | $39/mo | 1 unit | 1 nemoclaw |
| Creator | $79/mo | 2 units | 2 nemoclaw, or 1 hermes |
| Operator | $149/mo | 4 units | Any mix up to 4 units |

Check your current plan and usage:
```bash
tytus status
```

---

## Agents

An **agent** is the AI runtime that runs inside your pod. You choose your agent when you connect:

### NemoClaw (1 unit) — Default

```bash
tytus connect --agent nemoclaw
```

OpenClaw runtime with the NemoClaw sandboxing blueprint. Lightweight, fast startup. Best for:
- General AI chat and coding assistance
- Quick tasks and one-off queries
- When you want maximum pods per plan

### Hermes (2 units)

```bash
tytus connect --agent hermes
```

Nous Research Hermes agent. More capable, heavier runtime. Best for:
- Complex multi-step reasoning
- Agentic workflows
- When quality matters more than quantity

### Switching Agents

You can't change the agent on a running pod. To switch:

```bash
tytus revoke <pod_id>     # Free the units (DESTRUCTIVE)
tytus connect --agent hermes  # Allocate with new agent
```

Your stable URL and API key remain the same after the switch.

---

## Models

Your pod gateway exposes these models via the OpenAI-compatible API:

| Model ID | Backed by | Capabilities | Use for |
|---|---|---|---|
| `ail-compound` | MiniMax M2.7 | Text, vision, audio | Coding, chat, analysis (default) |
| `ail-image` | MiniMax image-01 | Image generation | Creating images from text |
| `ail-embed` | mistral-embed | Embeddings | Vector search, RAG applications |

### Using a specific model

```bash
# In tytus chat
tytus chat --model ail-compound

# In curl
curl "$OPENAI_BASE_URL/chat/completions" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d '{"model":"ail-compound","messages":[{"role":"user","content":"hello"}]}'

# In Python
from openai import OpenAI
client = OpenAI(base_url="http://10.42.42.1:18080/v1", api_key="sk-tytus-user-...")
response = client.chat.completions.create(model="ail-compound", messages=[...])
```

### What models are NOT available

Your pod runs specific models from the SwitchAILocal gateway. Standard model IDs like `gpt-4`, `claude-3`, `llama-3` are **not available**. If a tool asks for a model, use `ail-compound`.

---

## Managing Your Pods

```bash
# See what's running
tytus status

# Allocate a new pod
tytus connect --agent nemoclaw

# Restart the agent (applies config changes)
tytus restart

# Free a pod (DESTRUCTIVE — wipes workspace)
tytus revoke <pod_id>

# Run a command inside the pod
tytus exec "ls /workspace"
```

---

## Unit Budget Math

| You have | You can run |
|---|---|
| 1 unit (Explorer) | 1 nemoclaw |
| 2 units (Creator) | 2 nemoclaw, OR 1 hermes |
| 3 units | 3 nemoclaw, OR 1 hermes + 1 nemoclaw |
| 4 units (Operator) | 4 nemoclaw, OR 2 hermes, OR 2 nemoclaw + 1 hermes |

If you try to allocate more than your budget allows:
```
403 plan_limit_reached: Current: 2/2 units used
```

Free a pod to make room: `tytus revoke <pod_id>`.
