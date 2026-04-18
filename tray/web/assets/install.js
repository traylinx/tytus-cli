(() => {
  'use strict';

  const $ = (id) => document.getElementById(id);

  const view = {
    show(name) {
      for (const v of ['chooser', 'installing', 'success', 'failure']) {
        const el = $(v);
        if (el) el.classList.toggle('hidden', v !== name);
      }
      document.body.dataset.view = name;
    },
  };

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
      const card = document.createElement('div');
      card.className = 'agent-card';
      card.dataset.id = a.id;
      card.innerHTML = `
        <div class="agent-head">
          <div class="agent-name"></div>
          <div class="agent-units"></div>
        </div>
        <div class="agent-tagline"></div>
        <div class="agent-desc"></div>
      `;
      card.querySelector('.agent-name').textContent = a.name || a.id;
      card.querySelector('.agent-units').textContent =
        `${a.units} unit${a.units === 1 ? '' : 's'}`;
      card.querySelector('.agent-tagline').textContent = a.tagline || '';
      card.querySelector('.agent-desc').textContent = a.description || '';
      card.addEventListener('click', () => beginInstall(a));
      grid.appendChild(card);
    }
    grid.classList.remove('hidden');
  }

  async function beginInstall(agent) {
    view.show('installing');
    $('installing-title').textContent = `Installing ${agent.name || agent.id}…`;
    $('log').textContent = '';

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
      showFailure(`Failed to start install: ${err}`);
      return;
    }

    streamInstall(jobId, agent);
  }

  function streamInstall(jobId, agent) {
    const es = new EventSource(`/api/jobs/${jobId}/stream`);
    const logEl = $('log');
    let appended = '';
    es.addEventListener('log', (ev) => {
      appended += (ev.data || '') + '\n';
      logEl.textContent = appended;
      logEl.scrollTop = logEl.scrollHeight;
    });
    es.addEventListener('done', (ev) => {
      es.close();
      try {
        const result = JSON.parse(ev.data || '{}');
        showSuccess(agent, result);
      } catch {
        showSuccess(agent, {});
      }
    });
    es.addEventListener('fail', (ev) => {
      es.close();
      showFailure(ev.data || 'Install failed');
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

  function showSuccess(agent, result) {
    view.show('success');
    const name = agent.name || agent.id;
    $('success-title').textContent = `${name} installed.`;
    const baseUrl = result.stable_ai_endpoint || 'http://10.42.42.1:18080';
    const apiKey = result.stable_user_key || '<run: tytus env --export>';
    $('env-block').textContent =
      `export OPENAI_BASE_URL="${baseUrl}/v1"\nexport OPENAI_API_KEY="${apiKey}"`;
  }

  function showFailure(msg) {
    view.show('failure');
    $('failure-msg').textContent = msg;
  }

  // ── wire up ──────────────────────────────────────────────
  $('retry').addEventListener('click', () => loadCatalog(true));
  $('copy-env').addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText($('env-block').textContent);
      $('copy-env').textContent = 'Copied';
      setTimeout(() => { $('copy-env').textContent = 'Copy'; }, 1500);
    } catch {
      // clipboard blocked — let the user select manually (env block is user-select: all)
    }
  });
  $('health-test').addEventListener('click', async () => {
    try {
      await fetch('/api/open-external?target=health-test', { method: 'POST' });
      $('health-test').textContent = 'Opened Terminal';
      setTimeout(() => { $('health-test').textContent = 'Run health test'; }, 1500);
    } catch {}
  });
  $('done').addEventListener('click', () => { window.close(); });
  $('failure-close').addEventListener('click', () => { window.close(); });

  loadCatalog(false);
})();
