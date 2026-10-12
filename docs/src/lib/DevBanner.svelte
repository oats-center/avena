<div class="avena-dev-banner" role="note">
  <span class="dot" aria-hidden="true"></span>
  <span>Avena is in active development.<span class="long"> Not everything described here is implemented yet.</span></span>
</div>

<style>
  .avena-dev-banner {
    position: fixed;
    inset: 0 0 auto 0;
    z-index: 1100;
    height: var(--avena-banner-h);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    padding: 0 1rem;
    font-size: 0.8rem;
    font-weight: 500;
    color: #3d2c00;
    background: repeating-linear-gradient(-45deg, #f2cf73 0 14px, #eec55e 14px 28px);
    white-space: nowrap;
    overflow: hidden;
  }
  @media (max-width: 640px) {
    .long { display: none; }
  }
  .dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    background: #8a5a00;
    flex: none;
    animation: pulse 2s ease-in-out infinite;
  }
  @keyframes pulse {
    50% { opacity: 0.3; }
  }
  @media (prefers-reduced-motion: reduce) {
    .dot { animation: none; }
  }
</style>
