// Generates a 1024x1024 source PNG for `tauri icon`. No image dependencies —
// encodes a PNG by hand (zlib via node:zlib) so the build stays self-contained.
import { deflateSync } from "node:zlib";
import { writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const SIZE = 1024;
const out = join(dirname(fileURLToPath(import.meta.url)), "..", "src-tauri", "icons");
mkdirSync(out, { recursive: true });

const buf = Buffer.alloc(SIZE * SIZE * 4);
const set = (x, y, [r, g, b, a]) => {
  if (x < 0 || y < 0 || x >= SIZE || y >= SIZE) return;
  const i = (y * SIZE + x) * 4;
  buf[i] = r;
  buf[i + 1] = g;
  buf[i + 2] = b;
  buf[i + 3] = a;
};

const BG = [30, 30, 46, 255];
const CARD = [14, 99, 156, 255];
const CHECK = [78, 201, 106, 255];

// background
for (let y = 0; y < SIZE; y++) for (let x = 0; x < SIZE; x++) set(x, y, BG);

// rounded card
const m = 170;
const radius = 130;
const inRounded = (x, y) => {
  const x0 = m,
    y0 = m,
    x1 = SIZE - m,
    y1 = SIZE - m;
  if (x < x0 || y < y0 || x > x1 || y > y1) return false;
  const cx = Math.min(Math.max(x, x0 + radius), x1 - radius);
  const cy = Math.min(Math.max(y, y0 + radius), y1 - radius);
  return (x - cx) ** 2 + (y - cy) ** 2 <= radius ** 2;
};
for (let y = 0; y < SIZE; y++)
  for (let x = 0; x < SIZE; x++) if (inRounded(x, y)) set(x, y, CARD);

// checkmark (two thick segments)
const distSeg = (px, py, ax, ay, bx, by) => {
  const dx = bx - ax,
    dy = by - ay;
  const t = Math.max(0, Math.min(1, ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)));
  const cx = ax + t * dx,
    cy = ay + t * dy;
  return Math.hypot(px - cx, py - cy);
};
const thick = 46;
for (let y = 0; y < SIZE; y++)
  for (let x = 0; x < SIZE; x++) {
    const d = Math.min(
      distSeg(x, y, 360, 520, 470, 640),
      distSeg(x, y, 470, 640, 690, 380),
    );
    if (d <= thick) set(x, y, CHECK);
  }

writeFileSync(join(out, "source.png"), encodePng(buf, SIZE, SIZE));
console.log("wrote src-tauri/icons/source.png");

function encodePng(rgba, w, h) {
  const raw = Buffer.alloc((w * 4 + 1) * h);
  for (let y = 0; y < h; y++) {
    raw[y * (w * 4 + 1)] = 0; // filter: none
    rgba.copy(raw, y * (w * 4 + 1) + 1, y * w * 4, (y + 1) * w * 4);
  }
  const idat = deflateSync(raw, { level: 9 });
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0);
  ihdr.writeUInt32BE(h, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // color type RGBA
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", idat),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

function chunk(type, data) {
  const t = Buffer.from(type, "ascii");
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([t, data])) >>> 0, 0);
  return Buffer.concat([len, t, data, crc]);
}

function crc32(buf) {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return ~c;
}
