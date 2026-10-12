<script>
  import { resolve } from '$app/paths'
  import { IconifyIcon } from '@sveltepress/theme-default/components'

  const tiles = [
    {
      icon: 'box',
      title: 'Image-based systems',
      body: 'Every device boots a signed OCI image through bootc. Images are built once and promoted by digest, so each device’s software is known exactly.',
      to: '/docs/concepts/images-and-flavors/',
      size: 'wide',
    },
    {
      icon: 'refresh',
      title: 'Reconciliation',
      body: 'The agent keeps each device in its declared state. Desired and actual state are always visible.',
      to: '/docs/concepts/components/',
    },
    {
      icon: 'clock-pause',
      title: 'Scheduled updates',
      body: 'Updates can be downloaded at the depot and applied in a maintenance window. Applications can hold reboots while they finish critical work.',
      to: '/docs/concepts/updates/',
    },
    {
      icon: 'topology-star-3',
      title: 'Networking providers',
      body: 'Tailscale, the Avena WireGuard overlay, mDNS, cellular, and C-V2X are all providers. Each flavor picks the ones it needs.',
      to: '/docs/concepts/networking/',
    },
    {
      icon: 'arrow-back-up',
      title: 'Automatic rollback',
      body: 'Each new release boots into probation and is checked against your product’s own health policy. If the checks fail, the device rolls itself back and won’t retry that release.',
      to: '/docs/concepts/health-and-rollback/',
      size: 'tall',
    },
    {
      icon: 'tools',
      title: 'Development overrides',
      body: 'Replace a host binary or container image for testing. Overrides live in /run, so a reboot restores the published system.',
      to: '/docs/concepts/development/',
      size: 'wide',
    },
  ]
</script>

<div class="bento">
  {#each tiles as t}
    <a class="tile {t.size ?? ''}" href={resolve(t.to)}>
      <span class="icon"><IconifyIcon collection="tabler" name={t.icon} /></span>
      <h3>{t.title}</h3>
      <p>{t.body}</p>
    </a>
  {/each}
</div>

<style>
  .bento {
    display: grid;
    gap: 1rem;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
  }
  @media (min-width: 960px) {
    .bento { grid-template-columns: repeat(3, 1fr); grid-auto-flow: dense; }
    .wide { grid-column: span 2; }
    .tall { grid-row: span 2; }
  }
  .tile {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 1.5rem;
    border-radius: 1rem;
    text-decoration: none;
    color: inherit;
    background: color-mix(in srgb, var(--brand) 5%, transparent);
    border: 1px solid color-mix(in srgb, var(--brand) 18%, transparent);
    transition: border-color 0.2s, background 0.2s;
    overflow: hidden;
  }
  .tile:hover {
    border-color: var(--brand);
    background: color-mix(in srgb, var(--brand) 9%, transparent);
  }
  .tile::after {
    content: '→';
    position: absolute;
    top: 1.3rem;
    right: 1.4rem;
    opacity: 0;
    color: var(--warm);
    transition: opacity 0.2s, transform 0.2s;
  }
  .tile:hover::after { opacity: 1; transform: translateX(3px); }
  .tall {
    background:
      linear-gradient(160deg, color-mix(in srgb, var(--warm) 14%, transparent), transparent 60%),
      color-mix(in srgb, var(--brand) 5%, transparent);
  }
  .icon {
    display: inline-flex;
    align-self: flex-start;
    padding: 0.6rem;
    border-radius: 0.7rem;
    font-size: 1.4rem;
    color: #fff;
    background: var(--brand);
  }
  h3 { margin: 0.8rem 0 0; font-size: 1.2rem; font-weight: 650; }
  p { margin: 0; line-height: 1.6; font-size: 0.95rem; color: color-mix(in srgb, currentColor 70%, transparent); }
</style>
