<script>
  import '@fontsource-variable/bricolage-grotesque'
  import '@fontsource-variable/jetbrains-mono'
  import '#lib/family/family.css'
  import '#lib/family/sveltepress.css'
  import '#lib/brand.css'
  import FamilyBar from '#lib/family/FamilyBar.svelte'

  const { children } = $props()
</script>

<FamilyBar current="avena" note="in development" />
{@render children?.()}
