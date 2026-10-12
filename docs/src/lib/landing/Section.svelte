<script>
  import { resolve } from '$app/paths'

  /** Shared section wrapper: eyebrow, heading, lede, optional link and a free-form body. */
  const { eyebrow, title, lede = '', link = null, split = false, reverse = false, children } = $props()
</script>

<section class="section" class:split class:reverse>
  <div class="intro">
    <p class="eyebrow">{eyebrow}</p>
    <h2>{title}</h2>
    {#if lede}<p class="lede">{lede}</p>{/if}
    {#if link}<a class="more" href={resolve(link.to)}>{link.label} <span aria-hidden="true">→</span></a>{/if}
  </div>
  <div class="body">
    {@render children?.()}
  </div>
</section>

<style>
  .section {
    max-width: 76rem;
    margin: 0 auto;
    padding: 4.5rem 1.5rem 0;
  }
  .intro { max-width: 42rem; }
  .eyebrow {
    margin: 0;
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font: 600 0.75rem var(--svp-code-font);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--warm);
  }
  .eyebrow::before {
    content: '';
    width: 1.5rem;
    height: 2px;
    background: var(--warm);
  }
  h2 {
    margin: 0.6rem 0 0.8rem;
    font-size: clamp(1.8rem, 3.4vw, 2.6rem);
    line-height: 1.1;
    font-weight: 700;
    letter-spacing: -0.025em;
    border: 0;
    padding: 0;
  }
  .lede {
    margin: 0;
    font-size: 1.08rem;
    line-height: 1.65;
    color: color-mix(in srgb, currentColor 70%, transparent);
  }
  .more {
    display: inline-block;
    margin-top: 1.1rem;
    font-weight: 600;
    color: var(--brand);
    text-decoration: none;
  }
  .more:hover { color: var(--warm); }
  .body { margin-top: 2.25rem; }

  @media (min-width: 960px) {
    .split {
      display: grid;
      grid-template-columns: 5fr 7fr;
      gap: 3.5rem;
      align-items: center;
    }
    .split .body { margin-top: 0; min-width: 0; }
    .split.reverse .intro { order: 2; }
  }
</style>
