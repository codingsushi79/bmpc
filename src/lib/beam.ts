// BeamMP text helpers: colour codes, map names, sizes, flags.

export interface Segment {
  text: string;
  color?: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strike?: boolean;
}

// BeamMP uses Minecraft-style codes. The dark ones are lifted so they stay
// readable on a dark launcher.
const COLORS: Record<string, string> = {
  "0": "#9a9ab0",
  "1": "#5b7cff",
  "2": "#2fcf6a",
  "3": "#22c7d6",
  "4": "#ff4d4d",
  "5": "#c05bff",
  "6": "#ffb020",
  "7": "#c8c8d8",
  "8": "#8a8aa0",
  "9": "#6f8dff",
  a: "#5dff8a",
  b: "#5df3ff",
  c: "#ff6b6b",
  d: "#ff7ad9",
  e: "#ffe45d",
  f: "#ffffff",
};

/** Split `^1Red ^lbold^r text` into styled runs. `^p` and `\n` break lines. */
export function parseCodes(input: string): Segment[] {
  const out: Segment[] = [];
  let style: Omit<Segment, "text"> = {};
  let buf = "";
  const flush = () => {
    if (buf) out.push({ text: buf, ...style });
    buf = "";
  };
  for (let i = 0; i < input.length; i++) {
    const c = input[i];
    if (c === "^" && i + 1 < input.length) {
      const code = input[i + 1].toLowerCase();
      if (code in COLORS || "lmnorp".includes(code)) {
        flush();
        i++;
        if (code in COLORS) style = { ...style, color: COLORS[code] };
        else if (code === "l") style = { ...style, bold: true };
        else if (code === "o") style = { ...style, italic: true };
        else if (code === "n") style = { ...style, underline: true };
        else if (code === "m") style = { ...style, strike: true };
        else if (code === "r") style = {};
        else if (code === "p") buf += "\n";
        continue;
      }
    }
    buf += c;
  }
  flush();
  return out;
}

export function plain(input: string): string {
  return parseCodes(input)
    .map((s) => s.text)
    .join("")
    .trim();
}

const MAP_NAMES: Record<string, string> = {
  gridmap_v2: "Grid Map",
  west_coast_usa: "West Coast USA",
  east_coast_usa: "East Coast USA",
  utah: "Utah",
  italy: "Italy",
  jungle_rock_island: "Jungle Rock Island",
  industrial: "Industrial Site",
  small_island: "Small Island",
  hirochi_raceway: "Hirochi Raceway",
  automation_test_track: "Automation Test Track",
  johnson_valley: "Johnson Valley",
  derby: "Derby Arenas",
  driver_training: "Driver Training",
  cliff: "Cliff",
  smallgrid: "Small Grid",
  west_coast_usa_v2: "West Coast USA",
};

export function mapName(map: string): string {
  if (MAP_NAMES[map]) return MAP_NAMES[map];
  return map
    .replace(/[_-]+/g, " ")
    .replace(/\b\w/g, (c) => c.toUpperCase())
    .trim() || "Unknown map";
}

/** A stable colour per map so cards are recognisable at a glance. */
export function mapHue(map: string): number {
  let h = 0;
  for (let i = 0; i < map.length; i++) h = (h * 31 + map.charCodeAt(i)) % 360;
  return h;
}

export function bytes(n: number): string {
  if (!n) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v >= 100 || i === 0 ? v.toFixed(0) : v.toFixed(1)} ${units[i]}`;
}

/** ISO country code → flag emoji. */
export function flag(code: string): string {
  const cc = (code || "").trim().toUpperCase();
  if (!/^[A-Z]{2}$/.test(cc)) return "🌐";
  return String.fromCodePoint(...[...cc].map((c) => 0x1f1e6 + c.charCodeAt(0) - 65));
}

export const REGIONS: Record<string, string> = {
  NA: "North America",
  EU: "Europe",
  AS: "Asia",
  OC: "Oceania",
  SA: "South America",
  AF: "Africa",
};

const CONTINENT: Record<string, string> = {};
for (const cc of "US CA MX".split(" ")) CONTINENT[cc] = "NA";
for (const cc of "GB UK DE FR NL BE LU IT ES PT PL CZ SK AT CH SE NO FI DK IE IS EE LV LT HU RO BG GR HR SI RS BA ME MK AL UA BY MD RU TR CY MT".split(" ")) CONTINENT[cc] = "EU";
for (const cc of "JP KR CN TW HK SG MY TH VN PH ID IN PK BD AE SA IL QA KZ".split(" ")) CONTINENT[cc] = "AS";
for (const cc of "AU NZ".split(" ")) CONTINENT[cc] = "OC";
for (const cc of "BR AR CL CO PE UY VE EC BO PY".split(" ")) CONTINENT[cc] = "SA";
for (const cc of "ZA EG NG KE MA TN DZ".split(" ")) CONTINENT[cc] = "AF";

export function region(code: string): string {
  return CONTINENT[(code || "").toUpperCase()] ?? "??";
}

export function serverKey(s: { ip: string; port: number }): string {
  return `${s.ip}:${s.port}`;
}

export function ago(ms: number): string {
  const s = Math.max(0, (Date.now() - ms) / 1000);
  if (s < 60) return "just now";
  if (s < 3600) return `${Math.floor(s / 60)} min ago`;
  if (s < 86400) return `${Math.floor(s / 3600)} h ago`;
  return `${Math.floor(s / 86400)} d ago`;
}
