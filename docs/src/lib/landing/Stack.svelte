<script>
  const layers = [
    {
      name: 'Trellis',
      tag: 'Avena.System · Avena.Workload · …',
      body: 'One typed API, with permissions checked on every call. The CLI, fleet tools, and your applications all use the same contracts. There is no second RPC protocol.',
    },
    {
      name: 'avena-agent',
      tag: 'reconcile · update policy · health',
      body: 'A privileged daemon that keeps the machine in its declared state, decides when updates are safe, and judges boot health.',
    },
    {
      name: 'Flavor',
      tag: '/usr/lib/avena/flavor.toml',
      body: 'Your product: drivers, host services, digest-pinned containers, and a manifest of the components it needs.',
    },
    {
      name: 'Avena base image',
      tag: 'ghcr.io/oats-center/avena',
      body: 'Fedora bootc plus the agent, Podman/Quadlet, and Trellis. Every update is atomic, and the previous deployment stays on disk to roll back to.',
    },
  ]
  let hovered = $state(1)
</script>

<div class="stack-wrap">
  <ol class="stack">
    {#each layers as l, i}
      <li
        class:on={hovered === i}
        style="--i:{i}"
        onmouseenter={() => (hovered = i)}
        onfocus={() => (hovered = i)}
        tabindex="0"
      >
        <span class="name">{l.name}</span>
        <code>{l.tag}</code>
      </li>
    {/each}
  </ol>
  <div class="detail" aria-live="polite">
    <p class="step">Layer {layers.length - hovered} of {layers.length}</p>
    <h3>{layers[hovered].name}</h3>
    <p>{layers[hovered].body}</p>
  </div>
</div>

<style>
  .stack-wrap {
    display: grid;
    gap: 2rem;
    align-items: center;
  }
  @media (min-width: 860px) {
    .stack-wrap { grid-template-columns: 1.2fr 1fr; }
  }
  .stack {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.5rem;
    perspective: 900px;
  }
  .stack li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 1.1rem 1.4rem;
    border-radius: 0.8rem;
    cursor: default;
    outline: none;
    margin-inline: calc((3 - var(--i)) * 0.9rem);
    background: color-mix(in srgb, var(--avena-green) calc(8% + var(--i) * 6%), transparent);
    border: 1px solid color-mix(in srgb, var(--avena-green) 25%, transparent);
    transform: rotateX(18deg);
    transition: transform 0.25s, background 0.25s, border-color 0.25s;
  }
  .stack li.on {
    transform: rotateX(0deg) scale(1.03);
    border-color: var(--avena-gold);
    background: color-mix(in srgb, var(--avena-gold) 16%, transparent);
  }
  .name { font-family: var(--avena-display); font-weight: 650; font-size: 1.1rem; }
  code {
    font-size: 0.75rem;
    opacity: 0.7;
    background: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .detail {
    padding: 1.5rem;
    border-left: 3px solid var(--avena-gold);
  }
  .step { margin: 0; font: 600 0.75rem var(--svp-code-font); color: var(--avena-gold); text-transform: uppercase; letter-spacing: 0.1em; }
  h3 { margin: 0.4rem 0 0.6rem; font-size: 1.5rem; }
  .detail p:last-child { margin: 0; line-height: 1.65; color: color-mix(in srgb, currentColor 72%, transparent); }
  @media (prefers-reduced-motion: reduce) {
    .stack li { transition: none; }
  }
</style>
