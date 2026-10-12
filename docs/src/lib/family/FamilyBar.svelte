<script>
  import { sites, org } from './sites.js'

  /** current: id of this site from sites.js; note: optional status text (e.g. "pre-1.0") */
  const { current, note = '' } = $props()
</script>

<nav class="family-bar" aria-label="OATS projects">
  <a class="org" href={org.url}>OATS</a>
  <span class="sep" aria-hidden="true"></span>
  <ul>
    {#each sites as s}
      <li>
        <a href={s.url} class:current={s.id === current} aria-current={s.id === current ? 'page' : undefined} title={s.role} style="--dot:{s.color}">
          <span class="dot" aria-hidden="true"></span>{s.name}
        </a>
      </li>
    {/each}
  </ul>
  {#if note}<span class="note">{note}</span>{/if}
</nav>

<style>
  .family-bar {
    position: fixed;
    inset: 0 0 auto 0;
    z-index: 1100;
    height: var(--family-bar-h);
    display: flex;
    align-items: center;
    gap: 0.9rem;
    padding: 0 1.25rem;
    font: 500 0.78rem/1 ui-sans-serif, system-ui, sans-serif;
    color: #a9b1ad;
    background: #12161a;
    border-bottom: 1px solid rgb(255 255 255 / 0.06);
    white-space: nowrap;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .org {
    font-family: var(--font-mono);
    font-weight: 700;
    letter-spacing: 0.12em;
    color: #e9ecea;
    text-decoration: none;
  }
  .sep { width: 1px; height: 14px; background: rgb(255 255 255 / 0.15); }
  ul { display: flex; gap: 0.25rem; margin: 0; padding: 0; list-style: none; }
  li a {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.35rem 0.6rem;
    border-radius: 999px;
    color: inherit;
    text-decoration: none;
    transition: color 0.15s, background 0.15s;
  }
  li a:hover { color: #fff; background: rgb(255 255 255 / 0.06); }
  li a.current { color: #fff; background: rgb(255 255 255 / 0.1); font-weight: 650; }
  .dot { width: 0.5rem; height: 0.5rem; border-radius: 50%; background: var(--dot); }
  .current .dot { box-shadow: 0 0 0 3px color-mix(in srgb, var(--dot) 35%, transparent); }
  .note {
    margin-left: auto;
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
    font-size: 0.72rem;
    color: var(--warm-ink);
    background: var(--warm);
  }
</style>
