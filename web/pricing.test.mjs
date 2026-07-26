// Landing-page pricing contract. Network-free on purpose: it stubs `fetch`, so
// CI never depends on the pricing API being up, and a red run always means the
// PAGE changed rather than production wobbling.
//
// Why this exists. Until 2026-07-26 this page hardcoded three plans that had
// been retired (Explorer/Creator/Operator at $39/$79/$149), advertised "Save 2
// months", and promised a "7-day money-back guarantee" that the terms of sale
// directly contradict. Nothing caught it because nothing was watching. These
// checks watch.
//
// Run: node web/pricing.test.mjs

import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';

const HTML = fs.readFileSync(new URL('./index.html', import.meta.url), 'utf8');
const SCRIPT = HTML.match(/<script>([\s\S]*?)<\/script>/)[1];

let failures = 0;
const test = (name, fn) => {
  try {
    const r = fn();
    return r instanceof Promise
      ? r.then(() => console.log(`  ok  ${name}`), e => { failures++; console.error(`  FAIL ${name}\n       ${e.message}`); })
      : console.log(`  ok  ${name}`);
  } catch (e) {
    failures++;
    console.error(`  FAIL ${name}\n       ${e.message}`);
  }
};

// ── sandbox ────────────────────────────────────────────────────────────────
function boot(fetchImpl) {
  const mk = () => ({ innerHTML: '', textContent: '', hidden: false, classList: { toggle() {} } });
  const els = { 'pricing-grid': mk(), 'annual-terms': mk(), 'pricing-footnote': mk() };
  // Real toggle stubs so setBillingButtons() is exercised: the failed-annual
  // path MOVES the toggle, and a test that cannot see that proves nothing.
  const buttons = ['month', 'year'].map(mode => ({
    mode, active: mode === 'month',
    getAttribute: () => mode,
    addEventListener() {},
    classList: { toggle(_cls, on) { this.owner.active = on; } }
  }));
  buttons.forEach(b => { b.classList.owner = b; });
  const ctx = {
    document: {
      getElementById: id => els[id] || null,
      querySelectorAll: sel => (sel === '[data-billing]' ? buttons : [])
    },
    navigator: { language: 'de-DE', languages: ['de-DE'] },
    Intl: { DateTimeFormat: () => ({ resolvedOptions: () => ({ timeZone: 'Europe/Berlin' }) }) },
    fetch: fetchImpl, URLSearchParams, console, setTimeout, clearTimeout
  };
  vm.createContext(ctx);
  vm.runInContext(SCRIPT, ctx);
  const activeToggle = () => (buttons.find(b => b.active) || {}).mode;
  return { ctx, els, buttons, activeToggle };
}

async function renderAt(ctx, els, mode) {
  vm.runInContext(`billingMode = ${JSON.stringify(mode)};`, ctx);
  await vm.runInContext('loadPricing()', ctx);
  const html = els['pricing-grid'].innerHTML;
  return {
    html,
    prices: [...html.matchAll(/<strong>([^<]+)<\/strong>/g)].map(m => m[1]),
    names: [...html.matchAll(/class="plan-name">([^<]+)</g)].map(m => m[1]),
    terms: els['annual-terms'],
    footnote: els['pricing-footnote'].textContent
  };
}

const jsonOk = payload => async () => ({ ok: true, json: async () => payload });
const dead = async () => { throw new Error('network down'); };

const livePayload = (interval, label) => ({
  success: true,
  data: {
    interval,
    region: { currency: 'eur', tax_copy: 'VAT included where applicable' },
    annual: interval === 'year'
      ? { enabled: true, refundable: false, consent_text: 'Billed once for 12 months, up front.' }
      : undefined,
    cards: [
      { id: 'starter', name: 'Starter', self_serve: true, popular: false, tagline: 'One private server.', price: { label }, highlights: ['EU-hosted'], cta: { label: 'Deploy My Pod' } },
      { id: 'custom', name: 'Custom', self_serve: false, tagline: 'Talk to us.', price: {}, highlights: [] }
    ]
  }
});

console.log('landing pricing contract');

// ── 1. the page must never carry a price of its own ───────────────────────
await test('no retired plan names survive in the markup', () => {
  const body = HTML.split('<script>')[0];
  for (const dead of ['Explorer', 'Creator', 'Operator']) {
    assert.ok(!body.includes(`>${dead}<`), `retired plan "${dead}" is still rendered`);
  }
});

await test('no refund promise anywhere (the terms say no refunds)', () => {
  assert.ok(!/money-back|money back/i.test(HTML.replace(/\/\/[^\n]*/g, '')), 'a money-back promise is still on the page');
});

await test('the annual toggle badge names no number of months', () => {
  const badge = HTML.match(/class="save"[^>]*>([^<]+)</)[1];
  assert.ok(!/\d/.test(badge), `toggle badge "${badge}" names a number`);
  assert.ok(!/month/i.test(badge), `toggle badge "${badge}" promises months`);
});

// ── 2. live payload wins ──────────────────────────────────────────────────
await test('renders the amount the API returns, monthly', async () => {
  const { ctx, els } = boot(jsonOk(livePayload('month', '€24')));
  const r = await renderAt(ctx, els, 'month');
  assert.deepEqual(r.prices, ['€24']);
  assert.deepEqual(r.names, ['Starter']);
});

await test('renders the amount the API returns, annual', async () => {
  const { ctx, els } = boot(jsonOk(livePayload('year', '€269/yr')));
  const r = await renderAt(ctx, els, 'year');
  assert.deepEqual(r.prices, ['€269/yr']);
});

