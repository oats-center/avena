# OATS family kit

Shared look for the OATS project sites. The goal: someone moving between sites should know
immediately that they're in the same family, *and* which project they're looking at.

| Site | URL |
|---|---|
| Avena | https://oats-center.github.io/avena/ |
| avena-net | https://oats-center.github.io/avena-net/ |
| Trellis | https://oats-center.github.io/trellis/ |
| wattdog | https://oats-center.github.io/wattdog/ |

## What's shared (belongs together)

Never vary these per site.

1. **The family bar.** A thin, dark strip fixed to the top of every page, linking every family
   site, with the current one highlighted. `FamilyBar.svelte`. Status notes like "pre-1.0" go
   in its `note` pill, not in a separate banner.
2. **The family footer.** Site links, then "Part of the OATS Center family" with every
   site. `FamilyFooter.svelte`.
3. **Paper and ink.** Warm paper background `#fbfaf5` in light mode, a near-black panel color
   in dark mode, and brand-tinted dark panels (`--ink`) for terminals and calls to action.
4. **Type.** Bricolage Grotesque for display (h1–h3), the system UI font or Inter for body text, and
   JetBrains Mono for code, eyebrows, and kickers.
5. **Component grammar.** Eyebrow (mono, uppercase, warm-colored rule) → heading → lede; bento
   tiles; a dark terminal console with tabs showing **real output**; numbered flows. Corners
   use `--radius-card` (1rem) and `--radius-control` (0.6rem).
6. **Two-color marks.** Each logo is drawn on a 32×32 grid with rounded strokes. It uses
   exactly two hues: the site's signature color and its warm metal.
7. **`sites.js`.** Identical on every site. Adding a project means updating it everywhere.

## What's each site's own (identifiable)

Each site picks exactly **one signature color, one warm metal, one motif, and one mark**, and sets
them in its own `brand.css`.

| Site | Signature (`--brand`) | Warm metal (`--warm`) | Motif | Mark |
|---|---|---|---|---|
| **Avena** | field green `#3f8a52` | oat gold `#d9a441` | crop rows (one direction of parallel lines) | oat sprig |
| **avena-net** | deep teal `#2a7f86` | oat gold `#d9a441` (inherited, since it's part of Avena) | crossed lattice (the mesh) | oat sprig with linked nodes |
| **Trellis** | navy ink `#101828` | copper `#b86b35` | vertical trellis / diamond grid | trellis T (existing) |
| **wattdog** | charcoal `#323536` | brass `#b28d45` | pulse waveform over field rows | dog + bolt (existing) |

The family bar dot for each site uses its most recognizable color (`sites.js`): green, teal,
copper, and brass.

Rules of thumb:

- The signature color carries buttons, icons, links, and tags. The warm metal carries eyebrows,
  highlights, hover states, and the primary call-to-action on dark panels.
- Dark panels (`--ink`, `--console-*`) are tinted toward the signature color, not neutral gray.
- Motifs are subtle (about 8–9% opacity) and live only in hero and closing panels.
- Avena sub-projects (like avena-net) keep Avena's gold. Independent projects (Trellis, wattdog)
  use their own warm metal.

## Using the kit

```text
src/lib/family/
  family.css        tokens + base rules (any stack)
  sveltepress.css   offsets for the Sveltepress default theme (Sveltepress sites only)
  FamilyBar.svelte  top strip
  FamilyFooter.svelte
  sites.js          the family list
src/lib/brand.css   this site's signature, warm metal, ink, console colors, motif
```

Sveltepress (`+layout.svelte`):

```svelte
<script>
  import '@fontsource-variable/bricolage-grotesque'
  import '@fontsource-variable/jetbrains-mono'
  import '#lib/family/family.css'
  import '#lib/family/sveltepress.css'
  import '#lib/brand.css'
  import FamilyBar from '#lib/family/FamilyBar.svelte'
  const { children } = $props()
</script>

<FamilyBar current="avena-net" note="pre-1.0" />
{@render children?.()}
```

Tailwind/daisyUI (Trellis): import `family.css` and `brand.css` from `app.css`, render
`FamilyBar` in the root layout, and add `padding-top: var(--family-bar-h)` to the layout's
top-level container. Map daisyUI's `--color-primary`/`--color-accent` to `--brand`/`--warm`.

This folder is copied between repositories for now. If it starts to drift, move it into a small
shared package.
