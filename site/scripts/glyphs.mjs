// Shapes the signboard words with HarfBuzz and writes their outlines as polylines to
// src/world/words.json, so the page can morph real glyph shapes between scripts.
//
//   node scripts/glyphs.mjs
//
// The output is committed; rerun only when the words or fonts change.

import fs from "node:fs";
import zlib from "node:zlib";
import { fileURLToPath } from "node:url";
import * as hb from "harfbuzzjs";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
const FONTS = "../node_modules/@fontsource";
const WORDS = [
  { text: "नमस्ते", font: `${FONTS}/rozha-one/files/rozha-one-devanagari-400-normal.woff` },
  { text: "آداب", font: `${FONTS}/noto-nastaliq-urdu/files/noto-nastaliq-urdu-arabic-400-normal.woff` },
  { text: "Hello", font: `${FONTS}/rozha-one/files/rozha-one-latin-400-normal.woff` },
];
const BOX = { w: 300, h: 112 };

/** HarfBuzz wants a plain sfnt; the font packages ship WOFF, which is the same tables deflated. */
function woffToSfnt(woff) {
  if (woff.toString("latin1", 0, 4) !== "wOFF") throw new Error("not a WOFF file");
  const numTables = woff.readUInt16BE(12);
  const tables = [];
  for (let i = 0; i < numTables; i++) {
    const at = 44 + i * 20;
    const offset = woff.readUInt32BE(at + 4), compLength = woff.readUInt32BE(at + 8), origLength = woff.readUInt32BE(at + 12);
    const raw = woff.subarray(offset, offset + compLength);
    tables.push({ tag: woff.subarray(at, at + 4), checksum: woff.readUInt32BE(at + 16), data: compLength < origLength ? zlib.inflateSync(raw) : raw });
  }
  const entrySelector = Math.floor(Math.log2(numTables));
  const header = Buffer.alloc(12 + numTables * 16);
  woff.copy(header, 0, 4, 8);
  header.writeUInt16BE(numTables, 4);
  header.writeUInt16BE(2 ** entrySelector * 16, 6);
  header.writeUInt16BE(entrySelector, 8);
  header.writeUInt16BE(numTables * 16 - 2 ** entrySelector * 16, 10);
  let offset = header.length;
  const parts = [header];
  tables.forEach((table, i) => {
    const at = 12 + i * 16;
    table.tag.copy(header, at);
    header.writeUInt32BE(table.checksum, at + 4);
    header.writeUInt32BE(offset, at + 8);
    header.writeUInt32BE(table.data.length, at + 12);
    const padded = Buffer.alloc(Math.ceil(table.data.length / 4) * 4);
    table.data.copy(padded);
    parts.push(padded);
    offset += padded.length;
  });
  return Buffer.concat(parts);
}

/** Flattens an SVG path (M, L, Q, C, Z with absolute coordinates) into closed polylines. */
function flatten(path, steps = 7) {
  const tokens = path.match(/[MLQCZ]|-?\d*\.?\d+(?:e[-+]?\d+)?/g) || [];
  const contours = [];
  let current = null, pen = [0, 0], i = 0;
  const num = () => parseFloat(tokens[i++]);
  const point = () => [num(), num()];
  while (i < tokens.length) {
    const cmd = tokens[i++];
    if (cmd === "M") {
      current = [point()];
      contours.push(current);
      pen = current[0];
    } else if (cmd === "L") {
      pen = point();
      current.push(pen);
    } else if (cmd === "Q") {
      const c = point(), e = point();
      for (let k = 1; k <= steps; k++) {
        const u = k / steps, v = 1 - u;
        current.push([v * v * pen[0] + 2 * v * u * c[0] + u * u * e[0], v * v * pen[1] + 2 * v * u * c[1] + u * u * e[1]]);
      }
      pen = e;
    } else if (cmd === "C") {
      const c1 = point(), c2 = point(), e = point();
      for (let k = 1; k <= steps; k++) {
        const u = k / steps, v = 1 - u;
        current.push([
          v * v * v * pen[0] + 3 * v * v * u * c1[0] + 3 * v * u * u * c2[0] + u * u * u * e[0],
          v * v * v * pen[1] + 3 * v * v * u * c1[1] + 3 * v * u * u * c2[1] + u * u * u * e[1],
        ]);
      }
      pen = e;
    } else if (cmd !== "Z") {
      throw new Error("unexpected path token " + cmd);
    }
  }
  return contours.filter((c) => c.length > 2);
}

const out = [];
for (const word of WORDS) {
  const sfnt = woffToSfnt(fs.readFileSync(here(word.font)));
  const font = new hb.Font(new hb.Face(new hb.Blob(sfnt.buffer.slice(sfnt.byteOffset, sfnt.byteOffset + sfnt.length))));
  const buffer = new hb.Buffer();
  buffer.addText(word.text);
  buffer.guessSegmentProperties();
  hb.shape(font, buffer);
  const infos = buffer.getGlyphInfos(), positions = buffer.getGlyphPositions();
  if (infos.some((g) => g.codepoint === 0)) throw new Error(`font is missing a glyph for "${word.text}"`);

  let contours = [], x = 0, y = 0;
  infos.forEach((glyph, k) => {
    const pos = positions[k];
    for (const c of flatten(font.glyphToPath(glyph.codepoint))) {
      contours.push(c.map(([px, py]) => [x + pos.xOffset + px, -(y + pos.yOffset + py)]));
    }
    x += pos.xAdvance;
    y += pos.yAdvance;
  });

  // Centre the word and scale it to the board.
  const xs = contours.flat().map((p) => p[0]), ys = contours.flat().map((p) => p[1]);
  const [x0, x1, y0, y1] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
  const scale = Math.min(BOX.w / (x1 - x0), BOX.h / (y1 - y0));
  contours = contours.map((c) => c.map(([px, py]) => [+((px - (x0 + x1) / 2) * scale).toFixed(1), +((py - (y0 + y1) / 2) * scale).toFixed(1)]));
  out.push({ text: word.text, glyphs: infos.length, contours });
  console.log(`${word.text}: ${infos.length} glyphs, ${contours.length} contours`);
}

fs.writeFileSync(here("../src/world/words.json"), JSON.stringify(out) + "\n");
