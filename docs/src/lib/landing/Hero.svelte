<script>
  import { resolve } from '$app/paths'

  const devices = [
    {
      id: 'tsd-02',
      flavor: 'Avena TSD',
      release: 'tsd 2026.10.3',
      net: [['tailscale', 'ok'], ['cellular', 'ok']],
      components: '12 running · 0 degraded',
      workloads: [['tsd-acquisition', 'running'], ['tsd-processing', 'running'], ['tsd-ui', 'running']],
      update: ['2026.10.4 downloaded', 'waiting: survey recording active', 'warn'],
    },
    {
      id: 'isoblue-12',
      flavor: 'Avena ISOBlue',
      release: 'isoblue 2026.09.2',
      net: [['wifi', 'ok'], ['avena-overlay', 'ok']],
      components: '9 running · 1 blocked',
      workloads: [['isoblue-logger', 'running'], ['isobus-decoder', 'running'], ['field-sync', 'blocked']],
      update: ['current', 'can.implement not present', 'warn'],
    },
    {
      id: 'roadside-047',
      flavor: 'Avena Roadside',
      release: 'roadside 2026.10.1',
      net: [['ethernet', 'ok'], ['avena-overlay', 'ok'], ['c-v2x', 'ok']],
      components: '14 running · 0 degraded',
      workloads: [['rs-acquisition', 'running'], ['v2x-processing', 'running'], ['telemetry', 'running']],
      update: ['2026.10.2 in probation', 'healthy for 94s / 600s', 'ok'],
    },
  ]

  let active = $state(0)
  const d = $derived(devices[active])
  const pad = (s, n) => s + ' '.repeat(Math.max(1, n - s.length))
</script>

<section class="hero">
  <div class="field" aria-hidden="true"></div>

  <div class="copy">
    <p class="kicker"><span>bootc</span> + <span>systemd</span> + <span>Podman</span> + <span>Trellis</span></p>
    <h1>One platform for every machine <em>in the field.</em></h1>
    <p class="lead">
      Avena turns the Linux computers in trucks, tractors, and roadside cabinets into reproducible,
      self-healing systems. They update atomically, roll back on their own when a release isn't
      healthy, and are all controlled through one API.
    </p>
    <div class="actions">
      <a class="btn primary" href={resolve('/docs/getting-started/quickstart-vm/')}>Get started <span aria-hidden="true">→</span></a>
      <a class="btn" href={resolve('/docs/')}>Read the docs</a>
      <a class="btn ghost" href="https://github.com/oats-center/avena">GitHub</a>
    </div>
  </div>

  <div class="console">
    <div class="tabs" role="tablist" aria-label="Example devices">
      {#each devices as dev, i}
        <button role="tab" aria-selected={i === active} class:on={i === active} onclick={() => (active = i)}>
          {dev.id}
        </button>
      {/each}
    </div>
    <pre aria-live="polite"><span class="p">$</span> avena -t {d.id} status
<b>{d.flavor}</b> — {d.id}

{pad('System', 12)}{d.release}  <span class="ok">healthy</span>
{pad('Trellis', 12)}connected
{pad('Networking', 12)}{#each d.net as [n, s]}{n} <span class={s}>●</span>  {/each}
{pad('Components', 12)}{d.components}
{#each d.workloads as [w, s], i}{pad(i === 0 ? 'Workloads' : '', 12)}{pad(w, 17)}<span class={s === 'running' ? 'ok' : 'warn'}>{s}</span>
{/each}
{pad('Update', 12)}{d.update[0]}
{pad('', 12)}<span class={d.update[2]}>{d.update[1]}</span></pre>
    <p class="caption">Same platform, three products. Pick a device.</p>
  </div>
</section>

<style>
  .hero {
    position: relative;
    display: grid;
    gap: 3rem;
    align-items: center;
    max-width: 76rem;
    margin: 0 auto;
    padding: 4.5rem 1.5rem 3rem;
  }
  @media (min-width: 960px) {
    .hero { grid-template-columns: 1.1fr 1fr; padding-top: 6rem; }
  }

  /* Contour / crop-row motif */
  .field {
    position: absolute;
    inset: -2rem -50vw 0;
    z-index: -1;
    background:
      radial-gradient(60% 70% at 75% 30%, color-mix(in srgb, var(--avena-green) 18%, transparent), transparent 70%),
      repeating-linear-gradient(170deg, transparent 0 22px, color-mix(in srgb, var(--avena-green) 9%, transparent) 22px 23px);
    mask-image: linear-gradient(to bottom, black 40%, transparent);
  }

  .kicker {
    margin: 0 0 1rem;
    font-family: var(--svp-code-font);
    font-size: 0.8rem;
    color: color-mix(in srgb, currentColor 55%, transparent);
  }
  .kicker span { color: var(--avena-green); font-weight: 600; }

  h1 {
    margin: 0;
    font-size: clamp(2.6rem, 6vw, 4.4rem);
    line-height: 1;
    font-weight: 750;
    letter-spacing: -0.035em;
  }
  h1 em {
    font-style: normal;
    background: linear-gradient(100deg, var(--avena-green) 10%, var(--avena-gold) 90%);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }

  .lead {
    margin: 1.5rem 0 0;
    max-width: 36rem;
    font-size: 1.15rem;
    line-height: 1.65;
    color: color-mix(in srgb, currentColor 72%, transparent);
  }

  .actions { display: flex; flex-wrap: wrap; gap: 0.75rem; margin-top: 2rem; }
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.7rem 1.3rem;
    border-radius: 0.6rem;
    font-weight: 600;
    text-decoration: none;
    color: inherit;
    border: 1.5px solid color-mix(in srgb, currentColor 22%, transparent);
    transition: transform 0.15s, border-color 0.15s, background 0.15s;
  }
  .btn:hover { transform: translateY(-1px); border-color: var(--avena-green); }
  .btn.primary { background: var(--avena-green); border-color: var(--avena-green); color: #fff; }
  .btn.primary:hover { background: var(--avena-green-deep); }
  .btn.ghost { border-color: transparent; }

  .console {
    border-radius: 1rem;
    background: #0d1610;
    border: 1px solid #253a2c;
    box-shadow: 0 40px 80px -30px rgb(10 30 15 / 0.55);
    overflow: hidden;
  }
  .tabs { display: flex; background: #142219; border-bottom: 1px solid #253a2c; }
  .tabs button {
    flex: 1;
    padding: 0.65rem 0.5rem;
    background: none;
    border: 0;
    border-bottom: 2px solid transparent;
    color: #7f9a87;
    font: 500 0.78rem var(--svp-code-font);
    cursor: pointer;
  }
  .tabs button.on { color: #e9f3ec; border-bottom-color: var(--avena-gold); background: #0d1610; }
  pre {
    margin: 0;
    padding: 1.1rem 1.25rem 0.5rem;
    min-height: 17.5rem;
    font: 0.8rem/1.6 var(--svp-code-font);
    color: #c9dccf;
    background: none;
    overflow-x: auto;
  }
  pre b { color: #fff; }
  .p { color: var(--avena-gold); }
  .ok { color: #6fd08c; }
  .warn { color: #e8bd62; }
  .caption {
    margin: 0;
    padding: 0.4rem 1.25rem 0.9rem;
    font-size: 0.75rem;
    color: #6f8a77;
  }
</style>
