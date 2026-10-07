<script lang="ts">
  // Inline stroke icons (no icon font, no network).
  let { name, size = 18 }: { name: string; size?: number } = $props();

  const P: Record<string, string> = {
    home: "M3 11.5 12 4l9 7.5M5.5 10v9.5h13V10",
    servers: "M4 5h16v5H4zM4 14h16v5H4zM8 7.5h.01M8 16.5h.01",
    library: "M12 17.3 6.2 20.5l1.1-6.5L2.6 9.4l6.5-.9L12 2.6l2.9 5.9 6.5.9-4.7 4.6 1.1 6.5z",
    mods: "M12 2 3 7v10l9 5 9-5V7zM3 7l9 5 9-5M12 12v10",
    account: "M12 12a4.5 4.5 0 1 0 0-9 4.5 4.5 0 0 0 0 9zM3.5 21a8.5 8.5 0 0 1 17 0",
    settings: "M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM19.4 13.5l1.6 1.2-2 3.5-1.9-.7a7 7 0 0 1-2.2 1.3L14.6 21h-4l-.4-2.2A7 7 0 0 1 8 17.5l-1.9.7-2-3.5 1.6-1.2a7 7 0 0 1 0-3l-1.6-1.2 2-3.5 1.9.7A7 7 0 0 1 10.2 5l.4-2h4l.4 2.2a7 7 0 0 1 2.2 1.3l1.9-.7 2 3.5-1.6 1.2a7 7 0 0 1 0 3z",
    play: "M7 4.5v15l12.5-7.5z",
    star: "M12 17.3 6.2 20.5l1.1-6.5L2.6 9.4l6.5-.9L12 2.6l2.9 5.9 6.5.9-4.7 4.6 1.1 6.5z",
    search: "M10.5 18a7.5 7.5 0 1 0 0-15 7.5 7.5 0 0 0 0 15zM21 21l-5.2-5.2",
    refresh: "M20 11a8 8 0 0 0-14.6-4.5L4 8M4 4v4h4M4 13a8 8 0 0 0 14.6 4.5L20 16M20 20v-4h-4",
    close: "M6 6l12 12M18 6 6 18",
    users: "M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM2 21a7 7 0 0 1 14 0M16 3.5a4 4 0 0 1 0 7.5M18.5 14a6 6 0 0 1 3.5 7",
    map: "M9 4 3 6.5v13.5L9 17.5l6 2.5 6-2.5V4l-6 2.5zM9 4v13.5M15 6.5V20",
    box: "M21 8 12 3 3 8v8l9 5 9-5zM3 8l9 5 9-5M12 13v8",
    lock: "M6 11h12v10H6zM8.5 11V7.5a3.5 3.5 0 0 1 7 0V11",
    signal: "M4 20v-3M9 20v-7M14 20v-11M19 20V4",
    link: "M10 14a4.5 4.5 0 0 0 6.4 0l3.2-3.2a4.5 4.5 0 0 0-6.4-6.4L12 5.6M14 10a4.5 4.5 0 0 0-6.4 0l-3.2 3.2a4.5 4.5 0 0 0 6.4 6.4l1.2-1.2",
    terminal: "M4 5h16v14H4zM7.5 9.5l3 2.5-3 2.5M12.5 15h4",
    folder: "M3 6.5h6l2 2h10V19H3z",
    copy: "M8 8h12v12H8zM4 16V4h12",
    check: "M4.5 12.5 9.5 17.5 19.5 6.5",
    alert: "M12 3 2 20h20zM12 10v4.5M12 17.5h.01",
    download: "M12 3v12.5M6.5 10l5.5 5.5 5.5-5.5M4 20.5h16",
    trash: "M4 7h16M9.5 7V4h5v3M6 7l1 13h10l1-13",
    stop: "M6 6h12v12H6z",
    chevron: "M9 6l6 6-6 6",
    globe: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM3 12h18M12 3c2.5 2.6 3.8 5.6 3.8 9s-1.3 6.4-3.8 9c-2.5-2.6-3.8-5.6-3.8-9S9.5 5.6 12 3z",
    logout: "M15 4h4v16h-4M10 8l-4 4 4 4M6 12h10",
    spark: "M12 2v5M12 17v5M2 12h5M17 12h5M5 5l3.5 3.5M15.5 15.5 19 19M19 5l-3.5 3.5M8.5 15.5 5 19",
    wrench: "M14.5 6.5a4 4 0 0 0 5 5L21 13l-8 8-3-3 8-8zM3 21l6-6",
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill={name === "play" || name === "star-fill" ? "currentColor" : "none"}
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={P[name === "star-fill" ? "star" : name] ?? ""} />
</svg>
