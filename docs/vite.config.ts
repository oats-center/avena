import adapter from '@sveltejs/adapter-static'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'
import { defaultTheme } from '@sveltepress/theme-default'
import { sveltepress } from '@sveltepress/vite'
import { defineConfig } from 'vite'
import sidebar from './sidebar.json' with { type: 'json' }

export default defineConfig({
  plugins: [
    sveltepress({
      theme: defaultTheme({
        logo: '/logo.svg',
        github: 'https://github.com/oats-center/avena',
        editLink: 'https://github.com/oats-center/avena/edit/main/docs/src/routes/:route',
        navbar: [
          { title: 'Docs', to: '/docs/' },
          { title: 'Tutorials', to: '/docs/tutorials/01-tour/' },
          { title: 'Concepts', to: '/docs/concepts/architecture/' },
          { title: 'Reference', to: '/docs/reference/cli/' },
        ],
        sidebar: { '/docs/': sidebar },
        themeColor: {
          light: '#f7f5ee',
          dark: '#0f1511',
          primary: '#3f8a52',
          primaryDeep: '#2a6338',
          hover: '#d9a441',
          gradient: { start: '#3f8a52', end: '#d9a441' },
        },
        highlighter: {
          languages: ['bash', 'sh', 'console', 'toml', 'rust', 'ini', 'dockerfile', 'yaml', 'json', 'diff', 'svelte', 'ts', 'js', 'md', 'css'],
          themeLight: 'github-light',
          themeDark: 'github-dark-dimmed',
        },
        preBuildIconifyIcons: {
          tabler: ['box', 'refresh', 'clock-pause', 'arrow-back-up', 'topology-star-3', 'tools', 'check', 'terminal-2', 'truck', 'tractor', 'road', 'arrow-right'],
        },
      }),
      siteConfig: {
        title: 'Avena',
        description: 'The operating platform for OATS edge systems.',
      },
      svelteKitOptions: {
        preprocess: [vitePreprocess()],
        adapter: adapter({ pages: 'dist' }),
        paths: { base: '/avena', relative: false },
      },
    }),
  ],
})
