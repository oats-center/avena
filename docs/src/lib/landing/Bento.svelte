<script>
  import { resolve } from '$app/paths'
  import { IconifyIcon } from '@sveltepress/theme-default/components'

  const tiles = [
    {
      icon: 'box',
      title: 'The image is the machine',
      body: 'Every device boots a signed OCI image through bootc. Build once, promote by digest, and know exactly what every device is running. No configuration drift.',
      to: '/docs/concepts/images-and-flavors/',
      size: 'wide',
    },
    {
      icon: 'refresh',
      title: 'Self-healing',
      body: 'The agent keeps each device in its declared state. Desired and actual state are always visible.',
      to: '/docs/concepts/components/',
    },
    {
      icon: 'clock-pause',
      title: 'Updates wait their turn',
      body: 'Download at the depot and apply in the maintenance window. Applications can hold reboots back while they finish critical work.',
      to: '/docs/concepts/updates/',
    },
    {
      icon: 'topology-star-3',
      title: 'Networking your way',
      body: 'Tailscale, the Avena WireGuard overlay, mDNS, cellular, and C-V2X are all providers. Each flavor picks the ones it needs.',
      to: '/docs/concepts/networking/',
    },
    {
      icon: 'arrow-back-up',
      title: 'Rollback with judgement',
      body: 'Each new release boots into probation and is checked against your product’s own health policy. If the checks fail, the device rolls itself back and won’t retry that release.',
      to: '/docs/concepts/health-and-rollback/',
      size: 'tall',
    },
    {
      icon: 'tools',
      title: 'Fearless development',
      body: 'Swap a host binary or container image in seconds. Every override lives in /run, so a reboot puts you back on the published system.',
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
