<script>
  import { asset, resolve } from '$app/paths'
  import FamilyFooter from '#lib/family/FamilyFooter.svelte'
</script>

<section class="cta">
  <div class="inner">
    <h2>Next steps</h2>
    <p>The VM quickstart boots the Avena lab image and walks through checking status and reconciling components with the <code>avena</code> CLI.</p>
    <div class="actions">
      <a class="btn primary" href={resolve('/docs/getting-started/quickstart-vm/')}>Quickstart →</a>
      <a class="btn" href={resolve('/docs/getting-started/concepts/')}>Avena in five minutes</a>
    </div>
  </div>
</section>

<FamilyFooter
  current="avena"
  logo={asset('/logo.svg')}
  links={[
    { label: 'Docs', href: resolve('/docs/') },
    { label: 'Tutorials', href: resolve('/docs/tutorials/01-tour/') },
    { label: 'CLI reference', href: resolve('/docs/reference/cli/') },
    { label: 'GitHub', href: 'https://github.com/oats-center/avena' },
  ]}
/>

<style>
  .cta { max-width: 76rem; margin: 6rem auto 0; padding: 0 1.5rem; }
  .inner {
    position: relative;
    padding: 3.5rem 2rem;
    border-radius: 1.25rem;
    text-align: center;
    color: var(--on-ink);
    background:
      repeating-linear-gradient(170deg, transparent 0 22px, rgb(255 255 255 / 0.04) 22px 23px),
      radial-gradient(80% 120% at 85% 0%, color-mix(in srgb, var(--warm) 35%, transparent), transparent 60%),
      var(--ink);
    overflow: hidden;
  }
  h2 { margin: 0; font-size: clamp(1.7rem, 3.5vw, 2.4rem); letter-spacing: -0.02em; border: 0; padding: 0; color: #fff; }
  p { margin: 0.8rem auto 2rem; max-width: 34rem; color: var(--on-ink-muted); font-size: 1.05rem; }
  .actions { display: flex; flex-wrap: wrap; justify-content: center; gap: 0.75rem; }
  .btn {
    padding: 0.7rem 1.3rem;
    border-radius: 0.6rem;
    font-weight: 600;
    text-decoration: none;
    color: var(--on-ink);
    border: 1.5px solid rgb(255 255 255 / 0.25);
  }
  .btn:hover { border-color: var(--warm); }
  .btn.primary { background: var(--warm); border-color: var(--warm); color: var(--warm-ink); }
  .btn.primary:hover { background: color-mix(in srgb, var(--warm) 85%, white); }
</style>
