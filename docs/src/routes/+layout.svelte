<script>
  import '@fontsource-variable/bricolage-grotesque'
  import '@fontsource-variable/jetbrains-mono'
  import '#lib/avena.css'
  import DevBanner from '#lib/DevBanner.svelte'

  const { children } = $props()
</script>

<DevBanner />
{@render children?.()}
