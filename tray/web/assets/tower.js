(() => {
  'use strict';

  const $ = (id) => document.getElementById(id);

  // ── Client-side display overrides ─────────────────────────────
  // The public catalog (Provider) is conservative — it keeps copy
  // short and API-like for non-UI consumers (`tytus agent catalog`,
  // future third-party clients). The install WIZARD is the product
  // surface, so we enrich cards here with tagline, feature bullets,
  // and bundled icons. Adding a new agent means editing THIS map +
  // dropping an SVG into web/assets/icons/<id>.svg — then the public
  // catalog entry can stay minimal.
  const DISPLAY = {
    nemoclaw: {
      icon: '/assets/icons/openclaw.svg',
      display_name: 'OpenClaw',
      tagline: 'Your personal AI on every channel you already use',
      description:
        'Chat with your assistant from Telegram, WhatsApp, Signal, ' +
        'Discord, Slack, iMessage, Matrix and 20+ more — all from one ' +
        'always-on brain. Voice in, voice out. Live Canvas you can ' +
        'point at and steer.',
      highlights: [
        '25+ messaging channels — one assistant, every app',
        'Voice + Canvas — speak, listen, and watch it draw',
        'Hardened by NVIDIA NemoClaw sandbox for extra safety',
        'Always-on — ready whenever you need it',
      ],
      homepage: 'https://openclaw.ai/',
      github: 'https://github.com/openclaw/openclaw',
    },
    hermes: {
      icon: '/assets/icons/hermes.svg',
      display_name: 'Hermes',
      tagline: 'The self-improving AI agent that gets smarter as you use it',
      description:
        'Learns from every task. Creates its own skills, remembers ' +
        'every past conversation, runs scheduled jobs, and splits ' +
        'complex work across parallel subagents. Talk to it from ' +
        'your terminal, Telegram, Discord, or Slack.',
      highlights: [
        'Self-improving skills — sharper every time you use them',
        'Total recall — searches every past conversation',
        'Scheduled automations — daily reports, nightly audits',
        'Parallel subagents — splits big jobs into concurrent work',
      ],
      homepage: 'https://hermes-agent.nousresearch.com/',
      github: 'https://github.com/NousResearch/hermes-agent',
    },
  };

  const view = {
    show(name) {
      for (const v of ['chooser', 'installing', 'success', 'failure']) {
        const el = $(v);
        if (el) el.classList.toggle('hidden', v !== name);
      }
      document.body.dataset.view = name;
    },
  };

  // ── Hash deep-link auto-run ───────────────────────────────────
  //
  // Tray menu items deep-link into Tower at hashes like
  // `#/run/doctor` or `#/pod/02/restart` instead of spawning a
  // Terminal. We pick those up here, wait for `state-ready` (so
  // pod panels exist), then trigger the matching in-page action.
  //
  // Each tray click appends `?n=<nonce>` to force `hashchange` to
  // fire even when the fragment matches the previous one — without
  // it, browsers focus the tab without re-running our handler.
  let __stateReady = false;
  let __pendingHash = location.hash;
  function __runHashAction(hash) {
    if (!hash) return;
    const route = hash.replace(/^#\/?/, '').split('?')[0];
    if (!route) return;
    const parts = route.split('/');
    if (parts[0] === 'run' && parts[1]) {
      // Buttons inside collapsed <details> need their parent opened
      // first — otherwise the panel they reveal sits invisibly inside
      // a closed disclosure. Map of button-id → optional details-id
      // to open before click.
      const buttonMap = {
        doctor: { btn: 'tr-doctor', openDetails: 'troubleshoot' },
        test:   { btn: 'hdr-health' }, // header button, no parent details
      };
      const route = buttonMap[parts[1]];
      if (route) {
        if (route.openDetails) {
          const d = document.getElementById(route.openDetails);
          if (d) d.open = true;
        }
        document.getElementById(route.btn)?.click();
        return;
      }
      if (parts[1] === 'channels-catalog') {
        __runChannelsCatalogInline();
        return;
      }
    }
    if (parts[0] === 'pod' && parts[1] && parts[2]) {
      // Phase B routes this into the per-pod subpage. For Phase A we
      // hit the existing /api/pod/<action> endpoints directly so the
      // tray rewires deliver value before Phase B lands. When Phase B
      // is loaded its listener calls preventDefault() to suppress the
      // fallback and avoid duplicate POSTs.
      const pod = parts[1];
      const action = parts[2];
      const params = new URLSearchParams(hash.split('?')[1] || '');
      const evt = new CustomEvent('pod-hash-action',
        { detail: { pod, action, params }, cancelable: true });
      window.dispatchEvent(evt);
      if (!evt.defaultPrevented) __runPhaseAFallback(pod, action);
      return;
    }
  }
  async function __runPhaseAFallback(pod, action) {
    // Defense-in-depth: this only runs if no `pod-hash-action`
    // listener called preventDefault on the dispatched event.
    // Phase B always registers such a listener (later in this same
    // script) and always calls preventDefault, so in normal operation
    // this fallback is unreachable. It exists in case a future
    // refactor accidentally drops Phase B's listener — the user still
    // gets the action triggered via the legacy fire-and-forget
    // endpoints.
    //
    // No confirm() calls here on purpose. If this fallback IS
    // reached, Phase B is missing entirely; we don't have its
    // confirmation UX either way. A bare POST without confirm is the
    // historical tray behavior (the prior Terminal-spawn flow had no
    // confirm step), so this preserves that contract for the
    // degenerate case.
    try {
      switch (action) {
        case 'restart': {
          await fetch(`/api/pod/restart?pod=${encodeURIComponent(pod)}`, { method: 'POST' });
          showToast(`Pod ${pod} restarting…`);
          break;
        }
        case 'stop-forwarder': {
          await fetch(`/api/pod/stop-forwarder?pod=${encodeURIComponent(pod)}`, { method: 'POST' });
          showToast(`Pod ${pod} forwarder stopping…`);
          break;
        }
        case 'open': {
          await fetch(`/api/pod/open?pod=${encodeURIComponent(pod)}`, { method: 'POST' });
          break;
        }
        // Intentionally absent: revoke, uninstall, doctor.
        //   revoke / uninstall — destructive. Without Phase B's
        //     confirm() there is no safe fallback. We'd rather no-op
        //     than silently delete a pod's workspace.
        //   doctor — has no fire-and-forget endpoint (only the
        //     streamed /api/pod/NN/run-streamed); a bare POST here
        //     would 404. Phase B owns it.
        // Channels actions land on the channels strip; Phase B opens
        // the dedicated subpage. For Phase A the strip auto-renders
        // when its parent panel is visible, so a no-op is fine.
        default: break;
      }
      // Refresh state so the running-session badge / pod URL reflects
      // the new lifecycle state. /api/state isn't auto-polled.
      loadBudget();
    } catch (err) {
      showToast(`Action failed: ${err}`, 'err');
    }
  }
  async function __runChannelsCatalogInline() {
    // Opportunistically reuse the existing #doctor-panel for output —
    // it's already styled and has a close button. The Troubleshoot
    // <details> needs to be open for the panel to be visible.
    const trouble = document.getElementById('troubleshoot');
    if (trouble) trouble.open = true;
    const panel = $('doctor-panel');
    const title = $('doctor-panel-title');
    const log = $('doctor-panel-log');
    if (!panel || !title || !log) return;
    panel.classList.remove('hidden', 'ok', 'err');
    title.textContent = 'Loading channels catalog…';
    log.textContent = '';
    try {
      const res = await fetch('/api/channels/catalog', { method: 'POST' });
      const body = await res.json();
      if (body.error) {
        panel.classList.add('err');
        title.textContent = 'Channels catalog failed to load';
        log.textContent = body.error;
      } else {
        panel.classList.add(body.ok ? 'ok' : 'err');
        title.textContent = body.ok
          ? 'Channels catalog'
          : `Channels catalog (exit ${body.exit_code})`;
        log.textContent =
          (body.stdout || '') +
          (body.stderr ? `\n\n[stderr]\n${body.stderr}` : '');
      }
    } catch (err) {
      panel.classList.add('err');
      title.textContent = 'Channels catalog errored';
      log.textContent = String(err);
    }
  }
  // ── Phase C: token modal ──────────────────────────────────────
  //
  // Opens a native <dialog> for collecting a channel bot token
  // without spawning Terminal. Submit POSTs JSON to
  // /api/channels/add — the token rides only the request body and
  // never touches the URL bar, address history, or browser logs.
  function openTokenModal(podId, channel, channelLabel, onDone) {
    const dlg = $('token-modal');
    if (!dlg || typeof dlg.showModal !== 'function') {
      // Defensive — older browsers without <dialog>; fall back to a
      // prompt rather than refusing the action entirely.
      const tok = window.prompt(`Paste your ${channelLabel} bot token:`);
      if (!tok) return;
      __submitToken(podId, channel, tok, channelLabel, onDone);
      return;
    }
    const titleEl = $('token-modal-title');
    const hintEl  = $('token-modal-hint');
    const input   = $('token-input');
    const errEl   = $('token-error');
    const cancel  = $('token-cancel');
    const form    = $('token-form');
    titleEl.textContent = `Add ${channelLabel} to pod ${podId}`;
    hintEl.textContent =
      `Paste your ${channelLabel} bot token. It's sent only to your local ` +
      `Tytus process on 127.0.0.1, then forwarded to the pod over TLS.`;
    input.value = '';
    errEl.textContent = '';
    errEl.classList.add('hidden');

    const close = () => {
      try { dlg.close(); } catch {}
      form.onsubmit = null;
      cancel.onclick = null;
    };
    cancel.onclick = (e) => { e.preventDefault(); close(); };
    form.onsubmit = async (e) => {
      e.preventDefault();
      const token = input.value.trim();
      if (!token) {
        errEl.textContent = 'Token is empty.';
        errEl.classList.remove('hidden');
        return;
      }
      const submit = $('token-submit');
      submit.disabled = true;
      const prev = submit.textContent;
      submit.textContent = 'Adding…';
      try {
        const res = await __submitTokenFetch(podId, channel, token);
        if (res.ok) {
          close();
          showToast(`${channelLabel} added to pod ${podId}.`);
          if (typeof onDone === 'function') onDone();
        } else {
          errEl.textContent = res.error ||
            (res.exit_code !== undefined ? `Failed (exit ${res.exit_code}).` : 'Failed.');
          if (res.stderr) errEl.textContent += `\n${res.stderr}`;
          errEl.classList.remove('hidden');
        }
      } catch (ex) {
        errEl.textContent = String(ex);
        errEl.classList.remove('hidden');
      } finally {
        submit.disabled = false;
        submit.textContent = prev;
      }
    };
    dlg.showModal();
    setTimeout(() => input.focus(), 0);
  }
  async function __submitTokenFetch(podId, channel, token) {
    const r = await fetch('/api/channels/add', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pod: podId, channel, token }),
    });
    return r.json();
  }
  async function __submitToken(podId, channel, token, label, onDone) {
    try {
      const r = await __submitTokenFetch(podId, channel, token);
      if (r.ok) {
        showToast(`${label} added to pod ${podId}.`);
        if (typeof onDone === 'function') onDone();
      } else {
        showToast(r.error || `Failed (exit ${r.exit_code})`, 'err');
      }
    } catch (e) {
      showToast(String(e), 'err');
    }
  }

  window.addEventListener('state-ready', () => {
    __stateReady = true;
    if (__pendingHash) {
      const h = __pendingHash;
      __pendingHash = null;
      __runHashAction(h);
    }
  });
  window.addEventListener('hashchange', () => {
    if (__stateReady) __runHashAction(location.hash);
    else __pendingHash = location.hash;
  });

  // ── Phase B: per-pod subpage (viewPod) ────────────────────────
  //
  // Hash routes:
  //   #/pod/<NN>             → viewPod.show(NN, 'overview')
  //   #/pod/<NN>/<tab>       → viewPod.show(NN, <tab>)  tab ∈ overview|output|channels
  //   #/pod/<NN>/<action>    → viewPod.show(NN, 'output'); run action streamed
  //                            (action ∈ restart|revoke|uninstall|stop-forwarder)
  //
  // Global commands `doctor` and `test` are intentionally NOT in the
  // per-pod action set — they aren't pod-scoped (CLI takes no --pod
  // flag for them), so they live on `#/run/doctor` / `#/run/test`
  // which target the global Tower handlers.
  //
  // body.pod-mode is the visibility switch (CSS). Install flow's
  // view.show() is unaffected — it manages chooser/installing/success/
  // failure within the (now-hidden) overview stack.
  const viewPod = (() => {
    const TABS = ['overview', 'output', 'channels'];
    // Actions that mount the Output tab and run a streamed subprocess.
    // `doctor` and `test` are intentionally excluded — they're global
    // commands (no --pod flag in the CLI), so they live on the Tower
    // header / Troubleshoot section, not as per-pod actions.
    const ACTION_TABS = new Set(['restart', 'revoke', 'uninstall', 'stop-forwarder']);
    let currentPod = null;
    let currentTab = null;
    let activeStream = null; // { es, jobId, podId }

    function show(pod, tabOrAction, params) {
      currentPod = pod;
      document.body.classList.add('pod-mode');
      // Make sure we're not stuck inside a sub-flow.
      const dest = (tabOrAction && ACTION_TABS.has(tabOrAction))
        ? 'output' : (tabOrAction || 'overview');
      switchTab(dest);
      renderHeader(pod);
      if (dest === 'overview') renderOverview(pod);
      if (dest === 'channels') renderChannels(pod, params);
      if (dest === 'output' && tabOrAction && ACTION_TABS.has(tabOrAction)) {
        // Phase B's hash-driven action runner. Confirm destructive
        // actions before kicking off the streamed subprocess.
        runStreamedAction(pod, tabOrAction);
      }
    }
    function hide() {
      document.body.classList.remove('pod-mode');
      currentPod = null;
      currentTab = null;
      // Don't tear down the EventSource — let the user revisit and see
      // accumulated output. Page reload will reap.
    }
    function switchTab(tab) {
      if (!TABS.includes(tab)) tab = 'overview';
      currentTab = tab;
      for (const t of TABS) {
        const pane = document.getElementById(`pod-tab-${t}`);
        const btn  = document.getElementById(`pod-tab-${t}-btn`);
        if (pane) pane.classList.toggle('hidden', t !== tab);
        if (btn)  btn.classList.toggle('tab-active', t === tab);
      }
    }
    function findPod(pod) {
      const s = budgetState || {};
      const list = (s.agents || []).concat(s.included || []);
      return list.find((a) => a.pod_id === pod) || null;
    }
    function renderHeader(pod) {
      const a = findPod(pod);
      const titleEl = document.getElementById('pod-sub-name');
      const iconEl  = document.getElementById('pod-sub-icon');
      if (iconEl) iconEl.innerHTML = '';
      if (a) {
        const override = DISPLAY[a.agent_type] || {};
        const name = override.display_name || a.agent_type || 'Pod';
        if (titleEl) titleEl.textContent = `${name} — Pod ${pod}`;
        if (override.icon && iconEl) {
          const img = document.createElement('img');
          img.src = override.icon; img.alt = '';
          iconEl.appendChild(img);
        }
      } else {
        if (titleEl) titleEl.textContent = `Pod ${pod}`;
      }
    }
    function renderOverview(pod) {
      const host = document.getElementById('pod-tab-overview');
      if (!host) return;
      const a = findPod(pod);
      if (!a) {
        host.innerHTML = `<p class="muted">Pod ${pod} not found in current state.</p>`;
        return;
      }
      const apiUrl = a.api_url || a.public_url || '— provisioning —';
      host.innerHTML = `
        <div class="pod-url-row">
          <span class="pod-url-label">API URL</span>
          <code class="pod-url"></code>
        </div>
        <div class="pod-actions" style="margin-top:12px"></div>
      `;
      host.querySelector('.pod-url').textContent = apiUrl;
      const actions = host.querySelector('.pod-actions');
      const mkBtn = (label, cls, onClick) => {
        const b = document.createElement('button');
        b.type = 'button';
        b.className = `pod-btn ${cls}`;
        b.textContent = label;
        b.addEventListener('click', onClick);
        return b;
      };
      if (a.public_url) {
        actions.appendChild(mkBtn('Open in Browser', 'pod-btn-primary', async () => {
          await fetch(`/api/pod/open?pod=${encodeURIComponent(pod)}`, { method: 'POST' });
        }));
      }
      actions.appendChild(mkBtn('Copy API URL', 'pod-copy-url', () => {
        if (a.api_url) {
          navigator.clipboard?.writeText(a.api_url);
          showToast('API URL copied');
        }
      }));
      actions.appendChild(mkBtn('Output ▸', 'pod-btn', () => {
        location.hash = `#/pod/${pod}/output`;
      }));
      actions.appendChild(mkBtn('Channels ▸', 'pod-btn', () => {
        location.hash = `#/pod/${pod}/channels`;
      }));
    }
    function renderChannels(pod, params) {
      const host = document.getElementById('pod-tab-channels');
      if (!host) return;
      // Reuse the existing helper that renders the channels strip.
      renderPodChannels(host, pod);
      // Phase C: tray menu deep-link `?action=add&type=X` opens the
      // token modal directly. Same shape `?action=remove&type=X` is
      // handled inside renderPodChannels (confirm + fetch).
      if (params && typeof params.get === 'function') {
        const action = params.get('action');
        const type = params.get('type');
        if (action === 'add' && type) {
          const label = (CHANNELS.find((c) => c.id === type) || {}).label || type;
          openTokenModal(pod, type, label, () => {
            setTimeout(() => renderPodChannels(host, pod), 20000);
          });
        }
      }
    }

    function streamForPod(pod, jobId, statusEl, onExit) {
      // Tear down any existing stream — only one per pod at a time.
      if (activeStream) {
        try { activeStream.es.close(); } catch {}
        activeStream = null;
      }
      const logEl = document.getElementById('pod-output-log');
      if (statusEl) {
        statusEl.classList.remove('ok', 'err');
        statusEl.textContent = 'Streaming…';
      }
      const es = new EventSource(`/api/jobs/${encodeURIComponent(jobId)}/stream`);
      activeStream = { es, jobId, podId: pod };
      es.addEventListener('log', (ev) => {
        const line = (ev.data || '').replace(/\\n/g, '\n');
        logEl.textContent += line + '\n';
        logEl.scrollTop = logEl.scrollHeight;
      });
      // Local handle so we can detect "this stream got superseded by
      // a newer one before its exit handler ran" — see refreshIfMine.
      const myStream = activeStream;
      const refreshIfMine = () => {
        // Only refresh budget when this is still the live stream; a
        // superseded stream's exit must NOT call loadBudget because a
        // transient fetch failure there would null budgetState while
        // the new stream is still rendering — UI flicker.
        if (activeStream === myStream || activeStream === null) loadBudget();
      };
      es.addEventListener('exit', (ev) => {
        let code = -1;
        try { code = (JSON.parse(ev.data || '{}').code) ?? -1; } catch {}
        if (statusEl) {
          statusEl.classList.add(code === 0 ? 'ok' : 'err');
          statusEl.textContent = code === 0
            ? `Done (exit 0).`
            : `Failed (exit ${code}).`;
        }
        es.close();
        if (activeStream === myStream) activeStream = null;
        // Refresh budget so the running-session dot clears and the
        // pod state reflects any lifecycle change (revoke/uninstall).
        refreshIfMine();
        // Caller-supplied post-exit hook (e.g. revoke navigates back
        // to Tower on success). Fired only for the live stream — a
        // superseded stream's onExit must not run, otherwise a stale
        // revoke handler could redirect the user away from a pod
        // they navigated to fresh.
        if (typeof onExit === 'function' && (activeStream === null || activeStream === myStream)) {
          try { onExit(code); } catch {}
        }
      });
      es.addEventListener('fail', (ev) => {
        if (statusEl) {
          statusEl.classList.add('err');
          statusEl.textContent = `Job failed: ${ev.data || 'unknown error'}`;
        }
        es.close();
        if (activeStream === myStream) activeStream = null;
        refreshIfMine();
      });
      es.onerror = () => {
        if (statusEl && !statusEl.textContent.startsWith('Done')) {
          statusEl.classList.add('err');
          statusEl.textContent = 'Stream lost.';
        }
        es.close();
        if (activeStream === myStream) activeStream = null;
      };
    }

    async function runStreamedAction(pod, action) {
      // Switch to Output tab so the user sees the stream.
      switchTab('output');
      const logEl = document.getElementById('pod-output-log');
      const statusEl = document.getElementById('pod-output-status');
      // Confirm destructive actions in-page (tray no longer confirms).
      if (action === 'revoke') {
        const ok = window.confirm(
          `Revoke pod ${pod}?\n\nFrees its units and wipes the agent's workspace. Cannot be undone.`);
        if (!ok) {
          if (statusEl) {
            statusEl.classList.remove('ok', 'err');
            statusEl.textContent = 'Cancelled.';
          }
          return;
        }
      }
      if (action === 'uninstall') {
        const ok = window.confirm(
          `Uninstall the agent on pod ${pod}?\n\nThe pod slot stays allocated; AIL keeps working.`);
        if (!ok) {
          if (statusEl) {
            statusEl.classList.remove('ok', 'err');
            statusEl.textContent = 'Cancelled.';
          }
          return;
        }
      }
      // Status hint while the POST is in flight; the log header is
      // written ONLY after we get a 202 + job_id back. That avoids a
      // misleading `$ tytus …` line preceding a 409 'pod busy'
      // response (where no subprocess actually ran).
      if (statusEl) {
        statusEl.classList.remove('ok', 'err');
        statusEl.textContent = 'Starting…';
      }
      try {
        const res = await fetch(
          `/api/pod/${encodeURIComponent(pod)}/run-streamed`,
          {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ action }),
          });
        const body = await res.json();
        if (res.status === 409) {
          if (statusEl) {
            statusEl.classList.add('err');
            statusEl.textContent = body.error || 'Pod is busy.';
          }
          return;
        }
        if (!res.ok || !body.job_id) {
          if (statusEl) {
            statusEl.classList.add('err');
            statusEl.textContent = body.error || 'Failed to start.';
          }
          return;
        }
        // Subprocess is starting — now safe to show the command header
        // in the log. Append (don't replace) so a fresh action keeps
        // any prior output visible until the user explicitly clears.
        if (logEl) logEl.textContent += `$ tytus ${action} (pod ${pod})\n`;
        // Refresh state so overview pod-rows show the running dot for
        // this pod while the stream runs.
        loadBudget();
        // Q3: revoke removes the pod from /api/state.agents on
        // success. Auto-navigate back to Tower when its exit code is
        // 0 — leaving the user on `#/pod/<NN>/output` would show a
        // 'Pod not found' screen which is disorienting. We pass a
        // post-exit hook to streamForPod for this.
        const onExit = (action === 'revoke')
          ? (code) => {
              if (code === 0) {
                // Small delay so the user sees the success line before
                // the page-context flips.
                setTimeout(() => { location.hash = ''; }, 1200);
              }
            }
          : null;
        streamForPod(pod, body.job_id, statusEl, onExit);
      } catch (err) {
        if (statusEl) {
          statusEl.classList.add('err');
          statusEl.textContent = `Failed: ${err}`;
        }
      }
    }

    return { show, hide, switchTab, runStreamedAction };
  })();

  // Wire pod-back button + tab clicks (resolved at first click since
  // the sections might mount lazily — but in this build they're all
  // baked into tower.html, so direct binding is safe).
  document.getElementById('pod-back')?.addEventListener('click', (e) => {
    e.preventDefault();
    location.hash = '';
  });
  for (const t of ['overview', 'output', 'channels']) {
    document.getElementById(`pod-tab-${t}-btn`)?.addEventListener('click', (e) => {
      e.preventDefault();
      const pod = (location.hash.match(/^#\/pod\/([0-9]+)/) || [])[1];
      if (pod) location.hash = `#/pod/${pod}/${t}`;
    });
  }
  // Output toolbar buttons → run streamed actions on the current pod.
  // Only per-pod actions appear here; global commands (doctor/test)
  // live on the Tower header and inside Troubleshoot.
  for (const [btnId, action] of [
    ['pod-run-restart', 'restart'],
    ['pod-run-stop-fwd', 'stop-forwarder'],
    ['pod-run-uninstall', 'uninstall'],
    ['pod-run-revoke', 'revoke'],
  ]) {
    document.getElementById(btnId)?.addEventListener('click', () => {
      const pod = (location.hash.match(/^#\/pod\/([0-9]+)/) || [])[1];
      if (pod) viewPod.runStreamedAction(pod, action);
    });
  }
  document.getElementById('pod-output-clear')?.addEventListener('click', () => {
    const log = document.getElementById('pod-output-log');
    if (log) log.textContent = '';
  });

  // Extend the hash router: pod routes route through viewPod.
  window.addEventListener('pod-hash-action', (e) => {
    // Suppress the Phase A fallback fetch — viewPod owns the action now.
    e.preventDefault();
    const { pod, action, params } = e.detail || {};
    if (pod) viewPod.show(pod, action, params);
  });
  // Mount the pod view for the bare `#/pod/<NN>` route (no segment).
  // Routes WITH a segment — both tabs (`/overview`, `/output`,
  // `/channels`) and actions (`/restart`, `/revoke`, `/uninstall`,
  // `/stop-forwarder`, `/doctor`) — are claimed by the
  // `pod-hash-action` listener above. That listener fires
  // synchronously on the same hashchange, so we let it handle every
  // seg case to avoid redundant `viewPod.show()` renders. Action
  // segments doubly so: a duplicate mount would `runStreamedAction`
  // twice and the second POST would hit 409 Conflict from the
  // Registry::create_pod busy-check.
  //
  // Also handles the leave-pod-mode case: hash no longer matches a
  // pod route → hide #view-pod and fall back to the overview stack.
  function __maybeMountPodView() {
    const m = location.hash.match(/^#\/pod\/([0-9]+)(?:\/([a-zA-Z-]+))?/);
    if (m) {
      // Seg present (tab or action) → owned by pod-hash-action.
      if (m[2]) return;
      viewPod.show(m[1], 'overview');
    } else if (document.body.classList.contains('pod-mode')) {
      viewPod.hide();
    }
  }
  window.addEventListener('hashchange', __maybeMountPodView);
  window.addEventListener('state-ready', __maybeMountPodView);

  // Cached budget snapshot — populated by loadBudget(), read by
  // renderCatalog() to mark cards enabled/disabled against the
  // remaining unit headroom. We fetch it on page load in parallel
  // with the catalog; install success triggers a re-fetch so the
  // bar updates without a page reload.
  let budgetState = null;
  // Pod-ids (as strings) that currently have a live localhost UI
  // forwarder. Drives the per-pod "Stop forwarder" button visibility.
  // Refreshed on every /api/state fetch via loadBudget().
  const POD_FORWARDERS = new Set();

  function formatUptime(secs) {
    if (!secs || secs <= 0) return null;
    const d = Math.floor(secs / 86400);
    const h = Math.floor((secs % 86400) / 3600);
    const m = Math.floor((secs % 3600) / 60);
    if (d > 0) return `up ${d}d ${h}h`;
    if (h > 0) return `up ${h}h ${m}m`;
    return `up ${m}m`;
  }

  /// Wave 4: render the status-line under the brand + the
  /// keychain-warning banner + the Troubleshoot last-refresh-error row
  /// + the Footer About panel. Called on every /api/state refresh so
  /// all four surfaces stay in sync with the daemon's live view.
  function renderStatusSignals(state) {
    // Sub-header: "signed in as <email> · up Xh Ym · daemon pid N"
    const statusLine = $('status-line');
    if (statusLine) {
      const bits = [];
      if (state.logged_in && state.email) {
        bits.push(`Signed in as ${state.email}`);
      }
      const up = formatUptime(state.uptime_secs);
      if (up) bits.push(up);
      if (state.daemon_running && state.daemon_pid) {
        bits.push(`daemon pid ${state.daemon_pid}`);
      } else if (state.logged_in && !state.daemon_running) {
        bits.push('daemon offline');
      }
      statusLine.innerHTML = bits
        .map((b, i) => i > 0 ? `<span class="sep">·</span>${b}` : b)
        .join('');
    }

    // Keychain banner: sticky warning when `!keychain_healthy`. The
    // data plane still works in this state — don't panic the user —
    // but the next `tytus login` is required before the current access
    // token expires. Matches the tray's ⚠︎ row in the Troubleshoot
    // submenu + metadata hint.
    const banner = $('banner');
    if (banner) {
      if (state.logged_in && state.keychain_healthy === false) {
        banner.className = 'banner warn';
        banner.classList.remove('hidden');
        banner.innerHTML = `
          <span class="banner-icon">⚠︎</span>
          <div class="banner-body">
            <strong>Keychain access pending.</strong>
            The OS keychain hasn't yielded the refresh token (pending dialog or stale ACL).
            The data plane still works, but re-run Sign In before your current token expires.
          </div>
          <button class="banner-action" type="button" id="banner-login">Sign In…</button>
        `;
        banner.querySelector('#banner-login')?.addEventListener('click', async () => {
          await fetch('/api/connect', { method: 'POST' }); // opens Terminal for tytus login too on first click
          showToast('Check the Terminal window.');
        });
      } else {
        banner.classList.add('hidden');
        banner.innerHTML = '';
      }
    }

    // Last refresh error — only render when present. Sits at the top
    // of the Troubleshoot body for visibility.
    const errRow = $('tr-refresh-err');
    const errText = $('tr-refresh-err-text');
    if (errRow && errText) {
      if (state.last_refresh_error) {
        errText.textContent = state.last_refresh_error;
        errRow.classList.remove('hidden');
      } else {
        errRow.classList.add('hidden');
        errText.textContent = '';
      }
    }

    // Footer About panel — populated even when collapsed, so a reveal
    // click doesn't require a second round-trip.
    const aboutBundle = $('about-bundle');
    if (aboutBundle) {
      aboutBundle.textContent = state.app_bundle_installed
        ? '/Applications/Tytus.app'
        : 'not installed';
    }
    const aboutDaemon = $('about-daemon');
    if (aboutDaemon) {
      aboutDaemon.textContent = state.daemon_running
        ? `running (pid ${state.daemon_pid || '?'})`
        : 'stopped';
    }
    const aboutEmail = $('about-email');
    if (aboutEmail) {
      aboutEmail.textContent = state.email || '—';
    }
  }

  async function loadBudget() {
    try {
      const res = await fetch('/api/state');
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      budgetState = await res.json();
      POD_FORWARDERS.clear();
      for (const p of (budgetState.forwarders || [])) POD_FORWARDERS.add(p);
      renderBudget(budgetState);
      renderStatusSignals(budgetState);
    } catch (err) {
      // Soft fail: hide the strip, keep cards enabled. The subprocess
      // call in the Rust handler can time out if the CLI is missing
      // or Sentinel is unreachable — don't block installs on it.
      $('budget').classList.add('hidden');
      budgetState = null;
    }
  }

  function renderBudget(state) {
    const el = $('budget');
    if (!state || !state.logged_in || !state.units_limit) {
      el.classList.add('hidden');
      return;
    }
    el.classList.remove('hidden');

    $('budget-used').textContent = String(state.units_used);
    $('budget-limit').textContent = String(state.units_limit);

    const tierEl = $('budget-tier');
    tierEl.textContent = state.tier || '—';

    const remaining = Math.max(0, state.units_limit - state.units_used);
    const remEl = $('budget-remaining');
    if (remaining === 0) {
      remEl.textContent = 'Plan full — upgrade to add more agents';
      remEl.style.color = 'var(--warning)';
    } else {
      remEl.textContent =
        `${remaining} unit${remaining === 1 ? '' : 's'} available`;
      remEl.style.color = '';
    }

    const pct = state.units_limit > 0
      ? Math.min(100, Math.round((state.units_used / state.units_limit) * 100))
      : 0;
    const fill = $('budget-bar-fill');
    fill.style.width = `${pct}%`;
    fill.classList.toggle('full', pct >= 100);

    // "Always included" chip list — AIL / LLM-gateway pods. These are
    // free on every plan and don't count against units. Clicking the
    // chip copies the OPENAI_BASE_URL + OPENAI_API_KEY env block, so
    // users can paste them straight into Cursor, Claude Desktop, any
    // OpenAI SDK, without running `tytus env --export`.
    const incEl = $('budget-included');
    const incList = $('budget-included-list');
    incList.innerHTML = '';
    if (!state.included || state.included.length === 0) {
      incEl.classList.add('hidden');
    } else {
      incEl.classList.remove('hidden');
      for (const inc of state.included) {
        const chip = buildIncludedChip(inc);
        incList.appendChild(chip);
      }
    }

    // Running-agent panels — one per active agent pod. Richer than a
    // chip: icon, name, public URL, and the same action set the tray
    // menu exposes (Open in Browser / Copy API URL / Restart /
    // Uninstall). This lets the wizard act as a full pod control
    // surface, not just an install dialog.
    const runEl = $('budget-running');
    const list = $('budget-running-list');
    list.innerHTML = '';
    if (!state.agents || state.agents.length === 0) {
      runEl.classList.add('hidden');
    } else {
      runEl.classList.remove('hidden');
      for (const a of state.agents) {
        list.appendChild(buildRunningPanel(a));
      }
    }
  }

  function buildRunningPanel(a) {
    const override = DISPLAY[a.agent_type] || {};
    const name = override.display_name || a.agent_type;
    const icon = override.icon || null;
    const apiUrlDisplay = a.api_url || a.public_url || '— provisioning —';

    const panel = document.createElement('div');
    panel.className = 'pod-panel';
    panel.innerHTML = `
      <div class="pod-head">
        <div class="pod-icon"></div>
        <div class="pod-head-text">
          <div class="pod-name">
            <span class="pod-name-text"></span>
            <span class="pod-running-dot" title="Running"></span>
          </div>
          <div class="pod-meta">
            <span class="pod-meta-item pod-id"></span>
            <span class="pod-meta-sep">·</span>
            <span class="pod-meta-item pod-units-badge"></span>
          </div>
        </div>
      </div>
      <div class="pod-url-row">
        <span class="pod-url-label">API URL</span>
        <code class="pod-url"></code>
      </div>
      <div class="pod-actions">
        <button type="button" class="pod-btn pod-btn-primary pod-open">
          <span class="icon">↗</span> Open in Browser
        </button>
        <button type="button" class="pod-btn pod-copy-url">
          <span class="icon">⧉</span> Copy API URL
        </button>
        <button type="button" class="pod-btn pod-copy-env">
          <span class="icon">⧉</span> Copy env
        </button>
        <button type="button" class="pod-btn pod-restart" title="Restart the agent container (keeps pod)">
          <span class="icon">⟳</span> Restart
        </button>
        <button type="button" class="pod-btn pod-btn-danger pod-uninstall" title="Remove the agent from this pod (keeps pod slot)">
          <span class="icon">✕</span> Uninstall
        </button>
        <button type="button" class="pod-btn pod-btn-danger pod-revoke" title="Revoke the pod: frees its units, wipes the workspace. Destructive.">
          <span class="icon">⚠</span> Revoke pod
        </button>
        <button type="button" class="pod-btn pod-stop-forwarder hidden" title="Stop the localhost UI forwarder for this pod.">
          <span class="icon">◼</span> Stop forwarder
        </button>
      </div>
    `;

    if (icon) {
      const img = document.createElement('img');
      img.src = icon; img.alt = ''; img.width = 28; img.height = 28;
      panel.querySelector('.pod-icon').appendChild(img);
    }
    panel.querySelector('.pod-name-text').textContent = name;
    // Phase B: prepend a running-job dot when the per-pod registry has
    // an active streamed action for this pod. budgetState carries
    // active_jobs_per_pod from /api/state.
    const podIdEl = panel.querySelector('.pod-id');
    podIdEl.textContent = '';
    const activeMap = (budgetState && budgetState.active_jobs_per_pod) || {};
    if (activeMap[a.pod_id]) {
      const dot = document.createElement('span');
      dot.className = 'pod-running-job-dot';
      dot.title = 'Action streaming…';
      podIdEl.appendChild(dot);
    }
    podIdEl.appendChild(document.createTextNode(`Pod ${a.pod_id}`));
    panel.querySelector('.pod-units-badge').textContent =
      `${a.units} unit${a.units === 1 ? '' : 's'}`;
    panel.querySelector('.pod-url').textContent = apiUrlDisplay;

    // Actions
    const openBtn = panel.querySelector('.pod-open');
    const hasPublic = !!a.public_url;
    if (!hasPublic) {
      openBtn.disabled = true;
      openBtn.title = 'Public URL not ready yet — try again after the pod finishes provisioning';
    }
    openBtn.addEventListener('click', async () => {
      try {
        const res = await fetch(
          `/api/pod/open?pod=${encodeURIComponent(a.pod_id)}`,
          { method: 'POST' },
        );
        if (!res.ok) flashErr(openBtn, 'Failed');
        else flashOk(openBtn, 'Opened');
      } catch { flashErr(openBtn, 'Failed'); }
    });

    const copyUrlBtn = panel.querySelector('.pod-copy-url');
    copyUrlBtn.addEventListener('click', async () => {
      if (!a.api_url) { flashErr(copyUrlBtn, 'No URL'); return; }
      await copyToClipboard(a.api_url);
      flashOk(copyUrlBtn, 'Copied');
    });

    const copyEnvBtn = panel.querySelector('.pod-copy-env');
    copyEnvBtn.addEventListener('click', async () => {
      if (!a.api_url) { flashErr(copyEnvBtn, 'No URL'); return; }
      const env =
        `export OPENAI_BASE_URL="${a.api_url}"\n` +
        `export OPENAI_API_KEY="${a.user_key || ''}"`;
      await copyToClipboard(env);
      flashOk(copyEnvBtn, 'Copied');
    });

    const restartBtn = panel.querySelector('.pod-restart');
    restartBtn.addEventListener('click', async () => {
      if (!confirm(`Restart the ${name} agent on pod ${a.pod_id}? In-flight requests are interrupted.`)) return;
      restartBtn.disabled = true;
      const prev = restartBtn.innerHTML;
      restartBtn.innerHTML = '<span class="icon">⟳</span> Restarting…';
      try {
        const res = await fetch(
          `/api/pod/restart?pod=${encodeURIComponent(a.pod_id)}`,
          { method: 'POST' },
        );
        if (res.status === 202) {
          setTimeout(() => { restartBtn.innerHTML = prev; restartBtn.disabled = false; loadBudget(); }, 2500);
        } else {
          flashErr(restartBtn, 'Failed');
          restartBtn.innerHTML = prev; restartBtn.disabled = false;
        }
      } catch {
        flashErr(restartBtn, 'Failed');
        restartBtn.innerHTML = prev; restartBtn.disabled = false;
      }
    });

    // "More formats" dropdown — per-pod variants use THIS pod's api_url
    // (public URL) combined with the stable user key. Same formats as
    // the AIL Gateway panel.
    panel.querySelector('.pod-actions')
      .appendChild(buildCopyMoreDropdown(a.api_url || '', a.user_key || ''));
    // "Open in ▸" dropdown — spawns a terminal with this pod's env and
    // optionally launches a detected AI CLI (Claude Code, Cursor,
    // OpenCode, Codex, …). Uses launcher.rs on the backend so the list
    // is identical to the tray's "Open in ▸" submenu.
    panel.querySelector('.pod-actions')
      .appendChild(buildOpenInDropdown(a.pod_id));

    // Revoke Pod — destructive. Ports the tray's per-pod Revoke item
    // (main.rs: `pod_NN_revoke`). Two-step confirm: the second one
    // forces the user to re-type the pod id so an accidental double-
    // click can't nuke a pod. Revoke frees the pod's units AND wipes
    // the container workspace, so we want positive intent here.
    const revokeBtn = panel.querySelector('.pod-revoke');
    revokeBtn.addEventListener('click', async () => {
      if (!confirm(
        `Revoke pod ${a.pod_id} (${name})?\n\n` +
        `This frees the pod's ${a.units} unit${a.units === 1 ? '' : 's'}, ` +
        `wipes the container workspace (unsaved conversations, state, ` +
        `overlays), and the slot goes back to the pool. You can reinstall ` +
        `later but the workspace state is GONE.`
      )) return;
      const typed = prompt(
        `Type the pod id (${a.pod_id}) to confirm:`
      );
      if (typed !== a.pod_id) {
        showToast('Revoke cancelled', 'err');
        return;
      }
      revokeBtn.disabled = true;
      const prev = revokeBtn.innerHTML;
      revokeBtn.innerHTML = '<span class="icon">⟳</span> Revoking…';
      try {
        const res = await fetch(
          `/api/pod/revoke?pod=${encodeURIComponent(a.pod_id)}`,
          { method: 'POST' }
        );
        if (!res.ok) {
          flashErr(revokeBtn, 'Failed');
          revokeBtn.innerHTML = prev; revokeBtn.disabled = false;
          return;
        }
        showToast(`Pod ${a.pod_id} revoking…`);
        // Scalesys + state.json settle in ~2.5s; then hard-refresh.
        setTimeout(async () => {
          await loadBudget();
          await loadCatalog(true);
        }, 2800);
      } catch (e) {
        flashErr(revokeBtn, 'Failed');
        revokeBtn.innerHTML = prev; revokeBtn.disabled = false;
      }
    });

    // Stop Forwarder — only visible when the per-pod localhost UI
    // forwarder is actually running (state.forwarders carries the
    // pod_id list). Populated on render from the /api/state snapshot.
    const stopFwdBtn = panel.querySelector('.pod-stop-forwarder');
    if (POD_FORWARDERS.has(a.pod_id)) {
      stopFwdBtn.classList.remove('hidden');
    }
    stopFwdBtn.addEventListener('click', async () => {
      stopFwdBtn.disabled = true;
      try {
        await fetch(
          `/api/pod/stop-forwarder?pod=${encodeURIComponent(a.pod_id)}`,
          { method: 'POST' }
        );
        showToast(`Forwarder for pod ${a.pod_id} stopping…`);
        // Give CLI cleanup ~1.5s, then hide the button.
        setTimeout(() => {
          stopFwdBtn.classList.add('hidden');
          stopFwdBtn.disabled = false;
        }, 1500);
      } catch (e) {
        showToast('Stop forwarder failed', 'err');
        stopFwdBtn.disabled = false;
      }
    });

    // Channels row — Telegram/Discord/Slack/LINE etc. Ported from the
    // tray submenu. Add/Remove actions spawn a Terminal (the CLI needs
    // hidden token input via `read -rs`, which browsers can't do).
    const channelsHost = document.createElement('div');
    channelsHost.className = 'pod-channels';
    panel.appendChild(channelsHost);
    renderPodChannels(channelsHost, a.pod_id);

    const uninstallBtn = panel.querySelector('.pod-uninstall');
    uninstallBtn.addEventListener('click', async () => {
      if (!confirm(
        `Uninstall the ${name} agent from pod ${a.pod_id}?\n\n` +
        `The pod slot stays allocated (AIL gateway keeps working on that pod). ` +
        `You can install a new agent on the same pod afterwards.`
      )) return;
      uninstallBtn.disabled = true;
      const prev = uninstallBtn.innerHTML;
      uninstallBtn.innerHTML = '<span class="icon">⟳</span> Uninstalling…';
      try {
        const res = await fetch(
          `/api/pod/uninstall?pod=${encodeURIComponent(a.pod_id)}`,
          { method: 'POST' },
        );
        if (res.status === 202) {
          // Uninstall is async — poll /api/state until the pod
          // disappears from the agents list (~5–15 s typically).
          pollUntilGone(a.pod_id, () => {
            loadBudget();
            loadCatalog(true);
          });
        } else {
          flashErr(uninstallBtn, 'Failed');
          uninstallBtn.innerHTML = prev; uninstallBtn.disabled = false;
        }
      } catch {
        flashErr(uninstallBtn, 'Failed');
        uninstallBtn.innerHTML = prev; uninstallBtn.disabled = false;
      }
    });

    return panel;
  }

  async function copyToClipboard(text) {
    try { await navigator.clipboard.writeText(text); return true; }
    catch {
      const ta = document.createElement('textarea');
      ta.value = text; document.body.appendChild(ta);
      ta.select();
      try { document.execCommand('copy'); } catch {}
      ta.remove();
      return false;
    }
  }

  // Wave 3a: clipboard-variant helpers used by both the Included
  // (AIL Gateway) panel and the per-pod panels. Keeping the formatters
  // here instead of inline in the HTML template avoids template string
  // escape hell and keeps the formats testable / tweakable in one place.
  const COPY_VARIANTS = {
    openai: (url, key) =>
      `export OPENAI_BASE_URL="${url}"\n` +
      `export OPENAI_API_KEY="${key}"\n` +
      `export OPENAI_API_BASE="${url}"`,
    // ANTHROPIC_BASE_URL is the bare origin (no /v1) — the Anthropic SDK
    // appends /v1/messages itself, so a double-prefixed URL 404s.
    anthropic: (url, key) => {
      const origin = url.replace(/\/v1\/?$/, '');
      return `export ANTHROPIC_API_KEY="${key}"\n` +
             `export ANTHROPIC_BASE_URL="${origin}"`;
    },
    json: (url, key) => JSON.stringify({ url, api_key: key }, null, 2),
    urlOnly: (url, _key) => url,
    keyOnly: (_url, key) => key,
  };

  // Editor catalog cache — populated once on page load from
  // /api/launchers. Built-once, referenced per-pod without re-fetching.
  let EDITORS = { loaded: false, list: [], terminal: true };

  async function loadEditors() {
    try {
      const res = await fetch('/api/launchers');
      const body = await res.json();
      EDITORS.list = body.editors || [];
      EDITORS.terminal = !!body.terminal_available;
      EDITORS.loaded = true;
    } catch (_) {
      EDITORS = { loaded: true, list: [], terminal: true };
    }
  }

  /// Build the "Open in ▸" dropdown that spawns a terminal with the
  /// pod's env vars set and optionally launches an AI CLI. `podId` is
  /// the 2-digit pod id ("02") to target a specific pod; null uses the
  /// first pod with a stable key. Reuses the copy-more-menu styling so
  /// the two dropdowns look consistent when placed side-by-side.
  function buildOpenInDropdown(podId) {
    const wrap = document.createElement('details');
    wrap.className = 'copy-more open-in';
    const podQS = podId ? `&pod=${encodeURIComponent(podId)}` : '';
    const editorRows = EDITORS.list.length
      ? EDITORS.list.map((e) =>
          `<button type="button" data-binary="${e.binary}">${e.name}</button>`
        ).join('')
      : `<div class="open-in-empty">No AI CLIs detected on PATH.</div>`;
    wrap.innerHTML = `
      <summary class="pod-btn copy-more-summary">
        <span class="icon">↗</span> Open in
        <span class="copy-more-arrow">▾</span>
      </summary>
      <div class="copy-more-menu">
        ${editorRows}
        ${EDITORS.list.length ? '<div class="copy-more-sep"></div>' : ''}
        <button type="button" data-binary="terminal">Terminal</button>
      </div>
    `;
    wrap.querySelectorAll('button[data-binary]').forEach((btn) => {
      btn.addEventListener('click', async (ev) => {
        ev.stopPropagation();
        const bin = ev.currentTarget.dataset.binary;
        wrap.removeAttribute('open');
        try {
          const res = await fetch(
            `/api/launch?editor=${encodeURIComponent(bin)}${podQS}`,
            { method: 'POST' }
          );
          const body = await res.json().catch(() => ({}));
          if (!res.ok) {
            showToast(body.error || 'Launch failed', 'err');
          } else {
            showToast(`Opening ${ev.currentTarget.textContent.trim()}…`);
          }
        } catch (err) {
          showToast(`Launch failed: ${err}`, 'err');
        }
      });
    });
    return wrap;
  }

  /// Build the <details>/<summary> "More copy formats" dropdown that
  /// supplements the panel's primary Copy buttons. `url` is the full
  /// OpenAI-compatible URL (…/v1) and `key` is the bearer token.
  /// Returns a DOM element ready to append into pod-actions.
  function buildCopyMoreDropdown(url, key) {
    const wrap = document.createElement('details');
    wrap.className = 'copy-more';
    wrap.innerHTML = `
      <summary class="pod-btn copy-more-summary">
        <span class="icon">⧉</span> More formats
        <span class="copy-more-arrow">▾</span>
      </summary>
      <div class="copy-more-menu">
        <button type="button" data-format="openai">Shell exports (OpenAI)</button>
        <button type="button" data-format="anthropic">Shell exports (Anthropic)</button>
        <button type="button" data-format="json">JSON ({url, api_key})</button>
        <button type="button" data-format="urlOnly">URL only</button>
        <button type="button" data-format="keyOnly">API key only</button>
        <a class="copy-more-link" href="https://github.com/traylinx/tytus-cli#connect-from-claude-cursor-opencode" target="_blank" rel="noopener">
          Paste into Claude / Cursor / OpenCode…
        </a>
      </div>
    `;
    const has = !!url && !!key;
    wrap.querySelectorAll('button[data-format]').forEach((btn) => {
      btn.disabled = !has;
      btn.addEventListener('click', async (ev) => {
        ev.stopPropagation();
        const fmt = ev.currentTarget.dataset.format;
        const text = COPY_VARIANTS[fmt](url, key);
        await copyToClipboard(text);
        wrap.removeAttribute('open');
        showToast(`Copied — ${ev.currentTarget.textContent.trim()}`);
      });
    });
    return wrap;
  }

  function flashOk(btn, label) {
    btn.classList.add('flashed-ok');
    const prev = btn.innerHTML;
    btn.innerHTML = label;
    setTimeout(() => { btn.classList.remove('flashed-ok'); btn.innerHTML = prev; }, 1200);
  }
  function flashErr(btn, label) {
    btn.classList.add('flashed-err');
    const prev = btn.innerHTML;
    btn.innerHTML = label;
    setTimeout(() => { btn.classList.remove('flashed-err'); btn.innerHTML = prev; }, 1500);
  }

  /// Wave 3c: render the Channels row for a single pod. Fetches
  /// /api/channels?pod=NN, lists configured channels with a Remove
  /// button per row, and exposes an "Add channel ▸" dropdown of the
  /// channels this pod does NOT yet have. Browse-all rides into the
  /// CLI via `tytus channels catalog`.
  async function renderPodChannels(host, podId) {
    host.innerHTML = '<div class="pod-channels-head">Channels</div>' +
                     '<div class="pod-channels-body pod-channels-loading">Loading…</div>';
    let body;
    try {
      const res = await fetch(`/api/channels?pod=${encodeURIComponent(podId)}`);
      body = await res.json();
      if (!res.ok) throw new Error(body.error || `HTTP ${res.status}`);
    } catch (err) {
      host.querySelector('.pod-channels-body').textContent =
        `Couldn't load channels: ${err.message || err}`;
      return;
    }
    const configured = body.configured || [];
    const available = body.available || [];

    const rows = document.createElement('div');
    rows.className = 'pod-channels-body';

    if (configured.length === 0) {
      const empty = document.createElement('div');
      empty.className = 'pod-channels-empty';
      empty.textContent = 'No channels configured yet.';
      rows.appendChild(empty);
    } else {
      for (const c of configured) {
        const row = document.createElement('div');
        row.className = 'pod-channel-row';
        row.innerHTML = `
          <span class="pod-channel-check">✓</span>
          <span class="pod-channel-name"></span>
          <span class="pod-channel-count"></span>
          <button type="button" class="pod-channel-remove btn-link-danger">Remove</button>
        `;
        row.querySelector('.pod-channel-name').textContent = c.label || c.name;
        row.querySelector('.pod-channel-count').textContent =
          `(${c.secret_count} secret${c.secret_count === 1 ? '' : 's'})`;
        row.querySelector('.pod-channel-remove').addEventListener('click', async () => {
          if (!confirm(
            `Remove ${c.label || c.name} from pod ${podId}?\n\n` +
            `Clears credentials from the OS keychain, wipes them from the pod's ` +
            `state volume, and redeploys the agent so the channel stops operating.`
          )) return;
          await fetch(
            `/api/channels/remove?pod=${encodeURIComponent(podId)}&name=${encodeURIComponent(c.name)}`,
            { method: 'POST' }
          );
          showToast('Removing — see the Terminal window.');
          setTimeout(() => renderPodChannels(host, podId), 12000);
        });
        rows.appendChild(row);
      }
    }

    // Add-channel dropdown. If every known channel is already
    // configured, we still show "Browse available channels…" so users
    // can discover future additions from the CLI catalog.
    const addWrap = document.createElement('details');
    addWrap.className = 'copy-more pod-channel-add';
    const addOptions = available.length
      ? available.map((c) =>
          `<button type="button" data-name="${c.name}">Add ${c.label}…</button>`
        ).join('')
      : '';
    addWrap.innerHTML = `
      <summary class="pod-btn copy-more-summary">
        <span class="icon">+</span> Add channel
        <span class="copy-more-arrow">▾</span>
      </summary>
      <div class="copy-more-menu">
        ${addOptions}
        ${addOptions ? '<div class="copy-more-sep"></div>' : ''}
        <button type="button" data-name="__browse">Browse available channels…</button>
      </div>
    `;
    addWrap.querySelectorAll('button[data-name]').forEach((btn) => {
      btn.addEventListener('click', async (ev) => {
        ev.stopPropagation();
        const name = ev.currentTarget.dataset.name;
        addWrap.removeAttribute('open');
        if (name === '__browse') {
          // Phase A migrated catalog to inline JSON; show in-page.
          __runChannelsCatalogInline();
          return;
        }
        // Phase C: open the token modal in-page; Terminal is no
        // longer involved. On submit the modal POSTs the token in
        // the request body to /api/channels/add.
        const label = (CHANNELS.find((c) => c.id === name) || {}).label || name;
        openTokenModal(podId, name, label, () => {
          // Refresh after the redeploy (15-20s).
          setTimeout(() => renderPodChannels(host, podId), 20000);
        });
      });
    });
    rows.appendChild(addWrap);

    host.innerHTML = '';
    host.appendChild(Object.assign(document.createElement('div'), {
      className: 'pod-channels-head',
      textContent: 'Channels',
    }));
    host.appendChild(rows);
  }

  async function pollUntilGone(pod_id, done) {
    for (let i = 0; i < 20; i++) {
      await new Promise((r) => setTimeout(r, 1200));
      try {
        const s = await (await fetch('/api/state')).json();
        const still = (s.agents || []).some((x) => x.pod_id === pod_id);
        if (!still) { done(); return; }
      } catch {}
    }
    done(); // give up politely after ~24 s — state refreshes on next focus
  }

  function buildIncludedChip(inc) {
    // AIL pod rendered as a full pod panel (same visual rhythm as an
    // agent pod) with its own action set: copy API URL, copy env, and
    // a link to the public mirror URL when the edge is wired up. No
    // Restart/Uninstall — the default pod is AIL-only and isn't
    // user-restartable from outside the droplet.
    const endpointV1 = (inc.endpoint || 'http://10.42.42.1:18080') + '/v1';
    const publicApi = inc.public_url ? `${inc.public_url}/v1` : null;
    const panel = document.createElement('div');
    panel.className = 'pod-panel pod-panel-included';
    panel.innerHTML = `
      <div class="pod-head">
        <div class="pod-icon pod-icon-ail">A</div>
        <div class="pod-head-text">
          <div class="pod-name">
            <span class="pod-name-text">AIL LLM Gateway</span>
            <span class="inc-free-badge">INCLUDED</span>
          </div>
          <div class="pod-meta">
            <span class="pod-meta-item pod-id"></span>
            <span class="pod-meta-sep">·</span>
            <span class="pod-meta-item">OpenAI-compatible · paste into Cursor, Claude Desktop, any SDK</span>
          </div>
        </div>
      </div>
      <div class="pod-url-row">
        <span class="pod-url-label">Private (WireGuard)</span>
        <code class="pod-url pod-url-private"></code>
      </div>
      <div class="pod-url-row pod-url-row-public hidden">
        <span class="pod-url-label">Public</span>
        <code class="pod-url pod-url-public"></code>
      </div>
      <div class="pod-actions">
        <button type="button" class="pod-btn pod-btn-primary pod-copy-env">
          <span class="icon">⧉</span> Copy env
        </button>
        <button type="button" class="pod-btn pod-copy-url">
          <span class="icon">⧉</span> Copy Private URL
        </button>
        <button type="button" class="pod-btn pod-copy-public-url hidden">
          <span class="icon">⧉</span> Copy Public URL
        </button>
      </div>
    `;
    panel.querySelector('.pod-id').textContent = `Pod ${inc.pod_id}`;
    panel.querySelector('.pod-url-private').textContent = endpointV1;
    if (publicApi) {
      panel.querySelector('.pod-url-row-public').classList.remove('hidden');
      panel.querySelector('.pod-url-public').textContent = publicApi;
      panel.querySelector('.pod-copy-public-url').classList.remove('hidden');
    }

    const copyEnv =
      `export OPENAI_BASE_URL="${endpointV1}"\n` +
      `export OPENAI_API_KEY="${inc.user_key || ''}"`;

    panel.querySelector('.pod-copy-env').addEventListener('click', async (ev) => {
      ev.stopPropagation();
      await copyToClipboard(copyEnv);
      flashOk(ev.currentTarget, 'Copied');
    });
    panel.querySelector('.pod-copy-url').addEventListener('click', async (ev) => {
      ev.stopPropagation();
      await copyToClipboard(endpointV1);
      flashOk(ev.currentTarget, 'Copied');
    });
    if (publicApi) {
      panel.querySelector('.pod-copy-public-url').addEventListener('click', async (ev) => {
        ev.stopPropagation();
        await copyToClipboard(publicApi);
        flashOk(ev.currentTarget, 'Copied');
      });
    }
    // "More formats" dropdown — OpenAI/Anthropic/JSON/URL-only/key-only
    // plus a Paste-guide link. Prefers the public URL (works from any
    // network) over the WG private endpoint when both are available.
    const preferredUrl = publicApi || endpointV1;
    panel.querySelector('.pod-actions')
      .appendChild(buildCopyMoreDropdown(preferredUrl, inc.user_key || ''));
    // "Open in ▸" for the AIL pod — no pod param so the server picks
    // the first pod with a stable key (same logic as the tray's
    // connection_pair helper).
    panel.querySelector('.pod-actions')
      .appendChild(buildOpenInDropdown(inc.pod_id));
    return panel;
  }

  async function loadCatalog(refresh = false) {
    $('catalog-loading').classList.remove('hidden');
    $('catalog-error').classList.add('hidden');
    $('catalog').classList.add('hidden');
    try {
      const res = await fetch(`/api/catalog${refresh ? '?refresh=1' : ''}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();
      renderCatalog(data.agents || []);
    } catch (err) {
      $('catalog-loading').classList.add('hidden');
      $('catalog-error-msg').textContent = String(err);
      $('catalog-error').classList.remove('hidden');
    }
  }

  function renderCatalog(agents) {
    $('catalog-loading').classList.add('hidden');
    const grid = $('catalog');
    grid.innerHTML = '';
    for (const a of agents) {
      // Wizard display overrides win over the bare catalog row.
      // Falls through cleanly for agents we don't have a mapping for
      // yet — they render with whatever the Provider returned.
      const override = DISPLAY[a.id] || {};
      const display = {
        name: override.display_name || a.name || a.id,
        tagline: override.tagline || a.tagline || '',
        description: override.description || a.description || '',
        highlights: override.highlights || [],
        icon: override.icon || null,
        homepage: override.homepage || a.docs_url || null,
        github: override.github || null,
      };

      // Budget gating: disable the card if the agent costs more units
      // than the user currently has free. A missing or degraded
      // budgetState (offline/logged-out) leaves every card enabled so
      // we don't silently block someone who could actually install.
      const remaining = budgetState && budgetState.units_limit
        ? Math.max(0, budgetState.units_limit - budgetState.units_used)
        : null;
      const blocked = remaining !== null && a.units > remaining;

      const card = document.createElement('div');
      card.className = 'agent-card' + (blocked ? ' disabled' : '');
      card.dataset.id = a.id;
      card.innerHTML = `
        <div class="agent-head">
          <div class="agent-icon"></div>
          <div class="agent-head-text">
            <div class="agent-name"></div>
            <div class="agent-tagline"></div>
          </div>
          <div class="agent-units"></div>
        </div>
        <div class="agent-desc"></div>
        <ul class="agent-highlights"></ul>
        <div class="agent-blocker-slot"></div>
        <div class="agent-footer">
          <div class="agent-links"></div>
          <div class="agent-cta"><span class="cta-text">Install</span><span class="arrow">→</span></div>
        </div>
      `;

      if (blocked) {
        const warn = document.createElement('div');
        warn.className = 'agent-blocker';
        const need = a.units - (remaining ?? 0);
        warn.textContent =
          `Needs ${a.units} unit${a.units === 1 ? '' : 's'} — ` +
          `${need} more required. ` +
          `Uninstall another agent or upgrade your plan.`;
        card.querySelector('.agent-blocker-slot').replaceWith(warn);
        card.querySelector('.cta-text').textContent = 'Not enough units';
      } else {
        card.querySelector('.agent-blocker-slot').remove();
      }

      // Footer links — docs + source. Small, low-contrast so they
      // don't compete with the Install CTA; stopPropagation so a
      // click on "GitHub" doesn't also fire the card's install click.
      const linksEl = card.querySelector('.agent-links');
      const addLink = (href, label) => {
        if (!href) return;
        const a = document.createElement('a');
        a.href = href;
        a.target = '_blank';
        a.rel = 'noopener noreferrer';
        a.textContent = label;
        a.className = 'agent-link';
        a.addEventListener('click', (e) => e.stopPropagation());
        linksEl.appendChild(a);
      };
      addLink(display.homepage, 'Website');
      addLink(display.github, 'GitHub');

      // Icon slot: embed the SVG inline when we have one so it inherits
      // page colors cleanly; blank div if not.
      const iconEl = card.querySelector('.agent-icon');
      if (display.icon) {
        const img = document.createElement('img');
        img.src = display.icon;
        img.alt = '';
        img.width = 40;
        img.height = 40;
        iconEl.appendChild(img);
      } else {
        iconEl.classList.add('empty');
      }

      card.querySelector('.agent-name').textContent = display.name;
      card.querySelector('.agent-units').textContent =
        `${a.units} unit${a.units === 1 ? '' : 's'}`;
      card.querySelector('.agent-tagline').textContent = display.tagline;
      card.querySelector('.agent-desc').textContent = display.description;

      const ul = card.querySelector('.agent-highlights');
      if (display.highlights.length === 0) {
        ul.remove();
      } else {
        for (const h of display.highlights) {
          const li = document.createElement('li');
          li.textContent = h;
          ul.appendChild(li);
        }
      }

      // Pass the enriched display name through so the "Installing X…"
      // and success-screen titles use the human name, not `nemoclaw`.
      const clickable = { ...a, name: display.name };
      if (!blocked) {
        card.addEventListener('click', () => beginInstall(clickable));
      }
      grid.appendChild(card);
    }
    grid.classList.remove('hidden');
  }

  // Install-session state — everything the failure-path retry
  // needs to revoke + try again lives here.
  let installSession = null;
  let elapsedTimer = null;

  async function beginInstall(agent) {
    view.show('installing');
    $('installing-title').textContent = `Installing ${agent.name || agent.id}…`;
    $('log').textContent = '';
    $('installing-status').textContent = 'Contacting Tytus…';
    startElapsedCounter();
    installSession = {
      agent,
      started_at: Date.now(),
      log: '',
      pod_id: null,        // filled when the CLI prints the pod id
      stage: 'starting',   // starting | allocating | deploying | waiting | ready | failed
    };

    let jobId;
    try {
      const res = await fetch('/api/install', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ agent_type: agent.id }),
      });
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const body = await res.json();
      jobId = body.job_id;
      if (!jobId) throw new Error('no job id returned');
    } catch (err) {
      stopElapsedCounter();
      showFailure(`Failed to start install: ${err}`);
      return;
    }

    streamInstall(jobId, agent);
  }

  // Elapsed counter — plain seconds from install start. Shows the
  // user something is still happening even during the 15-60 s
  // reachability wait after CLI "done".
  function startElapsedCounter() {
    stopElapsedCounter();
    const started = Date.now();
    const el = $('installing-elapsed');
    const tick = () => {
      const s = Math.round((Date.now() - started) / 1000);
      if (el) el.textContent = `${s}s`;
    };
    tick();
    elapsedTimer = setInterval(tick, 1000);
  }
  function stopElapsedCounter() {
    if (elapsedTimer) { clearInterval(elapsedTimer); elapsedTimer = null; }
  }

  // Phase hints — derived from real CLI stderr phrases to update
  // the status line above the log. No fake progress bar; the log
  // IS the truth.
  function advanceStatusFromLog(line) {
    if (!installSession) return;
    const lower = line.toLowerCase();
    const set = (stage, status) => {
      installSession.stage = stage;
      $('installing-status').textContent = status;
    };
    if (lower.includes('allocating pod')) {
      set('allocating', 'Reserving your pod on the droplet…');
    } else if (lower.includes('deploying') || (lower.includes('installing ') && lower.includes(' on pod'))) {
      set('deploying', 'Starting the agent container…');
    } else if (lower.includes('zero-config') || lower.includes('configure') || lower.includes('gateway')) {
      set('deploying', 'Configuring the gateway…');
    } else if (lower.includes('activate:') || lower.includes('installed on pod')) {
      set('waiting', 'Finalizing…');
    }
    // Capture the pod id from the CLI echo "✓ <agent> installed on pod NN"
    // so the failure path can revoke it cleanly.
    const m = line.match(/installed on pod (\d+)/i);
    if (m) installSession.pod_id = m[1];
  }

  function streamInstall(jobId, agent) {
    const es = new EventSource(`/api/jobs/${jobId}/stream`);
    const logEl = $('log');
    let appended = '';
    es.addEventListener('log', (ev) => {
      const line = ev.data || '';
      appended += line + '\n';
      if (installSession) installSession.log = appended;
      logEl.textContent = appended;
      logEl.scrollTop = logEl.scrollHeight;
      advanceStatusFromLog(line);
    });
    es.addEventListener('done', (ev) => {
      es.close();
      // CLI returned "installed" BUT the container is almost always
      // still starting up. Don't jump to success — keep the live log
      // visible and poll the backend readiness probe until the pod
      // is actually answering. Only then transition to success.
      $('installing-status').textContent = 'Waiting for your assistant to come online…';
      let result = {};
      try { result = JSON.parse(ev.data || '{}'); } catch {}
      if (result.pod_id && installSession) installSession.pod_id = String(result.pod_id);
      waitForReadyThenSuccess(agent, result);
    });
    es.addEventListener('fail', (ev) => {
      es.close();
      stopElapsedCounter();
      // CLI exited non-zero. Pod may or may not exist (Scalesys
      // could have allocated before the failure). Show the full log
      // and offer the revoke+retry path — we'll figure out the pod
      // id either from the captured log OR by diffing state against
      // what was there at install start.
      showFailure(ev.data || 'Install failed', { offerRetry: true });
    });
    es.onerror = () => {
      // EventSource auto-retries; if the server is gone we cap it.
      // Server closes after `done`/`fail`, so we treat error without a
      // prior terminal event as a failure.
      if (document.body.dataset.view === 'installing') {
        // Wait one tick — some browsers fire error after the terminal
        // event before close; harmless if we already showed success.
        setTimeout(() => {
          if (document.body.dataset.view === 'installing') {
            showFailure('Lost connection to install stream.');
            es.close();
          }
        }, 500);
      }
    };
  }

  // Channel catalog for the "Connect a messenger" CTA. Must match the
  // CLI's REGISTRY in cli/src/channels.rs — `id` is the name passed to
  // `tytus channels add <id>`. Emoji is display-only; swap for real
  // lobehub SVGs in a future pass once we verify each slug exists.
  const CHANNELS = [
    { id: 'telegram', label: 'Telegram', emoji: '✈️', note: 'Works with a BotFather token' },
    { id: 'discord',  label: 'Discord',  emoji: '🎮', note: 'Bot token from Discord Developer Portal' },
    { id: 'slack',    label: 'Slack',    emoji: '💼', note: 'Socket Mode — no public URL needed' },
    { id: 'line',     label: 'LINE',     emoji: '💚', note: 'Outbound works today; inbound soon' },
  ];

  // Currently-installing/installed agent (captured in beginInstall +
  // surfaced in showSuccess for the next-step CTAs).
  let successContext = null;

  // After the CLI reports "done", poll the backend until the pod is
  // actually reachable. Writes "[HH:MM:SS] probing..." lines into
  // the live log so the user sees the wait isn't a freeze. On
  // timeout or repeated failure, transition to the failure view
  // with a retry button. This is the gate that stops "fake success
  // with a broken Chat now button" from ever shipping.
  async function waitForReadyThenSuccess(agent, result) {
    const logEl = $('log');
    const appendSystem = (msg) => {
      const stamp = new Date().toTimeString().slice(0, 8);
      const line = `[tytus ${stamp}] ${msg}\n`;
      logEl.textContent += line;
      logEl.scrollTop = logEl.scrollHeight;
      if (installSession) installSession.log = logEl.textContent;
    };

    const maxWaitMs = 120_000;       // 2 min hard ceiling
    const intervalMs = 2_500;        // poll every 2.5s
    const deadline = Date.now() + maxWaitMs;
    const pod_id = (result && result.pod_id) ||
      (installSession && installSession.pod_id) || null;

    if (!pod_id) {
      // No pod id surfaced from the CLI output — can't readiness-check
      // a specific pod. Best we can do is show success and hope state
      // caught up. Rare; keeps things moving.
      appendSystem(`(no pod id in CLI output — skipping readiness wait)`);
      stopElapsedCounter();
      await showSuccess(agent, result);
      return;
    }

    appendSystem(`CLI finished. Waiting for pod ${pod_id} to answer…`);

    let lastReason = '';
    while (Date.now() < deadline) {
      // Tell /api/state we want fresh data; it reads state.json each
      // time, so we always see the latest edge fields + the newly
      // derived slug-inherited public URL.
      try {
        const probe = await fetch(
          `/api/pod/ready?pod=${encodeURIComponent(pod_id)}`,
        ).then((r) => r.json());
        if (probe.ready) {
          appendSystem(`Pod ${pod_id} is online (${probe.reason}).`);
          stopElapsedCounter();
          await showSuccess(agent, result);
          return;
        }
        if (probe.reason && probe.reason !== lastReason) {
          appendSystem(`… still warming up (${probe.reason})`);
          lastReason = probe.reason;
        }
      } catch (e) {
        appendSystem(`(ready probe error: ${e}) — retrying`);
      }
      await new Promise((r) => setTimeout(r, intervalMs));
    }

    // Timed out. Pod may eventually come up, but from the user's POV
    // this is a failure — don't show a success screen with a button
    // that doesn't work. Offer revoke + retry.
    appendSystem(`Timeout after ${Math.round(maxWaitMs / 1000)}s. Pod didn't answer.`);
    stopElapsedCounter();
    showFailure(
      `Pod ${pod_id} didn't come online within ${Math.round(maxWaitMs / 1000)} seconds. ` +
      `You can revoke it and try again.`,
      { offerRetry: true },
    );
  }

  async function showSuccess(agent, result) {
    view.show('success');
    const name = agent.name || agent.id;
    $('success-title').textContent = `${name} installed.`;
    const panelSlot = $('success-panel-slot');
    panelSlot.innerHTML =
      '<div class="success-loading"><div class="spinner"></div>' +
      '<span>Finalizing…</span></div>';
    // Reset the reveal panels from any previous install.
    $('channels-picker').classList.add('hidden');
    $('editor-env').classList.add('hidden');

    // The CLI's `agent install --json` output only includes pod_id,
    // agent_type, and stable_ai_endpoint — NOT the stable_user_key,
    // and NOT the edge_public_url. So to build a correct env block +
    // a real pod panel (with Open in Browser / API URL / unit pill)
    // we re-fetch /api/state, which reads state.json directly and
    // carries every field we need. state.json is written *before* the
    // CLI prints the final JSON, so it's fresh by the time we land
    // here. Tolerate a tiny async gap with a short retry.
    let snap = null;
    let newPod = null;
    const target = String(result.pod_id || '');
    for (let i = 0; i < 5; i++) {
      try {
        snap = await (await fetch('/api/state')).json();
        budgetState = snap; // feed the chooser view as well
        newPod = (snap.agents || []).find((a) => a.pod_id === target);
        if (newPod) break;
      } catch {}
      await new Promise((r) => setTimeout(r, 400));
    }

    if (!newPod) {
      // Degraded path: state didn't surface the pod within 2s. Don't
      // block the user — show a minimal env block using known defaults
      // and the stable_user_key from the AIL/included pod (same key is
      // used across all of a user's pods).
      const fallbackKey = (snap && snap.included && snap.included[0])
        ? snap.included[0].user_key : '';
      const baseUrl = result.stable_ai_endpoint || 'http://10.42.42.1:18080';
      panelSlot.innerHTML = '';
      const env = document.createElement('pre');
      env.className = 'env';
      env.textContent =
        `export OPENAI_BASE_URL="${baseUrl}/v1"\n` +
        `export OPENAI_API_KEY="${fallbackKey || '<run: tytus env --export>'}"`;
      panelSlot.appendChild(env);
      return;
    }

    // Happy path: stash context for the CTAs, render the pod panel,
    // prep the editor env block.
    successContext = { agent, pod: newPod, snap };
    panelSlot.innerHTML = '';
    panelSlot.appendChild(buildRunningPanel(newPod));

    // Pre-populate the editor env block with the pod's own public URL
    // when available, else the universal private-endpoint pair. The
    // CTA only reveals it — this way copy works instantly.
    const included = (snap.included && snap.included[0]) || null;
    const envApi = newPod.api_url
      || (included ? `${included.endpoint}/v1` : 'http://10.42.42.1:18080/v1');
    const envKey = newPod.user_key
      || (included ? included.user_key : '');
    $('editor-env-block').textContent =
      `export OPENAI_BASE_URL="${envApi}"\n` +
      `export OPENAI_API_KEY="${envKey}"`;

    // Adjust the primary "Chat now" CTA: greyed + tooltip when the
    // public UI URL isn't ready yet (edge propagation lag after fresh
    // install). We still render the button so the flow is predictable.
    const openBtn = $('nx-open');
    if (!newPod.ui_url && !newPod.public_url) {
      openBtn.disabled = true;
      openBtn.title = 'Public URL still provisioning. Try again in 30–60 s, or use the editor CTA.';
    } else {
      openBtn.disabled = false;
      openBtn.title = '';
    }
  }

  // Build the messenger picker grid on demand — wired once on page
  // load, but populated with the active pod's id each time the picker
  // is revealed so "Connect Telegram" always targets the right pod.
  function populateChannelsPicker() {
    const grid = $('channels-grid');
    grid.innerHTML = '';
    const pod = successContext && successContext.pod;
    if (!pod) return;

    for (const c of CHANNELS) {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'channel-btn';
      btn.innerHTML = `
        <span class="channel-emoji"></span>
        <span class="channel-body">
          <span class="channel-label"></span>
          <span class="channel-note"></span>
        </span>
        <span class="channel-arrow">→</span>
      `;
      btn.querySelector('.channel-emoji').textContent = c.emoji;
      btn.querySelector('.channel-label').textContent = c.label;
      btn.querySelector('.channel-note').textContent = c.note;
      btn.addEventListener('click', async () => {
        btn.classList.add('flashed-ok');
        btn.querySelector('.channel-arrow').textContent = 'Opening Terminal…';
        try {
          await fetch(
            `/api/open-external?target=channel-setup&channel=${encodeURIComponent(c.id)}&pod=${encodeURIComponent(pod.pod_id)}`,
            { method: 'POST' },
          );
        } catch {}
        setTimeout(() => {
          btn.classList.remove('flashed-ok');
          btn.querySelector('.channel-arrow').textContent = '→';
        }, 1500);
      });
      grid.appendChild(btn);
    }
  }

  function showFailure(msg, opts) {
    opts = opts || {};
    view.show('failure');
    $('failure-msg').textContent = msg;
    // Pipe the full install log into the failure-log pre so the user
    // can scroll through the actual output if they want to file a
    // bug report or re-investigate. Hidden behind a disclosure so
    // the failure screen itself stays clean.
    const log = (installSession && installSession.log) || '';
    $('failure-log').textContent = log;

    const retryBtn = $('failure-retry');
    // Only show the revoke+retry button when we have a pod id to
    // revoke AND the caller opted in — errors that happened before
    // any pod was allocated don't have anything to revoke.
    const pod_id = installSession && installSession.pod_id;
    if (retryBtn) {
      if (opts.offerRetry && pod_id) {
        retryBtn.classList.remove('hidden');
        retryBtn.disabled = false;
        retryBtn.textContent = `Revoke pod ${pod_id} & try again`;
      } else {
        retryBtn.classList.add('hidden');
      }
    }
  }

  // ── wire up ──────────────────────────────────────────────
  $('retry').addEventListener('click', () => loadCatalog(true));
  $('install-another').addEventListener('click', async () => {
    // Refresh state so the budget strip + running-agents list + any
    // now-disabled cards reflect the install we just completed,
    // then return to the chooser.
    await loadBudget();
    await loadCatalog(true);
    view.show('chooser');
  });

  // ── Success-view CTAs ────────────────────────────────────
  $('nx-open').addEventListener('click', async () => {
    const pod = successContext && successContext.pod;
    if (!pod) return;
    try {
      await fetch(
        `/api/pod/open?pod=${encodeURIComponent(pod.pod_id)}`,
        { method: 'POST' },
      );
    } catch {}
  });

  $('nx-channels').addEventListener('click', () => {
    const picker = $('channels-picker');
    populateChannelsPicker();
    picker.classList.toggle('hidden');
    // Also hide the sibling reveal so only one is open at a time.
    $('editor-env').classList.add('hidden');
  });

  $('nx-editor').addEventListener('click', () => {
    const env = $('editor-env');
    env.classList.toggle('hidden');
    $('channels-picker').classList.add('hidden');
  });

  $('editor-copy').addEventListener('click', async () => {
    const btn = $('editor-copy');
    await copyToClipboard($('editor-env-block').textContent);
    const prev = btn.textContent;
    btn.textContent = 'Copied';
    setTimeout(() => { btn.textContent = prev; }, 1500);
  });
  // Optional-element wire-up: these IDs may not exist depending on
  // which success-view revision is live. Guard each to avoid the
  // `null.addEventListener` crash that would kill all later JS
  // (budget load, catalog load — everything got stuck behind it).
  const bind = (id, ev, fn) => {
    const el = $(id);
    if (el) el.addEventListener(ev, fn);
  };
  // Close buttons — try window.close() (works when the tab was
  // script-opened, which is our case: tray's `open <url>` usually
  // counts as programmatic in Chrome but not always). Fall back to
  // a farewell screen so the user never sees the stale wizard
  // sitting on a now-dead port.
  const farewell = () => {
    // Tear down SSE / timers in case we're navigating from mid-
    // stream, then swap the DOM to a calm "you can close this tab"
    // message. Leaving the tab navigable means the user can
    // re-trigger install from the tray menu if they want.
    const shell = document.querySelector('.shell');
    if (!shell) return;
    shell.innerHTML = `
      <div class="farewell">
        <div class="farewell-check">✓</div>
        <h2>You can close this tab.</h2>
        <p>Reopen from the Tytus tray menu any time — <strong>Pods Agents → Install Agent…</strong></p>
      </div>
    `;
  };
  const closeOrFarewell = () => {
    try { window.close(); } catch {}
    // window.close() is silent when blocked; give it a tick, then
    // swap the DOM if the tab is still here.
    setTimeout(() => {
      if (!document.hidden) farewell();
    }, 150);
  };
  bind('done', 'click', closeOrFarewell);
  bind('failure-close', 'click', closeOrFarewell);

  // Failure-view retry: revoke the allocated pod, refresh state,
  // return to the chooser. Shows "Revoking…" while the CLI spawns
  // detached — the actual revoke takes ~1-5s end to end.
  bind('failure-retry', 'click', async () => {
    const btn = $('failure-retry');
    const pod_id = installSession && installSession.pod_id;
    if (!btn || !pod_id) return;
    btn.disabled = true;
    const prev = btn.textContent;
    btn.textContent = `Revoking pod ${pod_id}…`;
    try {
      await fetch(
        `/api/pod/revoke?pod=${encodeURIComponent(pod_id)}`,
        { method: 'POST' },
      );
      // Give Scalesys + state.json a beat to settle, then hard-refresh
      // the chooser so the budget + running list reflect the revoke.
      setTimeout(async () => {
        await loadBudget();
        await loadCatalog(true);
        view.show('chooser');
        btn.textContent = prev;
        btn.disabled = false;
      }, 2500);
    } catch (e) {
      btn.textContent = `Revoke failed — try from Terminal`;
    }
  });

  // ── Tower Wave 1: header actions + Settings ──────────────────
  //
  // Header: Run Health Test + Connect/Disconnect. The connect button's
  // label flips between "Connect" / "Disconnect" based on tunnel_active
  // from /api/state. Both call simple POST endpoints; Connect spawns
  // a Terminal (sudo needs a TTY — matches tray UX), Disconnect is
  // detached and headless.
  //
  // Settings: two autostart toggles + Sign Out. Toggle state mirrors
  // the LaunchAgent plists on disk (read via /api/settings).

  function showToast(msg, kind = 'ok', ms = 2800) {
    const t = $('toast');
    if (!t) return;
    t.textContent = msg;
    t.className = `toast ${kind}`;
    t.classList.remove('hidden');
    clearTimeout(showToast._h);
    showToast._h = setTimeout(() => t.classList.add('hidden'), ms);
  }

  async function refreshHeaderConn() {
    try {
      const s = await (await fetch('/api/state')).json();
      const btn = $('hdr-conn');
      if (!btn) return;
      if (!s.logged_in) {
        btn.textContent = 'Sign In…';
        btn.dataset.state = 'connect';
        btn.disabled = false;
        return;
      }
      if (s.tunnel_active) {
        btn.textContent = 'Disconnect';
        btn.dataset.state = 'disconnect';
      } else {
        btn.textContent = 'Connect';
        btn.dataset.state = 'connect';
      }
      btn.disabled = false;
    } catch (_) { /* leave as-is */ }
  }

  $('hdr-conn')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget;
    const state = btn.dataset.state;
    btn.disabled = true;
    if (state === 'disconnect') {
      try {
        await fetch('/api/disconnect', { method: 'POST' });
        showToast('Tunnel coming down…');
        setTimeout(refreshHeaderConn, 2000);
      } catch (err) {
        showToast('Disconnect failed', 'err');
        btn.disabled = false;
      }
    } else {
      try {
        await fetch('/api/connect', { method: 'POST' });
        showToast('Check the Terminal window that just opened…');
        setTimeout(refreshHeaderConn, 4000);
      } catch (err) {
        showToast('Connect failed', 'err');
        btn.disabled = false;
      }
    }
  });

  // ── Streamed global action (Run Health Test / Run Doctor) ────
  //
  // Both endpoints now respond with `{job_id}` (HTTP 202) and stream
  // their subprocess output via SSE on /api/jobs/<id>/stream — same
  // pipeline as the install flow + the per-pod streamed actions.
  // Output appears live, line-by-line, instead of all-at-once after
  // the subprocess exits. Requires `cli/src/wizard.rs::flush()` to
  // flush stdout per println — without that, Rust block-buffers
  // stdout when piped and the lines all release at process exit.
  async function streamGlobalAction(opts) {
    const { url, panel, title, log, btn, prevText,
            okTitle, errTitle, runningTitle, errorTitle } = opts;
    const restoreBtn = () => { if (btn) { btn.disabled = false; btn.textContent = prevText; } };
    let res, body;
    try {
      res = await fetch(url, { method: 'POST' });
      body = await res.json();
    } catch (err) {
      panel.classList.add('err');
      title.textContent = errorTitle;
      log.textContent = String(err);
      restoreBtn();
      return;
    }
    if (!res.ok || !body || !body.job_id) {
      panel.classList.add('err');
      title.textContent = errorTitle;
      log.textContent = (body && body.error) || `Failed (HTTP ${res.status})`;
      restoreBtn();
      return;
    }
    if (title) title.textContent = runningTitle;
    const es = new EventSource(`/api/jobs/${encodeURIComponent(body.job_id)}/stream`);
    es.addEventListener('log', (ev) => {
      const line = (ev.data || '').replace(/\\n/g, '\n');
      log.textContent += line + '\n';
      log.scrollTop = log.scrollHeight;
    });
    es.addEventListener('exit', (ev) => {
      let code = -1;
      try { code = (JSON.parse(ev.data || '{}').code) ?? -1; } catch {}
      panel.classList.add(code === 0 ? 'ok' : 'err');
      title.textContent = code === 0 ? okTitle : errTitle(code);
      es.close();
      restoreBtn();
    });
    es.addEventListener('fail', (ev) => {
      panel.classList.add('err');
      title.textContent = errorTitle;
      log.textContent += `\n[error] ${ev.data || 'job failed'}`;
      es.close();
      restoreBtn();
    });
    es.onerror = () => {
      // Only treat onerror as fatal if no terminal event already fired
      // (panel already has ok/err class set in that case).
      if (!panel.classList.contains('ok') && !panel.classList.contains('err')) {
        panel.classList.add('err');
        title.textContent = 'Connection lost';
      }
      es.close();
      restoreBtn();
    };
  }

  $('hdr-health')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget;
    const panel = $('health-panel');
    const title = $('health-panel-title');
    const log = $('health-panel-log');
    if (!panel || !title || !log) return;
    btn.disabled = true;
    const prev = btn.textContent;
    btn.textContent = 'Testing…';
    panel.classList.remove('hidden', 'ok', 'err');
    title.textContent = 'Running health test…';
    log.textContent = '';
    streamGlobalAction({
      url: '/api/test',
      panel, title, log, btn, prevText: prev,
      okTitle: 'Health test passed ✓',
      errTitle: (code) => `Health test failed (exit ${code})`,
      runningTitle: 'Running health test…',
      errorTitle: 'Health test errored',
    });
  });

  $('health-panel-close')?.addEventListener('click', () => {
    $('health-panel').classList.add('hidden');
  });

  async function refreshSettings() {
    try {
      const s = await (await fetch('/api/settings')).json();
      const tun = $('st-autostart-tunnel');
      const tray = $('st-autostart-tray');
      if (tun) tun.checked = !!s.autostart_tunnel;
      if (tray) tray.checked = !!s.autostart_tray;
    } catch (_) { /* non-critical */ }
  }

  async function handleToggle(inputEl, endpoint, label) {
    const desired = inputEl.checked;
    inputEl.disabled = true;
    try {
      const res = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled: desired }),
      });
      const body = await res.json();
      if (!res.ok || body.error) {
        inputEl.checked = !desired;
        showToast(`${label}: ${body.error || 'failed'}`, 'err');
      } else {
        showToast(`${label}: ${desired ? 'enabled' : 'disabled'}`);
      }
    } catch (err) {
      inputEl.checked = !desired;
      showToast(`${label}: ${err}`, 'err');
    } finally {
      inputEl.disabled = false;
    }
  }

  $('st-autostart-tunnel')?.addEventListener('change', (e) =>
    handleToggle(e.currentTarget, '/api/settings/autostart-tunnel', 'Start tunnel at login'),
  );
  $('st-autostart-tray')?.addEventListener('change', (e) =>
    handleToggle(e.currentTarget, '/api/settings/autostart-tray', 'Launch tray at login'),
  );

  $('st-configure')?.addEventListener('click', async () => {
    try {
      await fetch('/api/configure', { method: 'POST' });
      showToast('Configure Agent opened in Terminal.');
    } catch (err) {
      showToast(`Configure failed: ${err}`, 'err');
    }
  });

  $('st-signout')?.addEventListener('click', async () => {
    const ok = window.confirm(
      'Sign out of Tytus?\n\n' +
      'This revokes all your pods, clears stored credentials, and tears ' +
      'down any active tunnels. You will need to sign in again to reconnect.'
    );
    if (!ok) return;
    try {
      await fetch('/api/logout', { method: 'POST' });
      showToast('Logging out — see the Terminal window.');
    } catch (err) {
      showToast('Sign out failed', 'err');
    }
  });

  // ── Tower Wave 2: Troubleshoot ───────────────────────────────
  //
  // Doctor button runs `tytus doctor` and renders stdout in a panel
  // (same shape as the Health panel). Daemon row shows live status
  // pill + start/stop/restart. Log viewer polls /api/logs with a
  // byte offset every 2s, appending new bytes to a <pre> and
  // auto-scrolling when "Follow" is checked.

  $('tr-doctor')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget;
    const panel = $('doctor-panel');
    const title = $('doctor-panel-title');
    const log = $('doctor-panel-log');
    if (!panel || !title || !log) return;
    btn.disabled = true;
    const prev = btn.textContent;
    btn.textContent = 'Running…';
    panel.classList.remove('hidden', 'ok', 'err');
    title.textContent = 'Running doctor…';
    log.textContent = '';
    streamGlobalAction({
      url: '/api/doctor',
      panel, title, log, btn, prevText: prev,
      okTitle: 'Doctor: all checks passed ✓',
      errTitle: (code) => `Doctor reported issues (exit ${code})`,
      runningTitle: 'Running doctor…',
      errorTitle: 'Doctor errored',
    });
  });

  $('doctor-panel-close')?.addEventListener('click', () => {
    $('doctor-panel').classList.add('hidden');
  });

  async function refreshDaemonStatus() {
    try {
      const s = await (await fetch('/api/daemon/status')).json();
      const pill = $('daemon-status-pill');
      const start = $('tr-daemon-start');
      const stop = $('tr-daemon-stop');
      const restart = $('tr-daemon-restart');
      if (!pill) return;
      if (s.running) {
        pill.textContent = `running (pid ${s.pid ?? '?'})`;
        pill.className = 'pill ok';
        if (start) start.disabled = true;
        if (stop) stop.disabled = false;
        if (restart) restart.disabled = false;
      } else {
        pill.textContent = 'stopped';
        pill.className = 'pill err';
        if (start) start.disabled = false;
        if (stop) stop.disabled = true;
        if (restart) restart.disabled = false;
      }
    } catch (_) {
      const pill = $('daemon-status-pill');
      if (pill) { pill.textContent = 'unknown'; pill.className = 'pill'; }
    }
  }

  async function daemonAction(verb) {
    try {
      const res = await fetch(`/api/daemon/${verb}`, { method: 'POST' });
      const body = await res.json().catch(() => ({}));
      if (res.ok) {
        showToast(`Daemon ${verb} — ok`);
      } else {
        showToast(`Daemon ${verb}: ${body.error || 'failed'}`, 'err');
      }
    } catch (err) {
      showToast(`Daemon ${verb}: ${err}`, 'err');
    } finally {
      // Give launchd a beat to update state before we probe.
      setTimeout(refreshDaemonStatus, 800);
    }
  }
  $('tr-daemon-start')?.addEventListener('click', () => daemonAction('start'));
  $('tr-daemon-stop')?.addEventListener('click', () => daemonAction('stop'));
  $('tr-daemon-restart')?.addEventListener('click', () => daemonAction('restart'));

  // ── Log tailing ──────────────────────────────────────────────
  const logState = {
    name: 'daemon',
    offset: 0,
    timer: null,
  };

  function startLogTail() {
    stopLogTail();
    // Reset view + offset when switching logs.
    const view = $('log-view');
    if (view) view.textContent = '';
    logState.offset = 0;
    pollLog(); // fire immediately so the user sees content on page load
    logState.timer = setInterval(pollLog, 2000);
  }

  function stopLogTail() {
    if (logState.timer) {
      clearInterval(logState.timer);
      logState.timer = null;
    }
  }

  async function pollLog() {
    try {
      const url = `/api/logs?name=${encodeURIComponent(logState.name)}&offset=${logState.offset}`;
      const res = await fetch(url);
      const body = await res.json();
      const missing = $('log-missing');
      if (body.missing) {
        if (missing) missing.classList.remove('hidden');
        return;
      }
      if (missing) missing.classList.add('hidden');
      const view = $('log-view');
      if (!view) return;
      if (body.truncated) {
        view.textContent = '';
      }
      if (body.chunk) {
        view.textContent += body.chunk;
        // Cap memory: trim to last 200 KB so the tab doesn't balloon on
        // long sessions. The server-side offset keeps incrementing; we
        // only drop UI-visible history.
        if (view.textContent.length > 200_000) {
          view.textContent = view.textContent.slice(view.textContent.length - 200_000);
        }
        if ($('log-follow')?.checked) {
          view.scrollTop = view.scrollHeight;
        }
      }
      logState.offset = body.offset;
    } catch (_) { /* transient — next tick will retry */ }
  }

  $('log-select')?.addEventListener('change', (e) => {
    logState.name = e.currentTarget.value;
    startLogTail();
  });
  $('log-clear')?.addEventListener('click', () => {
    const view = $('log-view');
    if (view) view.textContent = '';
  });

  $('footer-about')?.addEventListener('click', (e) => {
    e.preventDefault();
    $('footer-about-panel')?.classList.toggle('hidden');
  });

  // ── Shared Folders panel (v0.5.4 — parity with the tray submenu) ──
  // Read-only listing of bound folders + streamed status / conflicts /
  // refresh-all actions. Bind stays tray-only because the browser
  // sandbox can't surface a real OS folder path.
  async function refreshSharedFoldersList() {
    const host = $('sf-bindings');
    if (!host) return;
    try {
      const res = await fetch('/api/shared-folders/list');
      const body = await res.json();
      const bindings = (body && body.bindings) || [];
      if (bindings.length === 0) {
        host.innerHTML = '<span class="settings-hint">No folders bound yet. Bind via the menu-bar tray (Pods → ▸ Files → Bind a Mac folder…) or run <code>garagetytus folder bind</code> in your shell.</span>';
        return;
      }
      // No /api/state.home today → show absolute paths (the bindings
      // list is already wide; saving 14 chars isn't worth a state-shape
      // change). Tray-side does compress to ~/… because it has direct
      // access to $HOME.
      host.innerHTML = '';
      bindings
        .slice()
        .sort((a, b) => (a.bucket || '').localeCompare(b.bucket || ''))
        .forEach((b) => {
          const localPath = b.local_path || '';
          const display = localPath;
          const pods = (b.pods_provisioned || []).join(', ') || '—';
          const btn = document.createElement('button');
          btn.type = 'button';
          btn.className = 'btn-secondary sf-binding-row';
          btn.title = `Open ${localPath} in Finder · pods: ${pods}`;
          btn.innerHTML = `<strong>${escapeHtml(b.bucket || '')}</strong>` +
            `<span class="sf-arrow">  ↔  </span>` +
            `<span class="sf-path">${escapeHtml(display)}</span>` +
            (pods !== '—' ? `<span class="sf-pods"> · pods: ${escapeHtml(pods)}</span>` : '');
          btn.addEventListener('click', async () => {
            try {
              const r = await fetch('/api/shared-folders/open', {
                method: 'POST',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({ local_path: localPath }),
              });
              if (!r.ok) {
                const j = await r.json().catch(() => ({}));
                showToast(j.error || `Open failed (${r.status})`, 'err');
              }
            } catch (err) {
              showToast(`Open: ${err}`, 'err');
            }
          });
          host.appendChild(btn);
        });
    } catch (err) {
      host.innerHTML = `<span class="settings-hint">Failed to load bindings: ${escapeHtml(String(err))}</span>`;
    }
  }

  function escapeHtml(s) {
    return String(s)
      .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
  }

  function sfRunStreamed(action, runningTitle, okTitle, errTitlePrefix) {
    const panel = $('sf-panel');
    const title = $('sf-panel-title');
    const log = $('sf-panel-log');
    if (!panel || !title || !log) return;
    panel.classList.remove('hidden', 'ok', 'err');
    title.textContent = runningTitle;
    log.textContent = '';
    const btnIds = { status: 'sf-status', conflicts: 'sf-conflicts',
                     list: 'sf-list', 'refresh-all': 'sf-refresh-all' };
    const btn = $(btnIds[action]);
    const prev = btn ? btn.textContent : null;
    if (btn) { btn.disabled = true; btn.textContent = 'Running…'; }
    streamGlobalAction({
      url: `/api/shared-folders/run-streamed?action=${encodeURIComponent(action)}`,
      panel, title, log, btn, prevText: prev,
      okTitle,
      errTitle: (code) => `${errTitlePrefix} (exit ${code})`,
      runningTitle,
      errorTitle: `${errTitlePrefix} — request failed`,
    });
  }

  $('sf-status')?.addEventListener('click', () => sfRunStreamed(
    'status', 'Running garagetytus folder status…',
    'All bindings healthy ✓', 'Status reports issues',
  ));
  $('sf-conflicts')?.addEventListener('click', () => sfRunStreamed(
    'conflicts', 'Scanning for unresolved conflicts…',
    'No conflicts ✓', 'Conflicts found',
  ));
  $('sf-list')?.addEventListener('click', () => sfRunStreamed(
    'list', 'Listing bindings…',
    'List complete ✓', 'List failed',
  ));
  $('sf-refresh-all')?.addEventListener('click', () => sfRunStreamed(
    'refresh-all', 'Running refresh watchdog across every pod…',
    'Refresh complete ✓', 'Refresh reported issues',
  ));
  $('sf-panel-close')?.addEventListener('click', () => {
    $('sf-panel').classList.add('hidden');
  });
  $('sf-open-cache')?.addEventListener('click', async () => {
    try {
      await fetch('/api/shared-folders/open-cache', { method: 'POST' });
    } catch (err) {
      showToast(`Open cache: ${err}`, 'err');
    }
  });

  // Refresh the bindings list whenever the Shared Folders details opens
  // and at a slow background cadence while it stays open.
  let sfTimer = null;
  $('shared-folders')?.addEventListener('toggle', (e) => {
    if (e.currentTarget.open) {
      refreshSharedFoldersList();
      sfTimer = setInterval(refreshSharedFoldersList, 15000);
    } else {
      if (sfTimer) { clearInterval(sfTimer); sfTimer = null; }
    }
  });

  // ── Per-pod Refresh creds button (Output toolbar) ──────────────
  // Lives in the same toolbar as Restart / Stop forwarder / Uninstall /
  // Revoke. Streams the garagetytus-pod-refresh job into the same
  // pod-output-log panel that the other per-pod actions use.
  $('pod-run-refresh-creds')?.addEventListener('click', async (e) => {
    const btn = e.currentTarget;
    const podName = $('pod-sub-name')?.textContent || '';
    const pod = (podName.match(/(\d+)/) || [])[1];
    if (!pod) { showToast('No pod active', 'err'); return; }
    const log = $('pod-output-log');
    const status = $('pod-output-status');
    const prev = btn.textContent;
    btn.disabled = true; btn.textContent = 'Refreshing…';
    if (log) log.textContent = '';
    if (status) status.textContent = `Refreshing pod-${pod} credentials…`;
    let res, body;
    try {
      res = await fetch(`/api/pod/refresh-creds?pod=${encodeURIComponent(pod)}`,
                       { method: 'POST' });
      body = await res.json();
    } catch (err) {
      if (status) status.textContent = `Request failed: ${err}`;
      btn.disabled = false; btn.textContent = prev;
      return;
    }
    if (!res.ok || !body || !body.job_id) {
      if (status) status.textContent =
        (body && body.error) || `Failed (HTTP ${res.status})`;
      btn.disabled = false; btn.textContent = prev;
      return;
    }
    const es = new EventSource(`/api/jobs/${encodeURIComponent(body.job_id)}/stream`);
    es.addEventListener('log', (ev) => {
      const line = (ev.data || '').replace(/\\n/g, '\n');
      if (log) { log.textContent += line + '\n'; log.scrollTop = log.scrollHeight; }
    });
    es.addEventListener('exit', (ev) => {
      let code = -1;
      try { code = (JSON.parse(ev.data || '{}').code) ?? -1; } catch {}
      if (status) status.textContent = code === 0
        ? `pod-${pod} credentials rotated ✓`
        : `Refresh failed (exit ${code})`;
      es.close();
      btn.disabled = false; btn.textContent = prev;
    });
    es.addEventListener('fail', (ev) => {
      if (status) status.textContent = `Failed: ${ev.data || 'job failed'}`;
      es.close();
      btn.disabled = false; btn.textContent = prev;
    });
  });

  // Lazy-start the log tail the first time Troubleshoot is opened so
  // we don't poll a never-viewed surface. Also refresh daemon status
  // every 5s while Troubleshoot is open.
  let troubleTimer = null;
  $('troubleshoot')?.addEventListener('toggle', (e) => {
    if (e.currentTarget.open) {
      refreshDaemonStatus();
      startLogTail();
      troubleTimer = setInterval(refreshDaemonStatus, 5000);
    } else {
      stopLogTail();
      if (troubleTimer) { clearInterval(troubleTimer); troubleTimer = null; }
    }
  });

  // Fire catalog + budget in parallel; both populate the chooser view.
  // Budget resolves first in the common case (it's a ~80ms local
  // subprocess) and gates the card rendering.
  (async () => {
    // Editors are cheap (~20ms `which` probes) but the dropdown is
    // built synchronously at panel-render time, so we need the list
    // resolved before loadBudget() spawns the Running Pod panels.
    await loadEditors();
    await loadBudget();
    await loadCatalog(false);
    refreshHeaderConn();
    refreshSettings();
    // Hash deep-links (#/run/doctor, #/pod/02/restart) wait for this
    // signal so pod panels exist before we try to interact with them.
    window.dispatchEvent(new CustomEvent('state-ready'));
    // Gentle poll so tunnel-state flips (from other surfaces like the
    // tray menu) are reflected without a manual refresh.
    setInterval(refreshHeaderConn, 10000);
  })();
})();