await test('a changed API amount changes the page with no code edit', async () => {
  const { ctx, els } = boot(jsonOk(livePayload('month', '€31')));
  const r = await renderAt(ctx, els, 'month');
  assert.deepEqual(r.prices, ['€31'], 'page did not follow the API');
});

await test('skips cards that are not self-serve', async () => {
  const { ctx, els } = boot(jsonOk(livePayload('month', '€24')));
  const r = await renderAt(ctx, els, 'month');
  assert.ok(!r.names.includes('Custom'), 'a non-self-serve card was rendered as buyable');
});

// ── 3. annual terms show only where they are true ─────────────────────────
await test('shows the annual terms on the annual view', async () => {
  const { ctx, els } = boot(jsonOk(livePayload('year', '€269/yr')));
  const r = await renderAt(ctx, els, 'year');
  assert.equal(r.terms.hidden, false);
  assert.match(r.terms.textContent, /12 months/);
});

await test('hides the annual terms on the monthly view', async () => {
  const { ctx, els } = boot(jsonOk(livePayload('month', '€24')));
  const r = await renderAt(ctx, els, 'month');
  assert.equal(r.terms.hidden, true, '"not refundable" must not sit next to a plan you can cancel monthly');
});

await test('hides the annual terms when the API says annual is dark', async () => {
  const p = livePayload('year', '€269/yr');
  p.data.annual = { enabled: false };
  const { ctx, els } = boot(jsonOk(p));
  const r = await renderAt(ctx, els, 'year');
  assert.equal(r.terms.hidden, true);
});

// ── 4. degradation, not blanking ──────────────────────────────────────────
await test('falls back to the bundled snapshot when the API is down', async () => {
  const { ctx, els } = boot(dead);
  for (const mode of ['month', 'year']) {
    const r = await renderAt(ctx, els, mode);
    assert.ok(r.prices.length >= 4, `${mode} blanked instead of degrading`);
    assert.ok(!/\$/.test(r.html), `${mode} fallback shows a USD price`);
  }
});

await test('rejects a malformed payload rather than rendering it', async () => {
  const { ctx, els } = boot(jsonOk({ success: true, data: { cards: [{ id: 'x' }] } }));
  const r = await renderAt(ctx, els, 'month');
  assert.ok(r.prices.length >= 4, 'a malformed payload was rendered instead of the snapshot');
});

await test('rejects a non-200 rather than rendering it', async () => {
  const { ctx, els } = boot(async () => ({ ok: false, status: 500, json: async () => ({}) }));
  const r = await renderAt(ctx, els, 'month');
  assert.ok(r.prices.length >= 4);
});

// ── 5. injection ──────────────────────────────────────────────────────────
await test('escapes API-supplied text', async () => {
  const p = livePayload('month', '<img src=x onerror=alert(1)>');
  const { ctx, els } = boot(jsonOk(p));
  const r = await renderAt(ctx, els, 'month');
  assert.ok(!r.html.includes('<img'), 'API text was injected as raw HTML');
  assert.ok(r.html.includes('&lt;img'), 'API text was not escaped');
});

// ── 6. term integrity: never show one term's prices under the other's label ──
// These three come from the codex review of PR #37. The first is the same bug
// class that shipped in the pricing proxy the same day: a structurally perfect
// payload carrying the WRONG TERM, which nothing failed loudly on.

await test('rejects a payload whose interval is not the one requested', async () => {
  // Asked for a year, proxy answered with month. Structurally valid, wrong term.
  const wrong = livePayload('month', '€24');
  const { ctx, els, activeToggle } = boot(jsonOk(wrong));
  const r = await renderAt(ctx, els, 'year');
  assert.ok(!r.prices.includes('€24') || r.prices.length >= 4,
    'monthly amounts were rendered for a yearly request');
  assert.equal(activeToggle(), 'month', 'toggle must follow the term actually shown');
});

await test('a late reply from a superseded request cannot repaint the page', async () => {
  let release;
  const slowMonth = new Promise(res => { release = res; });
  const fetchImpl = async url => {
    if (String(url).includes('interval=month')) {
      await slowMonth;
      return { ok: true, json: async () => livePayload('month', '€24') };
    }
    return { ok: true, json: async () => livePayload('year', '€269/yr') };
  };
  const { ctx, els } = boot(fetchImpl);

  vm.runInContext("billingMode='month';", ctx);
  const monthCall = vm.runInContext('loadPricing()', ctx);   // in flight
  vm.runInContext("billingMode='year';", ctx);
  await vm.runInContext('loadPricing()', ctx);               // supersedes it
  release();                                                  // month lands late
  await monthCall;

  const prices = [...els['pricing-grid'].innerHTML.matchAll(/<strong>([^<]+)<\/strong>/g)].map(m => m[1]);
  assert.deepEqual(prices, ['€269/yr'], 'a stale monthly response repainted the yearly view');
});

await test('a failed ANNUAL load drops to monthly and moves the toggle with it', async () => {
  // The bundled snapshot cannot know whether annual is still sellable. Showing
  // an unconfirmed annual ladder would advertise a term we may have switched
  // off, so degrade toward the SHORTER commitment, never the longer one.
  const { ctx, els, activeToggle } = boot(dead);
  const r = await renderAt(ctx, els, 'year');
  assert.equal(activeToggle(), 'month', 'toggle still said Yearly while showing fallback prices');
  assert.equal(r.terms.hidden, true, 'annual terms shown for an unconfirmed annual ladder');
  assert.ok(r.prices.every(p => !/\/yr/.test(p)), 'yearly labels survived a failed annual load');
});

console.log(failures ? `\n${failures} failing` : '\nall passing');
process.exit(failures ? 1 : 0);
