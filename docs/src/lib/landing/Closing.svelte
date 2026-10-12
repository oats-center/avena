<script>
  import { asset, resolve } from '$app/paths'
</script>

<section class="cta">
  <div class="inner">
    <h2>Boot Avena in a VM in ten minutes.</h2>
    <p>Download the lab image, run a few commands, and watch the agent reconcile the machine.</p>
    <div class="actions">
      <a class="btn primary" href={resolve('/docs/getting-started/quickstart-vm/')}>Start the quickstart →</a>
      <a class="btn" href={resolve('/docs/getting-started/concepts/')}>Avena in five minutes</a>
    </div>
  </div>
</section>

<footer class="foot">
  <div class="brand">
    <img src={asset('/logo.svg')} alt="" width="22" height="22" />
    <span><strong>Avena</strong>, an <a href="https://github.com/oats-center">OATS Center</a> project</span>
  </div>
  <nav>
    <a href={resolve('/docs/')}>Docs</a>
    <a href={resolve('/docs/tutorials/01-tour/')}>Tutorials</a>
    <a href={resolve('/docs/reference/cli/')}>CLI reference</a>
    <a href="https://github.com/oats-center/avena">GitHub</a>
  </nav>
</footer>

<style>
  .cta { max-width: 76rem; margin: 6rem auto 0; padding: 0 1.5rem; }
  .inner {
    position: relative;
    padding: 3.5rem 2rem;
    border-radius: 1.25rem;
    text-align: center;
    color: #eef6f0;
    background:
      repeating-linear-gradient(170deg, transparent 0 22px, rgb(255 255 255 / 0.04) 22px 23px),
      radial-gradient(80% 120% at 85% 0%, rgb(217 164 65 / 0.35), transparent 60%),
      var(--avena-soil);
    overflow: hidden;
  }
  h2 { margin: 0; font-size: clamp(1.7rem, 3.5vw, 2.4rem); letter-spacing: -0.02em; border: 0; padding: 0; color: #fff; }
  p { margin: 0.8rem auto 2rem; max-width: 34rem; color: #b9cdbf; font-size: 1.05rem; }
  .actions { display: flex; flex-wrap: wrap; justify-content: center; gap: 0.75rem; }
  .btn {
    padding: 0.7rem 1.3rem;
    border-radius: 0.6rem;
    font-weight: 600;
    text-decoration: none;
    color: #eef6f0;
    border: 1.5px solid rgb(255 255 255 / 0.25);
  }
  .btn:hover { border-color: var(--avena-gold); }
  .btn.primary { background: var(--avena-gold); border-color: var(--avena-gold); color: #2b1f00; }
  .btn.primary:hover { background: #e8bd62; }

  .foot {
    max-width: 76rem;
    margin: 4rem auto 0;
    padding: 1.5rem 1.5rem 3rem;
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: 1rem;
    font-size: 0.9rem;
    border-top: 1px solid color-mix(in srgb, currentColor 10%, transparent);
    color: color-mix(in srgb, currentColor 65%, transparent);
  }
  .brand { display: flex; align-items: center; gap: 0.6rem; }
  .foot a { color: inherit; text-decoration: none; }
  .foot a:hover { color: var(--avena-green); }
  nav { display: flex; gap: 1.4rem; flex-wrap: wrap; }
</style>
