<script>
  import { resolve } from '$app/paths'
  import { IconifyIcon } from '@sveltepress/theme-default/components'

  const flavors = [
    { icon: 'truck', name: 'TSD', image: 'avena-tsd', to: '/docs/flavors/tsd/', body: 'Survey trucks collecting GNSS, CAN, and camera data on the road, with uploads over cellular whenever there’s signal.', tags: ['GNSS', 'CAN', 'cameras', 'cellular'] },
    { icon: 'tractor', name: 'ISOBlue', image: 'avena-isoblue', to: '/docs/flavors/isoblue/', body: 'Tractor loggers for ISOBUS and GNSS data. Machines in the same field mesh with each other, with no uplink needed.', tags: ['ISOBUS', 'overlay', 'Wi-Fi'] },
    { icon: 'road', name: 'Roadside', image: 'avena-roadside', to: '/docs/flavors/roadside/', body: 'Unattended cabinets running sensing and V2X processing, with nobody nearby to press reset. Recovery has to be automatic.', tags: ['C-V2X', 'overlay', '600 s probation'] },
  ]
</script>

<div class="flavors">
  {#each flavors as f}
    <a class="card" href={resolve(f.to)}>
      <div class="top">
        <span class="icon"><IconifyIcon collection="tabler" name={f.icon} /></span>
        <code>{f.image}</code>
      </div>
      <h3>Avena {f.name}</h3>
      <p>{f.body}</p>
      <ul>{#each f.tags as t}<li>{t}</li>{/each}</ul>
    </a>
  {/each}
</div>

<style>
  .flavors { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fit, minmax(17rem, 1fr)); }
  .card {
    display: flex;
    flex-direction: column;
    padding: 1.5rem;
    border-radius: 1rem;
    text-decoration: none;
    color: inherit;
    border: 1px solid color-mix(in srgb, currentColor 12%, transparent);
    transition: border-color 0.2s, transform 0.2s;
  }
  .card:hover { border-color: var(--avena-gold); transform: translateY(-3px); }
  .top { display: flex; justify-content: space-between; align-items: center; }
  .icon { font-size: 2rem; color: var(--avena-gold); }
  code { font-size: 0.72rem; opacity: 0.6; background: none; }
  h3 { margin: 1rem 0 0.4rem; font-size: 1.3rem; }
  p { margin: 0; flex: 1; line-height: 1.6; font-size: 0.95rem; color: color-mix(in srgb, currentColor 70%, transparent); }
  ul { list-style: none; margin: 1.2rem 0 0; padding: 0; display: flex; flex-wrap: wrap; gap: 0.4rem; }
  li {
    padding: 0.2rem 0.6rem;
    border-radius: 999px;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--avena-green);
    background: color-mix(in srgb, var(--avena-green) 12%, transparent);
  }
</style>
