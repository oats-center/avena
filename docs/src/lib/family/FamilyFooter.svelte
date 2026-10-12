<script>
  import { sites, org } from './sites.js'

  /** current: this site's id; logo: URL of this site's mark; links: [{ label, href }] for this site */
  const { current, logo, links = [] } = $props()
  const self = sites.find((s) => s.id === current)
</script>

<footer class="family-footer">
  <div class="top">
    <div class="me">
      <img src={logo} alt="" width="24" height="24" />
      <strong>{self?.name}</strong>
      <span>{self?.role}</span>
    </div>
    <nav aria-label="{self?.name} links">
      {#each links as l}<a href={l.href}>{l.label}</a>{/each}
    </nav>
  </div>
  <div class="family">
    <span>Part of the <a href={org.github}>{org.name}</a> family:</span>
    {#each sites as s}
      <a href={s.url} class:current={s.id === current} style="--dot:{s.color}"><span class="dot" aria-hidden="true"></span>{s.name}</a>
    {/each}
  </div>
</footer>

<style>
  .family-footer {
    max-width: 76rem;
    margin: 4rem auto 0;
    padding: 1.5rem 1.5rem 3rem;
    font-size: 0.9rem;
    border-top: 1px solid color-mix(in srgb, currentColor 10%, transparent);
    color: color-mix(in srgb, currentColor 65%, transparent);
  }
  .top, .family { display: flex; flex-wrap: wrap; align-items: center; gap: 1rem 1.4rem; }
  .top { justify-content: space-between; }
  .me { display: flex; align-items: center; gap: 0.6rem; }
  .me strong { color: var(--brand); font-family: var(--font-display); font-size: 1.05rem; }
  nav { display: flex; flex-wrap: wrap; gap: 1.4rem; }
  a { color: inherit; text-decoration: none; }
  a:hover { color: var(--brand); }
  .family { margin-top: 1.1rem; font-size: 0.82rem; gap: 0.5rem 1.1rem; }
  .family a { display: inline-flex; align-items: center; gap: 0.4rem; }
  .family a.current { font-weight: 650; color: var(--brand); }
  .dot { width: 0.45rem; height: 0.45rem; border-radius: 50%; background: var(--dot); }
</style>
