<script>
  const steps = [
    { state: 'downloaded', note: 'Staged at the depot over Wi-Fi. Applying it later doesn’t need the network.' },
    { state: 'waiting', note: 'TSD.Acquisition holds a reboot inhibitor while the survey records.', hold: true },
    { state: 'applying', note: 'Maintenance window opens and the inhibitor is released. The device reboots.' },
    { state: 'probation', note: 'The new release must pass the flavor’s health checks for 180 s.' },
    { state: 'healthy', note: 'Marked known-good. If it had failed, Avena would have rolled back on its own.', done: true },
  ]
</script>

<ol class="flow">
  {#each steps as s, i}
    <li class:hold={s.hold} class:done={s.done} style="--d:{i * 120}ms">
      <span class="dot">{i + 1}</span>
      <span class="state">{s.state}</span>
      <p>{s.note}</p>
    </li>
  {/each}
</ol>

<style>
  .flow {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 1rem;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    counter-reset: none;
    position: relative;
  }
  @media (min-width: 960px) {
    .flow::before {
      content: '';
      position: absolute;
      top: 1.05rem;
      left: 1rem;
      right: 1rem;
      height: 2px;
      background: linear-gradient(90deg, var(--brand), var(--warm));
      opacity: 0.5;
    }
  }
  li { position: relative; }
  .dot {
    position: relative;
    display: inline-grid;
    place-items: center;
    width: 2.1rem;
    height: 2.1rem;
    border-radius: 50%;
    font: 700 0.85rem var(--svp-code-font);
    color: #fff;
    background: var(--brand);
    box-shadow: 0 0 0 5px var(--paper);
  }
  :global(html.dark) .dot { box-shadow: 0 0 0 5px var(--paper-dark); }
  .hold .dot { background: var(--warm); color: var(--warm-ink); }
  .done .dot { background: #2f9e57; }
  .state {
    display: block;
    margin-top: 0.9rem;
    font-size: 0.9rem;
    font-weight: 600;
    font-family: var(--svp-code-font);
  }
  .hold .state { color: var(--warm); }
  p { margin: 0.35rem 0 0; font-size: 0.9rem; line-height: 1.55; color: color-mix(in srgb, currentColor 68%, transparent); }
</style>
