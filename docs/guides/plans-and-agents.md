# Plans, Agents, and Models

> What you're paying for, what runs on your pod, and what models are available.

## Plans

Every Tytus plan comes with a **unit budget** — a fixed number of units you can allocate across pods.

| Plan | Price | Units | What you can run |
|---|---|---|---|
| Explorer | $39/mo | 1 unit | 1 OpenClaw or Hermes pod |
| Creator | $79/mo | 2 units | 2 one-unit pods, or 1 Cortex pod after activation |
| Operator | $149/mo | 4 units | Any mix up to 4 units |

Check your current plan and usage:
```bash
tytus status
```

---

## Agents

An **agent** is the AI runtime that runs inside your pod. You choose your agent when you connect:

### OpenClaw (1 unit) — Default

```bash
tytus connect --agent openclaw
```

OpenClaw runtime. Lightweight, fast startup. Best for:
- General AI chat and coding assistance
- Quick tasks and one-off queries
- When you want maximum pods per plan

### Hermes (1 unit)

```bash
tytus connect --agent hermes
```

Nous Research Hermes agent. More capable, heavier runtime. Best for:
- Complex multi-step reasoning
- Agentic workflows
- When quality matters more than quantity

### Cortex (2 units, gated preview)

```bash
tytus connect --agent cortex
```

Cortex is an API-first cited knowledge engine. It uses the pod-local
switchAILocal aliases `ail-compound` and `ail-embed`, has no browser UI, and
requires Creator or higher. The CLI/provider/infra path can be shipped before
customer production activation; until that separate gate opens, allocation can
still be refused by the service.

Use `tytus configure` for health and restart actions. Use the Cortex API or
`tytus exec` for operator work; `tytus ui` deliberately refuses Cortex.

### Switching Agents

You can't change the agent on a running pod. To switch:

```bash
tytus revoke <pod_id>     # Free the units (DESTRUCTIVE)
tytus connect --agent hermes  # Allocate with new agent
```

Your stable URL and API key remain the same after the switch.

---

## Models

Your pod gateway exposes AIL aliases via the OpenAI-compatible API. These aliases are configured globally by the selected AIL route. Apps should discover or receive the model list from the gateway/host, not hardcode provider model names.

Common aliases:

| Model ID | Capabilities | Use for |
|---|---|---|
| `ail-compound` | Text, vision, audio through the configured AIL route | Coding, chat, analysis |
| `ail-image` | Image generation through the configured AIL route | Creating images from text |
| `ail-embed` | Embeddings through the configured AIL route | Vector search, RAG applications |

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

### Model source of truth

Use the aliases returned by the gateway/model list or global AIL configuration. If a tool asks for a default chat model, `ail-compound` is the normal alias. Do not document a provider model as permanent unless it comes from current AIL configuration.

---

## Managing Your Pods

```bash
# See what's running
tytus status

# Allocate a new pod
tytus connect --agent openclaw

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
| 1 unit (Explorer) | 1 OpenClaw or Hermes pod; Cortex is refused |
| 2 units (Creator) | 2 one-unit pods, OR 1 Cortex pod after activation |
| 3 units | 3 one-unit pods, OR 1 Cortex pod + 1 one-unit pod |
| 4 units (Operator) | 4 one-unit pods, OR 2 Cortex pods, OR 1 Cortex pod + 2 one-unit pods |

If you try to allocate more than your budget allows:
```
403 plan_limit_reached: Current: 2/2 units used
```

Free a pod to make room: `tytus revoke <pod_id>`.
