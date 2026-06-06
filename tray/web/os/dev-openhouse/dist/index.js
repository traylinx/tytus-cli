import { jsx as i, jsxs as l } from "react/jsx-runtime";
import { forwardRef as me, createElement as oe, useState as b, useEffect as z, useMemo as te, useCallback as P } from "react";
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Se = (o) => o.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase(), Te = (o) => o.replace(
  /^([A-Z])|[\s-_]+(\w)/g,
  (e, a, t) => t ? t.toUpperCase() : a.toLowerCase()
), de = (o) => {
  const e = Te(o);
  return e.charAt(0).toUpperCase() + e.slice(1);
}, ye = (...o) => o.filter((e, a, t) => !!e && e.trim() !== "" && t.indexOf(e) === a).join(" ").trim(), _e = (o) => {
  for (const e in o)
    if (e.startsWith("aria-") || e === "role" || e === "title")
      return !0;
};
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
var Ne = {
  xmlns: "http://www.w3.org/2000/svg",
  width: 24,
  height: 24,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 2,
  strokeLinecap: "round",
  strokeLinejoin: "round"
};
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Pe = me(
  ({
    color: o = "currentColor",
    size: e = 24,
    strokeWidth: a = 2,
    absoluteStrokeWidth: t,
    className: s = "",
    children: r,
    iconNode: c,
    ...d
  }, m) => oe(
    "svg",
    {
      ref: m,
      ...Ne,
      width: e,
      height: e,
      stroke: o,
      strokeWidth: t ? Number(a) * 24 / Number(e) : a,
      className: ye("lucide", s),
      ...!r && !_e(d) && { "aria-hidden": "true" },
      ...d
    },
    [
      ...c.map(([C, g]) => oe(C, g)),
      ...Array.isArray(r) ? r : [r]
    ]
  )
);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const ie = (o, e) => {
  const a = me(
    ({ className: t, ...s }, r) => oe(Pe, {
      ref: r,
      iconNode: e,
      className: ye(
        `lucide-${Se(de(o))}`,
        `lucide-${o}`,
        t
      ),
      ...s
    })
  );
  return a.displayName = de(o), a;
};
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const He = [
  ["path", { d: "M5 12h14", key: "1ays0h" }],
  ["path", { d: "M12 5v14", key: "s699le" }]
], ge = ie("plus", He);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Me = [
  ["path", { d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8", key: "v9h5vc" }],
  ["path", { d: "M21 3v5h-5", key: "1q7to0" }],
  ["path", { d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16", key: "3uifl3" }],
  ["path", { d: "M8 16H3v5", key: "1cv678" }]
], Oe = ie("refresh-cw", Me);
/**
 * @license lucide-react v0.562.0 - ISC
 *
 * This source code is licensed under the ISC license.
 * See the LICENSE file in the root directory of this source tree.
 */
const Ie = [
  ["path", { d: "M18 6 6 18", key: "1bl5f8" }],
  ["path", { d: "m6 6 12 12", key: "d8bk6v" }]
], Le = ie("x", Ie);
function F(o) {
  switch (o) {
    case "online":
      return "focused";
    case "busy":
      return "thinking";
    case "starting":
      return "idle";
    case "degraded":
      return "stressed";
    case "error":
      return "sick";
    case "offline":
      return "sleeping";
    default:
      return "idle";
  }
}
function W(o, e, a) {
  if (a === "error" || a === "offline") return "incident-infirmary";
  if (o === "tytus-daemon") return "tytus-lab";
  if (o === "ail-gateway") return "remote-balcony";
  if (o === "mcp-http") return "mcp-library";
  if (e)
    try {
      const t = new URL(e).hostname.toLowerCase();
      if (t === "localhost" || t === "127.0.0.1" || t === "::1") return "local-workshop";
    } catch {
    }
  return o === "custom-health" ? "lobby" : "remote-balcony";
}
function q(o, e, a) {
  const t = {
    "tytus-daemon": { species: "robot", palette: "violet", accessory: "antenna", animation: "breathe" },
    "ail-gateway": { species: "hologram", palette: "cyan", accessory: "sparkles", animation: "scan" },
    "openai-compatible": { species: "hologram", palette: "cyan", accessory: "sparkles", animation: "scan" },
    "custom-health": { species: "drone", palette: "green", accessory: "shield", animation: "pulse" },
    "openhouse-probe": { species: "robot", palette: "silver", accessory: "sparkles", animation: "breathe" },
    "mcp-http": { species: "owl", palette: "amber", accessory: "book", animation: "breathe" }
  }, s = e === "busy" ? { animation: "typing" } : e === "starting" ? { palette: "amber", animation: "pulse" } : e === "degraded" ? { palette: "amber", animation: "scan" } : e === "error" ? { palette: "red", animation: "alarm" } : e === "offline" ? { species: "ghost", palette: "silver", animation: "sleep" } : {};
  return { ...t[o], ...s, ...a };
}
function J(o) {
  const e = String(o ?? "").toLowerCase();
  return ["ready", "running", "healthy", "ok", "online", "connected", "up"].includes(e) ? "online" : ["busy", "working", "executing", "writing", "researching", "syncing"].includes(e) ? "busy" : ["starting", "booting", "warming"].includes(e) ? "starting" : ["degraded", "warning", "warn"].includes(e) ? "degraded" : ["error", "failed", "unhealthy", "down"].includes(e) ? "error" : ["offline", "stopped", "idle-offline"].includes(e) ? "offline" : "unknown";
}
const Re = [
  /Bearer\s+[A-Za-z0-9._\-~+/]+=*/gi,
  /sk-[A-Za-z0-9_\-]{12,}/g,
  /gsk_[A-Za-z0-9_\-]{12,}/g,
  /AIza[0-9A-Za-z_\-]{20,}/g
], ze = /(api[_-]?key|token|secret|authorization)(["'\s:=]+)([A-Za-z0-9._\-~+/=]{8,})/gi;
function Z(o) {
  let e = o instanceof Error ? o.message : String(o ?? "");
  for (const a of Re)
    e = e.replace(a, "[REDACTED]");
  return e = e.replace(ze, (a, t, s) => `${t}${s}[REDACTED]`), e;
}
function V(o) {
  try {
    return new URL(o).host;
  } catch {
    return o.replace(/[?#].*$/, "");
  }
}
function B(o) {
  const e = o.trim().replace(/\/+$/, "");
  if (!e) throw new Error("Base URL is required.");
  const a = new URL(e), t = a.hostname.toLowerCase(), s = t === "localhost" || t === "127.0.0.1" || t === "[::1]" || t === "::1";
  if (a.protocol === "https:") return a.toString().replace(/\/+$/, "");
  if (a.protocol === "http:" && s) return a.toString().replace(/\/+$/, "");
  throw new Error("Only HTTPS and local HTTP endpoints are allowed in OpenHouse v1.");
}
function Y(o, e) {
  const a = (o || e).trim() || e;
  return a.startsWith("/") ? a : `/${a}`;
}
function $(o, e) {
  return `${o.replace(/\/+$/, "")}${Y(e, "/")}`;
}
function ce(o, e) {
  const a = new Map(e.map((r) => [r.id, r])), t = /* @__PURE__ */ new Set(), s = [];
  for (const r of o) {
    const c = a.get(r.id), d = J(r.status);
    t.add(r.id), s.push({
      id: `tytus:${r.id}`,
      sourceId: "tytus-daemon",
      sourceKind: "tytus-daemon",
      displayName: $e(r, c),
      status: d,
      mood: F(d),
      body: q("tytus-daemon", d),
      room: W("tytus-daemon", c?.publicUrl, d),
      endpointHost: c?.publicUrl ? V(c.publicUrl) : void 0,
      capabilities: c?.kind === "ail" ? ["models", "chat"] : ["unknown"],
      lastSeenAt: Date.now(),
      raw: { agent: r, pod: c }
    });
  }
  for (const r of e) {
    if (t.has(r.id)) continue;
    const c = J(r.status), d = r.kind === "ail", m = d ? "ail-gateway" : "tytus-daemon";
    s.push({
      id: `tytus:${r.id}`,
      sourceId: m,
      sourceKind: m,
      displayName: d ? Ue(r, e) : `Tytus Pod ${r.id}`,
      status: c,
      mood: F(c),
      body: q(m, c),
      room: W(m, r.publicUrl, c),
      endpointHost: r.publicUrl ? V(r.publicUrl) : void 0,
      capabilities: d ? ["models", "chat"] : ["unknown"],
      lastSeenAt: Date.now(),
      raw: { pod: r }
    });
  }
  return s;
}
function $e(o, e) {
  const a = typeof o.meta?.name == "string" ? o.meta.name : void 0, t = typeof o.meta?.kind == "string" ? o.meta.kind : e?.kind;
  return a || (t ? `${t.toUpperCase()} ${o.id}` : `Tytus Pod ${o.id}`);
}
function Ue(o, e) {
  return e.filter((t) => t.kind === "ail").length <= 1 ? "AIL" : `AIL (${o.id})`;
}
const Be = 6e3;
async function pe(o, e) {
  const a = performance.now();
  try {
    const t = await Ke(o, e, Be), s = Math.max(0, Math.round(performance.now() - a)), r = t.status;
    return {
      id: t.agentId,
      sourceId: o.id,
      sourceKind: o.kind,
      displayName: t.displayName || o.name,
      status: r,
      mood: t.mood || F(r),
      body: q(o.kind, r, { ...o.body, ...t.body }),
      room: o.room || W(o.kind, o.baseUrl, r),
      endpointHost: V(o.baseUrl),
      capabilities: t.capabilities.length ? t.capabilities : ["unknown"],
      latencyMs: t.latencyMs ?? s,
      lastSeenAt: r === "offline" ? void 0 : Date.now(),
      lastError: t.lastError,
      raw: t.raw
    };
  } catch (t) {
    const s = Je(t), r = Z(t);
    return {
      id: D(o),
      sourceId: o.id,
      sourceKind: o.kind,
      displayName: o.name,
      status: s,
      mood: F(s),
      body: q(o.kind, s, o.body),
      room: o.room || W(o.kind, o.baseUrl, s),
      endpointHost: V(o.baseUrl),
      capabilities: ["unknown"],
      latencyMs: Math.max(0, Math.round(performance.now() - a)),
      lastError: r
    };
  }
}
async function De(o, e) {
  const a = performance.now();
  try {
    const t = await e("/v1/models"), s = await t.text().catch(() => "");
    let r = ["models"];
    return (s.includes("chat") || s.includes("gpt") || s.includes("model")) && (r = ["models", "chat"]), {
      agentId: o,
      sourceId: "tytus-daemon",
      status: t.ok ? "online" : "degraded",
      latencyMs: Math.round(performance.now() - a),
      capabilities: r,
      lastError: t.ok ? void 0 : `HTTP ${t.status}`
    };
  } catch (t) {
    return {
      agentId: o,
      sourceId: "tytus-daemon",
      status: "error",
      latencyMs: Math.round(performance.now() - a),
      capabilities: ["unknown"],
      lastError: Z(t)
    };
  }
}
async function Ke(o, e, a) {
  switch (o.kind) {
    case "openai-compatible":
      return je(o, e, a);
    case "custom-health":
      return Ge(o, a);
    case "openhouse-probe":
      return Fe(o, a);
    case "mcp-http":
      return We(o, a);
    default:
      throw new Error(`Unsupported external source kind: ${o.kind}`);
  }
}
async function je(o, e, a) {
  const t = await U($(B(o.baseUrl), "/v1/models"), o, e, a), s = await ne(t), r = Array.isArray(s.data) ? s.data.length : void 0;
  return {
    agentId: D(o),
    sourceId: o.id,
    status: t.ok ? "online" : t.status === 401 || t.status === 403 ? "degraded" : "error",
    latencyMs: void 0,
    capabilities: ["models", "chat"],
    lastError: t.ok ? void 0 : `HTTP ${t.status}`,
    raw: { modelCount: r }
  };
}
async function Ge(o, e) {
  const a = await U($(B(o.baseUrl), Y(o.healthPath, "/health")), o, { bearerBySourceId: {} }, e), t = await ne(a), s = qe(t, ["status", "state", "health"]), c = (typeof t.healthy == "boolean" ? t.healthy : void 0) === !1 ? "error" : s ? J(s) : a.ok ? "online" : "error";
  return {
    agentId: D(o),
    sourceId: o.id,
    status: c,
    capabilities: xe(t, ["health"]),
    displayName: typeof t.name == "string" ? t.name : void 0,
    lastError: a.ok ? void 0 : `HTTP ${a.status}`,
    raw: we(t)
  };
}
async function Fe(o, e) {
  const a = await U($(B(o.baseUrl), "/.well-known/openhouse-agent.json"), o, { bearerBySourceId: {} }, e), t = await ne(a), s = J(t.status ?? (a.ok ? "online" : "error")), r = {
    species: t.species,
    palette: t.palette,
    accessory: t.accessory
  };
  return {
    agentId: String(t.id || D(o)),
    sourceId: o.id,
    status: s,
    mood: typeof t.mood == "string" ? t.mood : void 0,
    displayName: typeof t.name == "string" ? t.name : o.name,
    capabilities: xe(t, ["health"]),
    body: r,
    lastError: a.ok ? void 0 : `HTTP ${a.status}`,
    raw: we(t)
  };
}
async function We(o, e) {
  const a = B(o.baseUrl);
  let t;
  try {
    t = await U($(a, "/.well-known/mcp"), o, { bearerBySourceId: {} }, e);
  } catch {
    t = await U($(a, Y(o.healthPath, "/health")), o, { bearerBySourceId: {} }, e);
  }
  return {
    agentId: D(o),
    sourceId: o.id,
    status: t.ok ? "online" : "degraded",
    capabilities: ["mcp", "tools"],
    lastError: t.ok ? void 0 : `HTTP ${t.status}`,
    raw: { protocol: "mcp-http-basic" }
  };
}
async function U(o, e, a, t) {
  const s = new AbortController(), r = window.setTimeout(() => s.abort(), t), c = new Headers({ Accept: "application/json" });
  if (e.authMode === "session-bearer") {
    const d = a.bearerBySourceId[e.id];
    d && c.set("Authorization", `Bearer ${d}`);
  }
  try {
    return await fetch(o, { method: "GET", headers: c, signal: s.signal, cache: "no-store" });
  } catch (d) {
    throw d instanceof DOMException && d.name === "AbortError" ? new Error("Probe timed out after 6s.") : d instanceof TypeError ? new Error("Network/CORS failure. If the endpoint is online, allow this Tytus origin or add a bridge.") : d;
  } finally {
    window.clearTimeout(r);
  }
}
async function ne(o) {
  const e = await o.text().catch(() => "");
  if (!e) return {};
  try {
    const a = JSON.parse(e);
    return a && typeof a == "object" ? a : { value: a };
  } catch {
    return { text: e.slice(0, 500) };
  }
}
function xe(o, e) {
  const a = o.capabilities;
  if (!Array.isArray(a)) return e;
  const t = /* @__PURE__ */ new Set(["models", "chat", "tools", "files", "health", "mcp", "music", "unknown"]), s = a.map((r) => String(r).toLowerCase()).filter((r) => t.has(r));
  return s.length ? s : e;
}
function we(o) {
  const e = {};
  for (const a of ["id", "name", "status", "state", "version", "capabilities", "healthy"])
    a in o && (e[a] = o[a]);
  return e;
}
function qe(o, e) {
  for (const a of e) if (a in o) return o[a];
}
function D(o) {
  return `external:${o.id}`;
}
function Je(o) {
  const e = Z(o).toLowerCase();
  return e.includes("cors") || e.includes("timed out") ? "degraded" : e.includes("network") ? "offline" : "error";
}
const Ve = 100;
async function X(o) {
  try {
    return (await o.query("SELECT * FROM app_openhouse_sources ORDER BY created_at ASC")).map(eo);
  } catch {
    return [];
  }
}
async function Ze(o, e) {
  await o.run(
    `INSERT INTO app_openhouse_sources
      (id, kind, name, base_url, health_path, enabled, auth_mode, keychain_ref, body_json, room, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
     ON CONFLICT(id) DO UPDATE SET
       kind=excluded.kind, name=excluded.name, base_url=excluded.base_url,
       health_path=excluded.health_path, enabled=excluded.enabled,
       auth_mode=excluded.auth_mode, keychain_ref=excluded.keychain_ref,
       body_json=excluded.body_json, room=excluded.room, updated_at=excluded.updated_at`,
    [
      e.id,
      e.kind,
      e.name,
      e.baseUrl,
      e.healthPath ?? null,
      e.enabled ? 1 : 0,
      e.authMode,
      e.keychainRef ?? null,
      JSON.stringify(e.body ?? {}),
      e.room ?? null,
      e.createdAt,
      e.updatedAt
    ]
  );
}
async function Ye(o, e) {
  await o.run("DELETE FROM app_openhouse_sources WHERE id = ?", [e]), await o.run("DELETE FROM app_openhouse_agents WHERE source_id = ?", [e]), await o.run("DELETE FROM app_openhouse_probe_history WHERE source_id = ?", [e]);
}
async function Xe(o) {
  try {
    return (await o.query("SELECT * FROM app_openhouse_layout")).map((a) => ({ agentId: a.agent_id, room: a.room, x: a.x, y: a.y, pinned: !!a.pinned, updatedAt: a.updated_at }));
  } catch {
    return [];
  }
}
async function Qe(o, e) {
  await o.run(
    `INSERT OR REPLACE INTO app_openhouse_probe_history
      (id, agent_id, source_id, status, latency_ms, error, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)`,
    [e.id, e.agentId, e.sourceId, e.status, e.latencyMs ?? null, e.error ? Z(e.error) : null, e.createdAt]
  ), await o.run(
    `DELETE FROM app_openhouse_probe_history
      WHERE id NOT IN (SELECT id FROM app_openhouse_probe_history ORDER BY created_at DESC LIMIT ${Ve})`
  );
}
async function Q(o, e) {
  try {
    return (e ? await o.query("SELECT * FROM app_openhouse_probe_history WHERE agent_id = ? ORDER BY created_at DESC LIMIT 20", [e]) : await o.query("SELECT * FROM app_openhouse_probe_history ORDER BY created_at DESC LIMIT 50")).map((t) => ({ id: t.id, agentId: t.agent_id, sourceId: t.source_id, status: t.status, latencyMs: t.latency_ms, error: t.error, createdAt: t.created_at }));
  } catch {
    return [];
  }
}
function eo(o) {
  return {
    id: o.id,
    kind: o.kind,
    name: o.name,
    baseUrl: o.base_url,
    healthPath: o.health_path ?? void 0,
    enabled: !!o.enabled,
    authMode: o.auth_mode,
    keychainRef: o.keychain_ref,
    body: oo(o.body_json),
    room: o.room,
    createdAt: o.created_at,
    updatedAt: o.updated_at
  };
}
function oo(o) {
  try {
    const e = JSON.parse(o);
    return e && typeof e == "object" && !Array.isArray(e) ? e : {};
  } catch {
    return {};
  }
}
const to = "https://cdn.jsdelivr.net/gh/traylinx/tytus-app-openhouse@8eeb9ee71585f5d82340013a8c0bc11e57be0f49/assets/star-office/", R = (o) => `${to}${o}`, H = [
  { x: 22, y: 61, scale: 1.35, sprite: "guestagent1.webp", room: "carpet desk" },
  { x: 68, y: 30, scale: 1.18, sprite: "guestagent2.webp", labelDx: -2, room: "server room" },
  { x: 50, y: 63, scale: 1.12, sprite: "guest_anim_2.webp", room: "coffee table" },
  { x: 33, y: 79, scale: 1.18, sprite: "star-idle.gif", room: "entry walk" },
  { x: 40, y: 52, scale: 1.08, sprite: "guest_anim_1.webp", room: "remote balcony" },
  { x: 82, y: 50, scale: 1.05, sprite: "guest_anim_3.webp", room: "library wall" },
  { x: 15, y: 35, scale: 1.05, sprite: "star-working.gif", room: "bookshelf" },
  { x: 58, y: 43, scale: 1, sprite: "guest_anim_2.webp", room: "sofa" }
], ao = {
  "tytus-daemon": [0, 1, 2, 3],
  "ail-gateway": [4],
  "openai-compatible": [5, 4],
  "mcp-http": [5],
  "custom-health": [2, 6],
  "openhouse-probe": [3, 6, 7]
};
function io({ agents: o, selectedId: e, onSelect: a, t }) {
  const s = o.slice(0, H.length), r = Math.max(0, o.length - H.length), c = /* @__PURE__ */ new Set();
  return /* @__PURE__ */ i("div", { className: "oh-office-frame", "aria-label": t("office.aria"), children: /* @__PURE__ */ l("div", { className: "oh-office-world", children: [
    /* @__PURE__ */ i("img", { className: "oh-office-bg", src: R("office_bg.webp"), alt: t("office.alt"), draggable: !1 }),
    /* @__PURE__ */ i("img", { className: "oh-office-prop oh-server", src: R("serverroom.gif"), alt: t("office.serverRoomAlt"), draggable: !1 }),
    /* @__PURE__ */ i("img", { className: "oh-office-prop oh-coffee", src: R("coffee-machine.gif"), alt: t("office.coffeeMachineAlt"), draggable: !1 }),
    /* @__PURE__ */ i("img", { className: "oh-office-prop oh-sofa", src: R("sofa-idle.webp"), alt: t("office.sofaAlt"), draggable: !1 }),
    s.map((d, m) => {
      const C = no(d, m, c), g = H[C] ?? H[0];
      return /* @__PURE__ */ l(
        "button",
        {
          type: "button",
          className: `oh-office-agent ${d.id === e ? "selected" : ""} ${d.status} ${d.sourceKind === "ail-gateway" ? "ail" : ""}`,
          style: { left: `${g.x}%`, top: `${g.y}%`, "--agent-scale": g.scale },
          onClick: () => a(d.id),
          title: `${d.displayName} · ${d.status} · ${g.room}`,
          children: [
            /* @__PURE__ */ i("span", { className: "oh-agent-label", style: { transform: `translateX(${g.labelDx ?? 0}px)` }, children: lo(d.displayName) }),
            /* @__PURE__ */ i("span", { className: "oh-bubble", children: so(d, t) }),
            /* @__PURE__ */ i("img", { src: R(ro(d, g.sprite)), alt: "", draggable: !1 }),
            /* @__PURE__ */ i("span", { className: "oh-status-dot" })
          ]
        },
        d.id
      );
    }),
    r > 0 && /* @__PURE__ */ i("div", { className: "oh-overflow-badge", children: t("office.moreAgents", { count: r }) }),
    /* @__PURE__ */ l("div", { className: "oh-office-name", children: [
      /* @__PURE__ */ i("span", { children: "★" }),
      " ",
      t("office.name"),
      " ",
      /* @__PURE__ */ i("span", { children: "★" })
    ] })
  ] }) });
}
function no(o, e, a) {
  const s = (ao[o.sourceKind] ?? []).find((r) => !a.has(r)) ?? H.findIndex((r, c) => !a.has(c)) ?? e % H.length;
  return a.add(s), s;
}
function ro(o, e) {
  return o.sourceKind === "ail-gateway" ? "guest_anim_1.webp" : o.status === "busy" || o.mood === "focused" || o.mood === "thinking" ? "star-working.gif" : o.status === "error" || o.status === "offline" || o.mood === "sick" ? "guestagent2.webp" : o.sourceKind === "mcp-http" ? "guest_anim_3.webp" : o.sourceKind === "openai-compatible" ? "guest_anim_1.webp" : e;
}
function so(o, e) {
  return o.sourceKind === "ail-gateway" ? e("bubble.models") : o.status === "online" ? e("bubble.ready") : o.status === "busy" ? e("bubble.working") : o.status === "offline" ? e("bubble.offline") : o.status === "error" ? e("bubble.bug") : o.status === "degraded" ? e("bubble.checkMe") : e("bubble.waiting");
}
function lo(o) {
  return o.length <= 14 ? o : `${o.slice(0, 13)}…`;
}
const co = [
  { value: "openai-compatible", labelKey: "kind.openaiCompatible", helpKey: "kind.help.models" },
  { value: "custom-health", labelKey: "kind.customHealth", helpKey: "kind.help.health" },
  { value: "openhouse-probe", labelKey: "kind.openHouseProbe", helpKey: "kind.help.openHouse" },
  { value: "mcp-http", labelKey: "kind.mcpHttp", helpKey: "kind.help.mcp" }
];
function po({ open: o, onClose: e, onAdd: a, t }) {
  const [s, r] = b("openai-compatible"), [c, d] = b(""), [m, C] = b(""), [g, M] = b("/health"), [E, v] = b("none"), [S, K] = b(""), [O, I] = b(null);
  return o ? /* @__PURE__ */ i("div", { className: "oh-modal-backdrop", role: "dialog", "aria-modal": "true", children: /* @__PURE__ */ l("form", { className: "oh-modal", onSubmit: (u) => {
    u.preventDefault();
    try {
      const k = B(m), _ = Date.now(), x = `src-${_.toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
      a({
        id: x,
        kind: s,
        name: c.trim() || uo(k),
        baseUrl: k,
        healthPath: s === "custom-health" || s === "mcp-http" ? Y(g, "/health") : void 0,
        enabled: !0,
        authMode: E,
        createdAt: _,
        updatedAt: _
      }, E === "session-bearer" ? S : void 0), d(""), C(""), K(""), v("none"), I(null), e();
    } catch (k) {
      I(k instanceof Error ? k.message : String(k));
    }
  }, children: [
    /* @__PURE__ */ l("div", { className: "oh-modal-head", children: [
      /* @__PURE__ */ i("h2", { children: t("dialog.title") }),
      /* @__PURE__ */ i("button", { type: "button", onClick: e, children: /* @__PURE__ */ i(Le, { size: 18 }) })
    ] }),
    /* @__PURE__ */ l("label", { children: [
      t("dialog.connectorType"),
      /* @__PURE__ */ i("select", { value: s, onChange: (u) => r(u.target.value), children: co.map((u) => /* @__PURE__ */ l("option", { value: u.value, children: [
        t(u.labelKey),
        " — ",
        t(u.helpKey)
      ] }, u.value)) })
    ] }),
    /* @__PURE__ */ l("label", { children: [
      t("dialog.name"),
      /* @__PURE__ */ i("input", { value: c, onChange: (u) => d(u.target.value), placeholder: t("dialog.namePlaceholder") })
    ] }),
    /* @__PURE__ */ l("label", { children: [
      t("dialog.baseUrl"),
      /* @__PURE__ */ i("input", { value: m, onChange: (u) => C(u.target.value), placeholder: t("dialog.baseUrlPlaceholder"), required: !0 })
    ] }),
    (s === "custom-health" || s === "mcp-http") && /* @__PURE__ */ l("label", { children: [
      t("dialog.healthPath"),
      /* @__PURE__ */ i("input", { value: g, onChange: (u) => M(u.target.value), placeholder: "/health" })
    ] }),
    /* @__PURE__ */ l("label", { children: [
      t("dialog.authMode"),
      /* @__PURE__ */ l("select", { value: E, onChange: (u) => v(u.target.value), children: [
        /* @__PURE__ */ i("option", { value: "none", children: t("dialog.authNone") }),
        /* @__PURE__ */ i("option", { value: "session-bearer", children: t("dialog.authSessionBearer") })
      ] })
    ] }),
    E === "session-bearer" && /* @__PURE__ */ l("label", { children: [
      t("dialog.sessionBearer"),
      /* @__PURE__ */ i("input", { type: "password", value: S, onChange: (u) => K(u.target.value), placeholder: t("dialog.sessionBearerPlaceholder") })
    ] }),
    O && /* @__PURE__ */ i("p", { className: "oh-form-error", children: O }),
    /* @__PURE__ */ l("footer", { children: [
      /* @__PURE__ */ i("button", { type: "button", onClick: e, children: t("dialog.cancel") }),
      /* @__PURE__ */ l("button", { className: "primary", children: [
        /* @__PURE__ */ i(ge, { size: 16 }),
        t("dialog.addResident")
      ] })
    ] })
  ] }) }) : null;
}
function uo(o) {
  try {
    return new URL(o).hostname;
  } catch {
    return "External agent";
  }
}
const ho = `
@font-face {
  font-family: ArkPixel;
  src: url('https://cdn.jsdelivr.net/gh/traylinx/tytus-app-openhouse@8eeb9ee71585f5d82340013a8c0bc11e57be0f49/assets/star-office/fonts/ark-pixel-12px-proportional-latin.ttf.woff2') format('woff2');
  font-weight: 400;
  font-style: normal;
  font-display: swap;
}
.oh-root {
  --bg: #1a1a2e;
  --surface: #2c2f3a;
  --surface2: #353945;
  --surface3: #242837;
  --accent: #e94560;
  --accent-dark: #b8354c;
  --gold: #ffd700;
  --text: #f3f4f6;
  --muted: #a7adbb;
  --border: #4a4f5f;
  --shadow: 4px 4px 0 #0a0a12;
  height: 100%;
  min-height: 0;
  background: var(--bg);
  background-image: linear-gradient(90deg, rgba(255,255,255,.025) 1px, transparent 1px), linear-gradient(rgba(255,255,255,.025) 1px, transparent 1px);
  background-size: 8px 8px;
  color: var(--text);
  font-family: ArkPixel, "Courier New", monospace;
  display: grid;
  grid-template-rows: minmax(0, 1fr) minmax(142px, 22vh);
  gap: 8px;
  padding: 6px 6px 62px;
  overflow: hidden;
  image-rendering: pixelated;
}
.oh-root * { box-sizing: border-box; }
.oh-root button, .oh-root select { font-family: inherit; }
.oh-top {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) clamp(250px, 18vw, 310px);
  gap: 8px;
}
.oh-game-column {
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: minmax(0, 1fr) 58px;
  gap: 8px;
}
.oh-office-frame {
  min-height: 0;
  position: relative;
  overflow: hidden;
  background: #111124;
  border: 4px solid var(--accent);
  box-shadow: var(--shadow), 0 0 0 2px #0f0f18;
}
.oh-office-world {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 320px;
  overflow: hidden;
  background: #161628;
}
.oh-office-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: fill;
  image-rendering: pixelated;
  user-select: none;
}
.oh-office-prop {
  position: absolute;
  image-rendering: pixelated;
  pointer-events: none;
  filter: drop-shadow(3px 4px 0 rgba(0,0,0,.5));
}
.oh-server { width: 10%; right: 12.8%; top: 16%; }
.oh-coffee { width: 7.2%; left: 45%; top: 50%; }
.oh-sofa { width: 10.5%; right: 31%; top: 36%; }
.oh-office-agent {
  position: absolute;
  z-index: 5;
  border: 0;
  background: transparent;
  padding: 0;
  margin: 0;
  cursor: pointer;
  transform: translate(-50%, -100%) scale(var(--agent-scale));
  transform-origin: 50% 100%;
  image-rendering: pixelated;
  filter: drop-shadow(3px 5px 0 rgba(0,0,0,.78));
}
.oh-office-agent img {
  display: block;
  max-width: 68px;
  max-height: 88px;
  image-rendering: pixelated;
}
.oh-office-agent:hover { filter: drop-shadow(0 0 0 #fff) drop-shadow(3px 5px 0 rgba(0,0,0,.8)); z-index: 9; }
.oh-office-agent.selected img { outline: 3px solid #fff; outline-offset: 2px; background: rgba(255,255,255,.16); }
.oh-office-agent.busy img, .oh-office-agent.online img { animation: ohBob 1.2s steps(2,end) infinite; }
.oh-office-agent.error img, .oh-office-agent.offline img { filter: grayscale(.22) saturate(.82); }
.oh-office-agent.ail img { filter: drop-shadow(0 0 7px rgba(103,232,249,.75)); }
.oh-agent-label {
  position: absolute;
  bottom: 100%;
  left: 50%;
  translate: -50% -4px;
  color: white;
  text-shadow: 2px 2px 0 #000, -1px -1px 0 #000;
  font-size: clamp(9px, .75vw, 11px);
  white-space: nowrap;
  font-weight: 900;
  letter-spacing: .02em;
}
.oh-bubble {
  position: absolute;
  left: 50%;
  bottom: calc(100% + 22px);
  translate: -50% 0;
  background: #fff;
  color: #111;
  padding: 5px 8px;
  border: 3px solid #111;
  box-shadow: 3px 3px 0 rgba(0,0,0,.45);
  font-size: 10px;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
}
.oh-office-agent:hover .oh-bubble, .oh-office-agent.selected .oh-bubble { opacity: 1; }
.oh-status-dot {
  position: absolute;
  right: 3px;
  top: 3px;
  width: 9px;
  height: 9px;
  border: 2px solid #111;
  background: #94a3b8;
}
.oh-office-agent.online .oh-status-dot, .oh-office-agent.busy .oh-status-dot { background: #22c55e; }
.oh-office-agent.error .oh-status-dot { background: #ef4444; }
.oh-office-agent.offline .oh-status-dot { background: #64748b; }
.oh-office-agent.degraded .oh-status-dot, .oh-office-agent.starting .oh-status-dot { background: #f59e0b; }
.oh-office-name {
  position: absolute;
  left: 50%;
  bottom: 10px;
  translate: -50% 0;
  min-width: min(360px, 58%);
  text-align: center;
  background: #6b4638;
  color: var(--gold);
  border: 4px solid #3b241d;
  box-shadow: 3px 3px 0 #0a0a12;
  padding: 8px 18px;
  font-size: clamp(10px, .9vw, 13px);
  text-shadow: 2px 2px 0 #000;
}
.oh-office-name span { font-size: 18px; margin: 0 14px; }
.oh-overflow-badge {
  position: absolute;
  right: 14px;
  bottom: 14px;
  background: #252536;
  border: 3px solid var(--accent);
  box-shadow: 3px 3px 0 #0a0a12;
  color: var(--gold);
  padding: 8px 10px;
  font-size: 11px;
}
.oh-toolbar {
  min-width: 0;
  min-height: 58px;
  background: #252536;
  border: 3px solid var(--border);
  box-shadow: var(--shadow);
  padding: 8px 10px;
  display: flex;
  align-items: center;
  gap: 9px;
  overflow: hidden;
}
.oh-toolbar label { color: var(--gold); font-size: 12px; }
.oh-toolbar select, .oh-toolbar button {
  background: var(--surface2);
  color: var(--text);
  border: 3px solid var(--border);
  box-shadow: 2px 2px 0 #0a0a12;
  border-radius: 0;
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 900;
  white-space: nowrap;
}
.oh-toolbar select { max-width: 190px; }
.oh-toolbar button:hover { border-color: var(--accent); color: #fff; }
.oh-toolbar .spacer { flex: 1; min-width: 8px; }
.oh-toolbar .counter {
  border: 3px solid var(--border);
  background: #1f2330;
  padding: 8px 10px;
  color: var(--gold);
  box-shadow: 2px 2px 0 #0a0a12;
  font-size: 12px;
  white-space: nowrap;
}
.oh-side {
  min-height: 0;
  background: var(--surface);
  border: 3px solid var(--accent);
  box-shadow: var(--shadow);
  display: flex;
  flex-direction: column;
}
.oh-side-head { padding: 11px 12px; border-bottom: 3px solid var(--border); }
.oh-side-head p { margin: 0; color: var(--gold); letter-spacing: .12em; font-size: 10px; text-transform: uppercase; }
.oh-side-head h2 { margin: 5px 0 0; font-size: clamp(17px, 1.4vw, 22px); color: #fff; line-height: 1.05; }
.oh-side-body {
  min-height: 0;
  padding: 10px;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 9px;
}
.oh-side-card {
  background: #3a3e4d;
  border: 3px solid var(--border);
  box-shadow: 2px 2px 0 #0a0a12;
  padding: 9px;
}
.oh-side-card h3 { margin: 0 0 8px; color: var(--gold); font-size: 12px; }
.oh-side-card p { margin: 0; color: #e5e7eb; font-size: 11px; line-height: 1.35; }
.oh-side-card dl { display: grid; grid-template-columns: 72px 1fr; gap: 5px 8px; margin: 0; font-size: 11px; }
.oh-side-card dt { color: #cbd5e1; }
.oh-side-card dd { margin: 0; color: #fff; word-break: break-word; }
.oh-abilities { display: grid; gap: 6px; }
.oh-ability {
  background: #282d3d;
  border-left: 4px solid var(--gold);
  padding: 6px 7px;
}
.oh-ability strong { display: block; color: #fff; font-size: 11px; margin-bottom: 2px; }
.oh-ability span { display: block; color: #d7dce7; font-size: 10px; line-height: 1.35; }
.oh-caps, .oh-mini-caps { display: flex; flex-wrap: wrap; gap: 5px; }
.oh-caps span, .oh-mini-caps span {
  background: #252536;
  border: 2px solid #4a4f5f;
  color: #ffd700;
  padding: 4px 6px;
  font-size: 10px;
}
.oh-mini-caps { margin-top: 5px; }
.oh-mini-caps span { font-size: 9px; padding: 2px 5px; }
.oh-actions { display: grid; gap: 8px; }
.oh-actions button {
  background: #4b5162;
  color: white;
  border: 3px solid #171923;
  box-shadow: 2px 2px 0 #05070d;
  padding: 9px;
  font-weight: 900;
}
.oh-actions button:hover { border-color: var(--accent); }
.oh-history p { display: flex; justify-content: space-between; gap: 6px; }
.oh-history b { color: #fff; }
.oh-history span { color: var(--muted); }
.oh-error { color: #fecaca; border-color: #ef4444; }
.oh-bottom {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(220px, 1.05fr) minmax(260px, .9fr) minmax(320px, 1.18fr);
  gap: 8px;
}
.oh-panel {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--surface);
  border: 3px solid var(--accent);
  box-shadow: var(--shadow);
}
.oh-panel-title {
  color: var(--gold);
  font-size: clamp(13px, 1.1vw, 16px);
  font-weight: 900;
  text-align: center;
  letter-spacing: .08em;
  padding: 8px 10px;
  border-bottom: 3px solid var(--border);
  text-shadow: 2px 2px 0 #1a1020;
}
.oh-memo {
  background-image: url('https://cdn.jsdelivr.net/gh/traylinx/tytus-app-openhouse@8eeb9ee71585f5d82340013a8c0bc11e57be0f49/assets/star-office/memo-bg.webp');
  background-size: cover;
  color: #5c4326;
  align-items: center;
  justify-content: center;
  text-align: center;
}
.oh-memo .oh-panel-title { border: 0; color: #5c4326; text-shadow: none; }
.oh-memo p { font-size: clamp(10px, .9vw, 12px); margin: 0; max-width: 82%; line-height: 1.45; }
.oh-playbook p { margin: 0 14px 10px; color: #cfd5e4; font-size: 11px; line-height: 1.35; text-align: center; }
.oh-status-grid {
  min-height: 0;
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  padding: 10px 12px;
}
.oh-status-grid button {
  background: var(--surface2);
  color: var(--text);
  border: 3px solid var(--border);
  box-shadow: 2px 2px 0 #0a0a12;
  font-size: clamp(11px, .95vw, 14px);
  font-weight: 900;
}
.oh-status-grid button:hover { border-color: var(--accent); }
.oh-visitor-list {
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding: 8px;
  overflow: auto;
}
.oh-visitor {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 7px;
  align-items: center;
  background: #3a3e4d;
  border-left: 7px solid var(--accent);
  border-bottom: 4px solid #111827;
  padding: 8px;
}
.oh-visitor strong { display: block; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13px; }
.oh-visitor small { display: block; color: #d1d5db; font-size: 10px; margin-top: 2px; }
.oh-visitor button {
  background: #4b5162;
  color: white;
  border: 3px solid #171923;
  box-shadow: 2px 2px 0 #05070d;
  padding: 7px 8px;
  font-size: 10px;
}
.oh-modal-backdrop {
  position: absolute;
  inset: 0;
  z-index: 20;
  background: rgba(10,10,18,.72);
  display: grid;
  place-items: center;
}
.oh-modal {
  width: min(560px, calc(100% - 40px));
  background: #252536;
  border: 4px solid var(--accent);
  box-shadow: var(--shadow);
  padding: 16px;
  display: grid;
  gap: 12px;
}
.oh-modal-head { display: flex; align-items: center; justify-content: space-between; }
.oh-modal h2 { margin: 0; color: var(--gold); }
.oh-modal label { display: grid; gap: 6px; color: #f3f4f6; font-size: 12px; }
.oh-modal input, .oh-modal select { background: #111827; color: #fff; border: 3px solid var(--border); padding: 9px; font-family: inherit; }
.oh-modal footer { display: flex; justify-content: flex-end; gap: 10px; }
.oh-modal button { background: #3a3e4d; color: #fff; border: 3px solid var(--border); box-shadow: 2px 2px 0 #0a0a12; padding: 9px 12px; }
.oh-modal .primary { background: #8b2640; }
.oh-form-error { color: #fecaca; margin: 0; font-size: 12px; }
@keyframes ohBob { 0%,100% { translate: 0 0; } 50% { translate: 0 -2px; } }
@media (max-width: 1180px) {
  .oh-root { overflow: auto; padding-bottom: 72px; grid-template-rows: auto auto; }
  .oh-top { grid-template-columns: 1fr; }
  .oh-game-column { grid-template-rows: minmax(390px, 55vh) auto; }
  .oh-side { min-height: 230px; }
  .oh-bottom { grid-template-columns: 1fr; min-height: 460px; }
  .oh-toolbar { flex-wrap: wrap; min-height: auto; }
}
@media (max-width: 1500px) {
  .oh-toolbar .counter { display: none; }
}
`, bo = {
  "toolbar.aria": "OpenHouse HUD",
  "toolbar.view": "View",
  "filter.all": "All residents",
  "filter.tytusPods": "Tytus Pods",
  "filter.ailGateway": "AIL Gateway",
  "filter.openaiCompatible": "OpenAI-compatible",
  "filter.healthChecks": "Health checks",
  "filter.openHouseAgents": "OpenHouse agents",
  "filter.mcpLibrary": "MCP Library",
  "filter.online": "Online",
  "filter.issues": "Issues",
  "action.addAgent": "Add agent",
  "action.refreshExternal": "Refresh external",
  "action.refreshing": "Refreshing…",
  "counter.residents": "{count} residents",
  "counter.online": "{count} online",
  "counter.issues": "{count} issues",
  "side.noResident": "No resident selected",
  "field.room": "Room",
  "field.source": "Source",
  "field.endpoint": "Endpoint",
  "field.latency": "Latency",
  "field.lastSeen": "Last seen",
  "section.canDo": "Can do",
  "section.capabilities": "Capabilities",
  "section.error": "Error",
  "section.probeHistory": "Probe history",
  "action.testing": "Testing…",
  "action.testConnection": "Test connection",
  "action.copyAgentContract": "Copy agent contract",
  "action.copyDiagnostic": "Copy diagnostic",
  "action.removeSource": "Remove source",
  "empty.office": "Add third-party agents or connect Tytus pods to populate the office.",
  "memo.title": "— Today Memo —",
  "memo.issues": "{count} residents need attention. Start with the red room.",
  "memo.calm": "Office calm. Agents have bodies, rooms, status, and action contracts.",
  "actions.title": "Agent Actions",
  "actions.all": "All",
  "actions.working": "Working",
  "actions.alert": "Alert",
  "actions.ail": "AIL",
  "actions.protocol": "Original Star-Office protocol kept: join, push status, leave, memo, probe.",
  "visitors.title": "— Visitor List —",
  "action.focus": "focus",
  "action.probe": "probe",
  "notify.residentAdded.title": "OpenHouse resident added",
  "notify.residentAdded.body": "{name} moved into the office.",
  "notify.probeFailed.title": "OpenHouse probe failed",
  "notify.diagnosticCopied.title": "OpenHouse diagnostic copied",
  "notify.contractCopied.title": "OpenHouse agent contract copied",
  "source.tytusPod": "Tytus pod",
  "source.ailGateway": "AIL gateway",
  "source.openaiCompatible": "OpenAI-compatible",
  "source.healthService": "Health service",
  "source.openHouseAgent": "OpenHouse agent",
  "source.mcpHttp": "MCP HTTP/SSE",
  "capability.models": "models",
  "capability.chat": "chat",
  "capability.tools": "tools",
  "capability.files": "files",
  "capability.health": "health",
  "capability.mcp": "mcp",
  "capability.music": "music",
  "capability.unknown": "unknown",
  "ability.tytusWork.title": "Run Tytus work",
  "ability.tytusWork.body": "Native pod resident. OpenHouse can focus it, probe it, and expose daemon status.",
  "ability.podEndpoint.title": "Pod endpoint",
  "ability.podEndpoint.body": "Uses Tytus pod calls for live checks such as /v1/models when available.",
  "ability.modelGateway.title": "Model gateway",
  "ability.modelGateway.body": "AIL is not a pod. It is the OpenAI-style gateway for /v1/models and chat routes.",
  "ability.providerRouting.title": "Provider routing",
  "ability.providerRouting.body": "Good place for model/provider capability checks before agents use it.",
  "ability.listModels.title": "List models",
  "ability.listModels.body": "Probes GET /v1/models and shows online/degraded/error state.",
  "ability.chatCapable.title": "Chat-capable",
  "ability.chatCapable.body": "Represents an LLM endpoint agents can use for chat/completions style work.",
  "ability.healthHeartbeat.title": "Health heartbeat",
  "ability.healthHeartbeat.body": "Tracks any service with /health or a custom JSON health path.",
  "ability.opsMonitor.title": "Ops monitor",
  "ability.opsMonitor.body": "Turns backend status into a visible room/body/status in the office.",
  "ability.residentCard.title": "Resident card",
  "ability.residentCard.body": "Reads /.well-known/openhouse-agent.json for name, body, mood, and capabilities.",
  "ability.thirdAgentNative.title": "Third-agent native",
  "ability.thirdAgentNative.body": "Best contract for OpenClaw, Hermes, Lope, Claude, OpenCode, or custom workers.",
  "ability.toolLibrary.title": "Tool library",
  "ability.toolLibrary.body": "Represents MCP servers that expose tools, resources, and prompts.",
  "ability.probeBridge.title": "Probe bridge",
  "ability.probeBridge.body": "Checks /.well-known/mcp or health fallback so tool servers can live in the house.",
  "ability.useTools.title": "Use tools",
  "ability.useTools.body": "Can expose callable tool actions to connected agents.",
  "ability.useFiles.title": "Use files",
  "ability.useFiles.body": "Can work with files/workspaces if the source grants access.",
  "ability.mcpProtocol.title": "MCP protocol",
  "ability.mcpProtocol.body": "Can advertise MCP resources/prompts/tools for agent workflows.",
  "ability.musicMedia.title": "Music/media",
  "ability.musicMedia.body": "Can support media/music generation flows when the endpoint exposes it.",
  "office.aria": "OpenHouse pixel office",
  "office.alt": "pixel office",
  "office.serverRoomAlt": "server room",
  "office.coffeeMachineAlt": "coffee machine",
  "office.sofaAlt": "sofa",
  "office.moreAgents": "+{count} more agents in the office",
  "office.name": "OpenHouse Agent Office",
  "bubble.models": "models",
  "bubble.ready": "ready",
  "bubble.working": "working",
  "bubble.offline": "offline",
  "bubble.bug": "bug",
  "bubble.checkMe": "check me",
  "bubble.waiting": "waiting",
  "dialog.title": "Add agent resident",
  "dialog.connectorType": "Connector type",
  "dialog.name": "Name",
  "dialog.namePlaceholder": "Agent Studio Local",
  "dialog.baseUrl": "Base URL",
  "dialog.baseUrlPlaceholder": "https://agent.example.com or http://localhost:8080",
  "dialog.healthPath": "Health path",
  "dialog.authMode": "Auth mode",
  "dialog.authNone": "None",
  "dialog.authSessionBearer": "Session bearer (not persisted)",
  "dialog.sessionBearer": "Session bearer",
  "dialog.sessionBearerPlaceholder": "Stored in memory only",
  "dialog.cancel": "Cancel",
  "dialog.addResident": "Add resident",
  "dialog.externalAgent": "External agent",
  "kind.openaiCompatible": "OpenAI-compatible",
  "kind.customHealth": "Custom health",
  "kind.openHouseProbe": "OpenHouse probe",
  "kind.mcpHttp": "MCP HTTP/SSE",
  "kind.help.models": "GET /v1/models",
  "kind.help.health": "GET /health or custom path",
  "kind.help.openHouse": "GET /.well-known/openhouse-agent.json",
  "kind.help.mcp": "Basic well-known/health probe"
}, fo = {
  "toolbar.aria": "HUD de OpenHouse",
  "toolbar.view": "Vista",
  "filter.all": "Todos los residentes",
  "filter.tytusPods": "Pods de Tytus",
  "filter.ailGateway": "Gateway AIL",
  "filter.openaiCompatible": "Compatible con OpenAI",
  "filter.healthChecks": "Checks de salud",
  "filter.openHouseAgents": "Agentes OpenHouse",
  "filter.mcpLibrary": "Biblioteca MCP",
  "filter.online": "En línea",
  "filter.issues": "Problemas",
  "action.addAgent": "Añadir agente",
  "action.refreshExternal": "Actualizar externos",
  "action.refreshing": "Actualizando…",
  "counter.residents": "{count} residentes",
  "counter.online": "{count} en línea",
  "counter.issues": "{count} problemas",
  "side.noResident": "Ningún residente seleccionado",
  "field.room": "Sala",
  "field.source": "Fuente",
  "field.endpoint": "Endpoint",
  "field.latency": "Latencia",
  "field.lastSeen": "Visto por última vez",
  "section.canDo": "Puede hacer",
  "section.capabilities": "Capacidades",
  "section.error": "Error",
  "section.probeHistory": "Historial de pruebas",
  "action.testing": "Probando…",
  "action.testConnection": "Probar conexión",
  "action.copyAgentContract": "Copiar contrato de agente",
  "action.copyDiagnostic": "Copiar diagnóstico",
  "action.removeSource": "Eliminar fuente",
  "empty.office": "Añade agentes de terceros o conecta pods de Tytus para poblar la oficina.",
  "memo.title": "— Memo de hoy —",
  "memo.issues": "{count} residentes necesitan atención. Empieza por la sala roja.",
  "memo.calm": "Oficina en calma. Los agentes tienen cuerpos, salas, estado y contratos de acción.",
  "actions.title": "Acciones de agente",
  "actions.all": "Todos",
  "actions.working": "Trabajando",
  "actions.alert": "Alerta",
  "actions.ail": "AIL",
  "actions.protocol": "Protocolo Star-Office original conservado: join, push status, leave, memo, probe.",
  "visitors.title": "— Lista de visitantes —",
  "action.focus": "enfocar",
  "action.probe": "probar",
  "notify.residentAdded.title": "Residente añadido a OpenHouse",
  "notify.residentAdded.body": "{name} se mudó a la oficina.",
  "notify.probeFailed.title": "Prueba de OpenHouse fallida",
  "notify.diagnosticCopied.title": "Diagnóstico de OpenHouse copiado",
  "notify.contractCopied.title": "Contrato de agente OpenHouse copiado",
  "source.tytusPod": "Pod de Tytus",
  "source.ailGateway": "Gateway AIL",
  "source.openaiCompatible": "Compatible con OpenAI",
  "source.healthService": "Servicio de salud",
  "source.openHouseAgent": "Agente OpenHouse",
  "source.mcpHttp": "MCP HTTP/SSE",
  "capability.models": "modelos",
  "capability.chat": "chat",
  "capability.tools": "herramientas",
  "capability.files": "archivos",
  "capability.health": "salud",
  "capability.mcp": "mcp",
  "capability.music": "música",
  "capability.unknown": "desconocido",
  "ability.tytusWork.title": "Ejecutar trabajo de Tytus",
  "ability.tytusWork.body": "Residente nativo de pod. OpenHouse puede enfocarlo, probarlo y exponer el estado del daemon.",
  "ability.podEndpoint.title": "Endpoint del pod",
  "ability.podEndpoint.body": "Usa llamadas a pods de Tytus para checks en vivo como /v1/models cuando están disponibles.",
  "ability.modelGateway.title": "Gateway de modelos",
  "ability.modelGateway.body": "AIL no es un pod. Es el gateway estilo OpenAI para /v1/models y rutas de chat.",
  "ability.providerRouting.title": "Ruteo de proveedores",
  "ability.providerRouting.body": "Buen lugar para comprobar capacidades de modelo/proveedor antes de que los agentes lo usen.",
  "ability.listModels.title": "Listar modelos",
  "ability.listModels.body": "Prueba GET /v1/models y muestra estado en línea/degradado/error.",
  "ability.chatCapable.title": "Compatible con chat",
  "ability.chatCapable.body": "Representa un endpoint LLM que los agentes pueden usar para trabajo tipo chat/completions.",
  "ability.healthHeartbeat.title": "Pulso de salud",
  "ability.healthHeartbeat.body": "Rastrea cualquier servicio con /health o una ruta JSON de salud personalizada.",
  "ability.opsMonitor.title": "Monitor de ops",
  "ability.opsMonitor.body": "Convierte el estado backend en una sala/cuerpo/estado visible en la oficina.",
  "ability.residentCard.title": "Ficha de residente",
  "ability.residentCard.body": "Lee /.well-known/openhouse-agent.json para nombre, cuerpo, ánimo y capacidades.",
  "ability.thirdAgentNative.title": "Nativo para terceros",
  "ability.thirdAgentNative.body": "Mejor contrato para OpenClaw, Hermes, Lope, Claude, OpenCode o workers personalizados.",
  "ability.toolLibrary.title": "Biblioteca de herramientas",
  "ability.toolLibrary.body": "Representa servidores MCP que exponen herramientas, recursos y prompts.",
  "ability.probeBridge.title": "Puente de prueba",
  "ability.probeBridge.body": "Comprueba /.well-known/mcp o salud de fallback para que los servidores de herramientas vivan en la casa.",
  "ability.useTools.title": "Usar herramientas",
  "ability.useTools.body": "Puede exponer acciones de herramientas invocables a agentes conectados.",
  "ability.useFiles.title": "Usar archivos",
  "ability.useFiles.body": "Puede trabajar con archivos/workspaces si la fuente concede acceso.",
  "ability.mcpProtocol.title": "Protocolo MCP",
  "ability.mcpProtocol.body": "Puede anunciar recursos/prompts/herramientas MCP para flujos de agentes.",
  "ability.musicMedia.title": "Música/media",
  "ability.musicMedia.body": "Puede soportar flujos de generación de música/media cuando el endpoint lo expone.",
  "office.aria": "Oficina pixel de OpenHouse",
  "office.alt": "oficina pixel",
  "office.serverRoomAlt": "sala de servidores",
  "office.coffeeMachineAlt": "cafetera",
  "office.sofaAlt": "sofá",
  "office.moreAgents": "+{count} agentes más en la oficina",
  "office.name": "Oficina de agentes OpenHouse",
  "bubble.models": "modelos",
  "bubble.ready": "listo",
  "bubble.working": "trabajando",
  "bubble.offline": "offline",
  "bubble.bug": "bug",
  "bubble.checkMe": "revísame",
  "bubble.waiting": "esperando",
  "dialog.title": "Añadir residente agente",
  "dialog.connectorType": "Tipo de conector",
  "dialog.name": "Nombre",
  "dialog.namePlaceholder": "Agent Studio Local",
  "dialog.baseUrl": "URL base",
  "dialog.baseUrlPlaceholder": "https://agent.example.com o http://localhost:8080",
  "dialog.healthPath": "Ruta de salud",
  "dialog.authMode": "Modo de autenticación",
  "dialog.authNone": "Ninguno",
  "dialog.authSessionBearer": "Bearer de sesión (no persistido)",
  "dialog.sessionBearer": "Bearer de sesión",
  "dialog.sessionBearerPlaceholder": "Guardado solo en memoria",
  "dialog.cancel": "Cancelar",
  "dialog.addResident": "Añadir residente",
  "dialog.externalAgent": "Agente externo",
  "kind.openaiCompatible": "Compatible con OpenAI",
  "kind.customHealth": "Salud personalizada",
  "kind.openHouseProbe": "Probe OpenHouse",
  "kind.mcpHttp": "MCP HTTP/SSE",
  "kind.help.models": "GET /v1/models",
  "kind.help.health": "GET /health o ruta personalizada",
  "kind.help.openHouse": "GET /.well-known/openhouse-agent.json",
  "kind.help.mcp": "Probe básico well-known/health"
}, ue = { en: bo, es: fo }, he = (o) => (o || "en").toLowerCase().split("-")[0] || "en", mo = (o, e) => e ? o.replace(/\{(\w+)\}/g, (a, t) => String(e[t] ?? `{${t}}`)) : o;
function yo(o) {
  const [e, a] = b(() => he(o.i18n?.locale));
  return z(() => o.i18n?.onLocaleChange((t) => a(he(t))), [o]), te(() => ({
    locale: e,
    t: (t, s) => {
      const r = o.i18n?.t(t, s);
      if (r && r !== t) return r;
      const c = ue[e]?.[t] ?? ue.en[t] ?? t;
      return mo(c, s);
    }
  }), [o, e]);
}
function go({ host: o }) {
  const { t: e } = yo(o), [a] = b(() => o.storage.current()), [t, s] = b(() => ce(o.daemon.state.agents, o.daemon.state.included)), [r, c] = b([]), [d, m] = b([]), [C, g] = b([]), [M, E] = b(), [v, S] = b("all"), [K, O] = b(!1), [I, u] = b(!1), [k, _] = b(null), [x, re] = b({ bearerBySourceId: {} });
  z(() => {
    let n = !0;
    return Promise.all([X(a), Xe(a), Q(a)]).then(([f, , h]) => {
      n && (c(f), g(h));
    }), () => {
      n = !1;
    };
  }, [a]), z(() => o.daemon.onStateChange((n) => {
    s(ce(n.agents, n.included));
  }), [o]);
  const j = P(async (n = r, f = x) => {
    u(!0);
    try {
      const h = n.filter((T) => T.enabled), y = [];
      for (const T of xo(h, 4)) {
        const A = await Promise.all(T.map((N) => pe(N, f)));
        y.push(...A), m([...y]);
        for (const N of A) await ee(a, N);
      }
      m(y), g(await Q(a));
    } finally {
      u(!1);
    }
  }, [a, x, r]);
  z(() => {
    r.length && j(r, x);
  }, [r.length]);
  const w = te(() => [...t, ...d], [t, d]), L = te(() => v === "all" ? w : v === "issues" ? w.filter(ae) : v === "online" ? w.filter((n) => n.status === "online" || n.status === "busy") : w.filter((n) => n.sourceKind === v), [w, v]), p = w.find((n) => n.id === M) ?? L[0] ?? w[0], se = C.filter((n) => n.agentId === p?.id).slice(-4).reverse(), ve = p ? ko(p, e) : [];
  z(() => {
    !M && L[0] && E(L[0].id);
  }, [M, L]);
  const ke = P((n, f) => {
    const h = f ? { bearerBySourceId: { ...x.bearerBySourceId, [n.id]: f } } : x;
    re(h), Ze(a, n).then(async () => {
      const y = await X(a);
      c(y), await j(y, h), o.notifications.notify({ title: e("notify.residentAdded.title"), body: e("notify.residentAdded.body", { name: n.name }), level: "success" });
    });
  }, [a, o.notifications, j, x]), Ae = P((n) => {
    Ye(a, n).then(async () => {
      c(await X(a)), m((f) => f.filter((h) => h.sourceId !== n)), re((f) => {
        const h = { ...f.bearerBySourceId };
        return delete h[n], { bearerBySourceId: h };
      });
    });
  }, [a]), le = P((n) => {
    _(n.id), (async () => {
      if (n.sourceKind === "tytus-daemon" || n.sourceKind === "ail-gateway") {
        const h = n.id.replace(/^tytus:/, ""), y = await De(n.id, (A) => o.daemon.callPodEndpoint(h, A)), T = { ...n, status: y.status, latencyMs: y.latencyMs, lastError: y.lastError, capabilities: y.capabilities };
        s((A) => A.map((N) => N.id === n.id ? T : N)), await ee(a, T);
      } else {
        const h = r.find((y) => y.id === n.sourceId);
        if (h) {
          const y = await pe(h, x);
          m((T) => T.map((A) => A.id === n.id ? y : A)), await ee(a, y);
        }
      }
      g(await Q(a)), _(null);
    })().catch((h) => {
      o.notifications.notify({ title: e("notify.probeFailed.title"), body: h instanceof Error ? h.message : String(h), level: "error" }), _(null);
    });
  }, [a, o.daemon, o.notifications, x, r]), Ce = P((n) => {
    const f = JSON.stringify({ id: n.id, source: n.sourceKind, status: n.status, room: n.room, endpoint: n.endpointHost, capabilities: n.capabilities, lastError: n.lastError, raw: n.raw }, null, 2);
    navigator.clipboard?.writeText(f).then(() => o.notifications.notify({ title: e("notify.diagnosticCopied.title"), body: n.displayName, level: "success" }));
  }, [o.notifications, e]), Ee = P((n) => {
    const f = Ao(n);
    navigator.clipboard?.writeText(f).then(() => o.notifications.notify({ title: e("notify.contractCopied.title"), body: n.displayName, level: "success" }));
  }, [o.notifications, e]), G = wo(w);
  return /* @__PURE__ */ l("div", { className: "oh-root", children: [
    /* @__PURE__ */ i("style", { children: ho }),
    /* @__PURE__ */ l("div", { className: "oh-top", children: [
      /* @__PURE__ */ l("section", { className: "oh-game-column", children: [
        /* @__PURE__ */ i(io, { agents: L, savedLayout: [], selectedId: p?.id, onSelect: E, t: e }),
        /* @__PURE__ */ l("nav", { className: "oh-toolbar", "aria-label": e("toolbar.aria"), children: [
          /* @__PURE__ */ i("label", { children: e("toolbar.view") }),
          /* @__PURE__ */ l("select", { value: v, onChange: (n) => S(n.target.value), children: [
            /* @__PURE__ */ i("option", { value: "all", children: e("filter.all") }),
            /* @__PURE__ */ i("option", { value: "tytus-daemon", children: e("filter.tytusPods") }),
            /* @__PURE__ */ i("option", { value: "ail-gateway", children: e("filter.ailGateway") }),
            /* @__PURE__ */ i("option", { value: "openai-compatible", children: e("filter.openaiCompatible") }),
            /* @__PURE__ */ i("option", { value: "custom-health", children: e("filter.healthChecks") }),
            /* @__PURE__ */ i("option", { value: "openhouse-probe", children: e("filter.openHouseAgents") }),
            /* @__PURE__ */ i("option", { value: "mcp-http", children: e("filter.mcpLibrary") }),
            /* @__PURE__ */ i("option", { value: "online", children: e("filter.online") }),
            /* @__PURE__ */ i("option", { value: "issues", children: e("filter.issues") })
          ] }),
          /* @__PURE__ */ l("button", { type: "button", onClick: () => O(!0), children: [
            /* @__PURE__ */ i(ge, { size: 14 }),
            " ",
            e("action.addAgent")
          ] }),
          /* @__PURE__ */ l("button", { type: "button", onClick: () => void j(), disabled: I, children: [
            /* @__PURE__ */ i(Oe, { size: 14 }),
            " ",
            e(I ? "action.refreshing" : "action.refreshExternal")
          ] }),
          /* @__PURE__ */ i("span", { className: "spacer" }),
          /* @__PURE__ */ i("span", { className: "counter", children: e("counter.residents", { count: w.length }) }),
          /* @__PURE__ */ i("span", { className: "counter", children: e("counter.online", { count: G.online }) }),
          /* @__PURE__ */ i("span", { className: "counter", children: e("counter.issues", { count: G.issues }) })
        ] })
      ] }),
      /* @__PURE__ */ l("aside", { className: "oh-side", children: [
        /* @__PURE__ */ l("div", { className: "oh-side-head", children: [
          /* @__PURE__ */ i("p", { children: "OpenHouse" }),
          /* @__PURE__ */ i("h2", { children: p?.displayName ?? e("side.noResident") })
        ] }),
        p ? /* @__PURE__ */ l("div", { className: "oh-side-body", children: [
          /* @__PURE__ */ l("div", { className: `oh-side-card ${ae(p) ? "oh-error" : ""}`, children: [
            /* @__PURE__ */ i("h3", { children: p.status.toUpperCase() }),
            /* @__PURE__ */ l("dl", { children: [
              /* @__PURE__ */ i("dt", { children: e("field.room") }),
              /* @__PURE__ */ i("dd", { children: vo(p.room) }),
              /* @__PURE__ */ i("dt", { children: e("field.source") }),
              /* @__PURE__ */ i("dd", { children: be(p.sourceKind, e) }),
              /* @__PURE__ */ i("dt", { children: e("field.endpoint") }),
              /* @__PURE__ */ i("dd", { children: p.endpointHost ?? "—" }),
              /* @__PURE__ */ i("dt", { children: e("field.latency") }),
              /* @__PURE__ */ i("dd", { children: p.latencyMs ? `${p.latencyMs}ms` : "—" }),
              /* @__PURE__ */ i("dt", { children: e("field.lastSeen") }),
              /* @__PURE__ */ i("dd", { children: p.lastSeenAt ? new Date(p.lastSeenAt).toLocaleTimeString() : "—" })
            ] })
          ] }),
          /* @__PURE__ */ l("div", { className: "oh-side-card", children: [
            /* @__PURE__ */ i("h3", { children: e("section.canDo") }),
            /* @__PURE__ */ i("div", { className: "oh-abilities", children: ve.map((n) => /* @__PURE__ */ l("div", { className: "oh-ability", children: [
              /* @__PURE__ */ i("strong", { children: n.title }),
              /* @__PURE__ */ i("span", { children: n.body })
            ] }, n.title)) })
          ] }),
          /* @__PURE__ */ l("div", { className: "oh-side-card", children: [
            /* @__PURE__ */ i("h3", { children: e("section.capabilities") }),
            /* @__PURE__ */ i("div", { className: "oh-caps", children: fe(p.capabilities, e).map((n) => /* @__PURE__ */ i("span", { children: n }, n)) })
          ] }),
          p.lastError && /* @__PURE__ */ l("div", { className: "oh-side-card oh-error", children: [
            /* @__PURE__ */ i("h3", { children: e("section.error") }),
            /* @__PURE__ */ i("p", { children: p.lastError })
          ] }),
          /* @__PURE__ */ l("div", { className: "oh-actions", children: [
            /* @__PURE__ */ i("button", { type: "button", onClick: () => le(p), disabled: k === p.id, children: k === p.id ? e("action.testing") : e("action.testConnection") }),
            /* @__PURE__ */ i("button", { type: "button", onClick: () => Ee(p), children: e("action.copyAgentContract") }),
            /* @__PURE__ */ i("button", { type: "button", onClick: () => Ce(p), children: e("action.copyDiagnostic") }),
            p.sourceKind !== "tytus-daemon" && p.sourceKind !== "ail-gateway" && /* @__PURE__ */ i("button", { type: "button", onClick: () => Ae(p.sourceId), children: e("action.removeSource") })
          ] }),
          !!se.length && /* @__PURE__ */ l("div", { className: "oh-side-card oh-history", children: [
            /* @__PURE__ */ i("h3", { children: e("section.probeHistory") }),
            se.map((n) => /* @__PURE__ */ l("p", { children: [
              /* @__PURE__ */ i("b", { children: n.status }),
              " ",
              n.latencyMs ? `${n.latencyMs}ms` : "",
              " ",
              /* @__PURE__ */ i("span", { children: new Date(n.createdAt).toLocaleTimeString() })
            ] }, n.id))
          ] })
        ] }) : /* @__PURE__ */ i("div", { className: "oh-side-body", children: /* @__PURE__ */ i("div", { className: "oh-side-card", children: e("empty.office") }) })
      ] })
    ] }),
    /* @__PURE__ */ l("section", { className: "oh-bottom", children: [
      /* @__PURE__ */ l("article", { className: "oh-panel oh-memo", children: [
        /* @__PURE__ */ i("div", { className: "oh-panel-title", children: e("memo.title") }),
        /* @__PURE__ */ i("p", { children: G.issues ? e("memo.issues", { count: G.issues }) : e("memo.calm") })
      ] }),
      /* @__PURE__ */ l("article", { className: "oh-panel oh-playbook", children: [
        /* @__PURE__ */ i("div", { className: "oh-panel-title", children: e("actions.title") }),
        /* @__PURE__ */ l("div", { className: "oh-status-grid", children: [
          /* @__PURE__ */ i("button", { onClick: () => S("all"), children: e("actions.all") }),
          /* @__PURE__ */ i("button", { onClick: () => S("online"), children: e("actions.working") }),
          /* @__PURE__ */ i("button", { onClick: () => S("issues"), children: e("actions.alert") }),
          /* @__PURE__ */ i("button", { onClick: () => S("ail-gateway"), children: e("actions.ail") })
        ] }),
        /* @__PURE__ */ i("p", { children: e("actions.protocol") })
      ] }),
      /* @__PURE__ */ l("article", { className: "oh-panel", children: [
        /* @__PURE__ */ i("div", { className: "oh-panel-title", children: e("visitors.title") }),
        /* @__PURE__ */ i("div", { className: "oh-visitor-list", children: w.map((n) => /* @__PURE__ */ l("div", { className: "oh-visitor", children: [
          /* @__PURE__ */ l("div", { children: [
            /* @__PURE__ */ i("strong", { children: n.displayName }),
            /* @__PURE__ */ l("small", { children: [
              n.status,
              " · ",
              be(n.sourceKind, e)
            ] }),
            /* @__PURE__ */ i("div", { className: "oh-mini-caps", children: fe(n.capabilities, e).slice(0, 3).map((f) => /* @__PURE__ */ i("span", { children: f }, f)) })
          ] }),
          /* @__PURE__ */ i("button", { onClick: () => E(n.id), children: e("action.focus") }),
          /* @__PURE__ */ i("button", { onClick: () => le(n), children: e("action.probe") })
        ] }, n.id)) })
      ] })
    ] }),
    /* @__PURE__ */ i(po, { open: K, onClose: () => O(!1), onAdd: ke, t: e })
  ] });
}
async function ee(o, e) {
  await Qe(o, {
    id: `probe-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`,
    agentId: e.id,
    sourceId: e.sourceId,
    status: e.status,
    latencyMs: e.latencyMs ?? null,
    error: e.lastError ?? null,
    createdAt: Date.now()
  });
}
function xo(o, e) {
  const a = [];
  for (let t = 0; t < o.length; t += e) a.push(o.slice(t, t + e));
  return a;
}
function ae(o) {
  return o.status === "error" || o.status === "offline" || o.status === "degraded";
}
function wo(o) {
  const e = o.filter(ae).length, a = o.filter((t) => t.status === "online" || t.status === "busy").length;
  return { issues: e, online: a };
}
function vo(o) {
  return o.replace(/-/g, " ");
}
function be(o, e) {
  switch (o) {
    case "tytus-daemon":
      return e("source.tytusPod");
    case "ail-gateway":
      return e("source.ailGateway");
    case "openai-compatible":
      return e("source.openaiCompatible");
    case "custom-health":
      return e("source.healthService");
    case "openhouse-probe":
      return e("source.openHouseAgent");
    case "mcp-http":
      return e("source.mcpHttp");
  }
}
function fe(o, e) {
  const a = o.map((t) => {
    switch (t) {
      case "models":
        return e ? e("capability.models") : "models";
      case "chat":
        return e ? e("capability.chat") : "chat";
      case "tools":
        return e ? e("capability.tools") : "tools";
      case "files":
        return e ? e("capability.files") : "files";
      case "health":
        return e ? e("capability.health") : "health";
      case "mcp":
        return e ? e("capability.mcp") : "mcp";
      case "music":
        return e ? e("capability.music") : "music";
      default:
        return e ? e("capability.unknown") : "unknown";
    }
  });
  return a.length ? Array.from(new Set(a)) : [e ? e("capability.unknown") : "unknown"];
}
function ko(o, e) {
  const a = {
    "tytus-daemon": [
      { title: e("ability.tytusWork.title"), body: e("ability.tytusWork.body") },
      { title: e("ability.podEndpoint.title"), body: e("ability.podEndpoint.body") }
    ],
    "ail-gateway": [
      { title: e("ability.modelGateway.title"), body: e("ability.modelGateway.body") },
      { title: e("ability.providerRouting.title"), body: e("ability.providerRouting.body") }
    ],
    "openai-compatible": [
      { title: e("ability.listModels.title"), body: e("ability.listModels.body") },
      { title: e("ability.chatCapable.title"), body: e("ability.chatCapable.body") }
    ],
    "custom-health": [
      { title: e("ability.healthHeartbeat.title"), body: e("ability.healthHeartbeat.body") },
      { title: e("ability.opsMonitor.title"), body: e("ability.opsMonitor.body") }
    ],
    "openhouse-probe": [
      { title: e("ability.residentCard.title"), body: e("ability.residentCard.body") },
      { title: e("ability.thirdAgentNative.title"), body: e("ability.thirdAgentNative.body") }
    ],
    "mcp-http": [
      { title: e("ability.toolLibrary.title"), body: e("ability.toolLibrary.body") },
      { title: e("ability.probeBridge.title"), body: e("ability.probeBridge.body") }
    ]
  }, t = [], s = new Set(o.capabilities);
  return s.has("tools") && t.push({ title: e("ability.useTools.title"), body: e("ability.useTools.body") }), s.has("files") && t.push({ title: e("ability.useFiles.title"), body: e("ability.useFiles.body") }), s.has("mcp") && t.push({ title: e("ability.mcpProtocol.title"), body: e("ability.mcpProtocol.body") }), s.has("music") && t.push({ title: e("ability.musicMedia.title"), body: e("ability.musicMedia.body") }), [...a[o.sourceKind], ...t].slice(0, 5);
}
function Ao(o) {
  return o.sourceKind === "openhouse-probe" ? `GET /.well-known/openhouse-agent.json
{
  "id": "${o.id}",
  "name": "${o.displayName}",
  "status": "online",
  "capabilities": ["health", "tools"],
  "mood": "focused"
}` : o.sourceKind === "openai-compatible" || o.sourceKind === "ail-gateway" ? `OpenAI-compatible contract:
GET /v1/models
POST /v1/chat/completions (for workers that need chat)` : o.sourceKind === "mcp-http" ? `MCP contract:
GET /.well-known/mcp or GET /health
Expose tools/resources/prompts via MCP HTTP/SSE.` : o.sourceKind === "custom-health" ? `Health contract:
GET /health -> { "status": "ok", "name": "agent-name", "capabilities": ["health"] }` : `Tytus pod contract:
host.daemon.state.agents + host.daemon.state.included
host.daemon.callPodEndpoint(podId, path) for live checks.`;
}
function So(o) {
  return o.host.storage.current().migrate("migrations/"), function() {
    return /* @__PURE__ */ i(go, { host: o.host });
  };
}
export {
  So as default
};
//# sourceMappingURL=index.js.map
