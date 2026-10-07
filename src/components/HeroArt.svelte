<script lang="ts">
  // Night highway, drawn in SVG: skyline, a road vanishing into the
  // distance, and long-exposure light trails. No bitmap assets to ship.
  const buildings = Array.from({ length: 46 }, (_, i) => {
    const w = 22 + ((i * 37) % 40);
    const h = 60 + ((i * 53) % 170);
    return { x: i * 34 - 20, w, h };
  });
  const windows = buildings.flatMap((b, i) =>
    Array.from({ length: 8 }, (_, j) => ({
      x: b.x + 5 + ((j * 7) % Math.max(6, b.w - 8)),
      y: 380 - b.h + 10 + ((j * 23 + i * 11) % Math.max(10, b.h - 20)),
      on: (i * 7 + j * 3) % 5 === 0,
    })),
  );
</script>

<svg class="hero-art" viewBox="0 0 1600 600" preserveAspectRatio="xMidYMid slice" aria-hidden="true">
  <defs>
    <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#05050a" />
      <stop offset=".55" stop-color="#1a0a1c" />
      <stop offset="1" stop-color="#3a0f24" />
    </linearGradient>
    <radialGradient id="glow" cx=".5" cy=".62" r=".55">
      <stop offset="0" stop-color="#ff2e63" stop-opacity=".45" />
      <stop offset="1" stop-color="#ff2e63" stop-opacity="0" />
    </radialGradient>
    <linearGradient id="road" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#0d0d14" />
      <stop offset="1" stop-color="#050508" />
    </linearGradient>
    <linearGradient id="trailR" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0" stop-color="#ff2e63" stop-opacity="0" />
      <stop offset=".6" stop-color="#ff2e63" stop-opacity=".9" />
      <stop offset="1" stop-color="#ff8a00" />
    </linearGradient>
    <linearGradient id="trailW" x1="1" y1="0" x2="0" y2="0">
      <stop offset="0" stop-color="#2ee6ff" stop-opacity="0" />
      <stop offset=".6" stop-color="#bff6ff" stop-opacity=".8" />
      <stop offset="1" stop-color="#ffffff" />
    </linearGradient>
    <filter id="blur"><feGaussianBlur stdDeviation="3" /></filter>
    <filter id="blurBig"><feGaussianBlur stdDeviation="14" /></filter>
  </defs>

  <rect width="1600" height="600" fill="url(#sky)" />
  <rect width="1600" height="600" fill="url(#glow)" />

  <!-- skyline -->
  <g fill="#0a0a12">
    {#each buildings as b}
      <rect x={b.x} y={380 - b.h} width={b.w} height={b.h + 40} />
    {/each}
  </g>
  <g>
    {#each windows as w}
      {#if w.on}<rect x={w.x} y={w.y} width="3" height="4" fill="#ffb86b" opacity=".55" />{/if}
    {/each}
  </g>

  <!-- road -->
  <path d="M0 600 L700 382 L900 382 L1600 600 Z" fill="url(#road)" />
  <path d="M0 600 L700 382 M1600 600 L900 382" stroke="#ff2e63" stroke-opacity=".35" stroke-width="2" />
  {#each Array.from({ length: 9 }, (_, i) => i) as i}
    <path
      d={`M${800 - 2 - i * 3} ${388 + i * i * 2.6} l${4 + i * 6} 0 l${2 + i * 2} ${8 + i * 6} l${-(8 + i * 10)} 0 z`}
      fill="#ffe7a3"
      opacity={0.18 + i * 0.05}
    />
  {/each}

  <!-- light trails -->
  <g filter="url(#blurBig)" opacity=".7">
    <path d="M-50 560 C 400 470, 620 410, 790 386" stroke="url(#trailR)" stroke-width="22" fill="none" />
    <path d="M1650 575 C 1200 470, 980 410, 812 386" stroke="url(#trailW)" stroke-width="22" fill="none" />
  </g>
  <g filter="url(#blur)">
    <path d="M-50 560 C 400 470, 620 410, 790 386" stroke="url(#trailR)" stroke-width="5" fill="none" />
    <path d="M-50 590 C 380 490, 610 420, 785 388" stroke="url(#trailR)" stroke-width="3" fill="none" opacity=".7" />
    <path d="M1650 575 C 1200 470, 980 410, 812 386" stroke="url(#trailW)" stroke-width="5" fill="none" />
    <path d="M1650 545 C 1220 455, 990 405, 816 384" stroke="url(#trailW)" stroke-width="3" fill="none" opacity=".6" />
  </g>

  <rect width="1600" height="600" fill="url(#sky)" opacity=".25" />
</svg>

<style>
  .hero-art {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
</style>
