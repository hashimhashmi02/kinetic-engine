// A cycle rickshaw in cut paper, seen from the side and facing right.
// Local origin: the ground directly below the saddle. Pure function of the two angles.

const INK = "#1b1620";
const FRAME = "#2c3352";
const HOOD = "#8a4f48";
const SEAT = "#4f6a66";
const BONE = "#e2d6bf";

export const SADDLE_Y = -124;
export const CRANK = { x: 24, y: -74, r: 14 };

const shade = (inner) => `<g fill="#0b0810" stroke="#0b0810" opacity="0.28" transform="translate(6 8)">${inner}</g>`;
const tube = (x1, y1, x2, y2, w = 7, color = FRAME) => `<path d="M${x1},${y1} L${x2},${y2}" stroke="${color}" stroke-width="${w}" stroke-linecap="round" fill="none"/>`;

function wheel(cx, cy, r, deg) {
  let spokes = "";
  for (let i = 0; i < 6; i++) {
    const a = (i * Math.PI) / 6;
    spokes += `<path d="M${(-Math.cos(a) * r).toFixed(1)},${(-Math.sin(a) * r).toFixed(1)} L${(Math.cos(a) * r).toFixed(1)},${(Math.sin(a) * r).toFixed(1)}" stroke="${BONE}" stroke-width="1.6" opacity="0.8"/>`;
  }
  return (
    `<g transform="translate(${cx} ${cy})">` +
    `<circle r="${r}" fill="none" stroke="#0b0810" stroke-width="8" opacity="0.28" transform="translate(5 7)"/>` +
    `<g transform="rotate(${deg.toFixed(1)})">${spokes}</g>` +
    `<circle r="${r}" fill="none" stroke="${INK}" stroke-width="8"/><circle r="${r - 7}" fill="none" stroke="${BONE}" stroke-width="1.5" opacity="0.6"/>` +
    `<circle r="7" fill="${FRAME}"/></g>`
  );
}

export function rickshaw(wheelDeg, crankRad) {
  const hood = "M-212,-150 L-212,-236 Q-208,-302 -138,-304 L-78,-304 Q-52,-300 -50,-268 L-50,-258 L-190,-258 L-190,-150Z";
  const bench = "M-196,-158 h132 v44 h-132Z";
  const back = "M-200,-222 h22 v74 h-22Z";
  const foot = "M-70,-96 h82 v10 h-82Z";
  const saddle = `M-18,${SADDLE_Y - 8} h38 q6,0 6,6 v2 h-50 v-2 q0,-6 6,-6Z`;
  const px = Math.cos(crankRad) * CRANK.r, py = Math.sin(crankRad) * CRANK.r;

  let scallops = "";
  for (let x = -188; x < -56; x += 22) scallops += `<path d="M${x},-258 q11,16 22,0Z" fill="${BONE}"/>`;

  return (
    `<ellipse cx="10" cy="6" rx="190" ry="9" fill="#000" opacity="0.3"/>` +
    wheel(-120, -60, 60, wheelDeg) +
    shade(`<path d="${hood}"/><path d="${bench}"/>`) +
    `<path d="${back}" fill="${SEAT}"/><path d="${hood}" fill="${HOOD}"/>${scallops}` +
    `<path d="${bench}" fill="${SEAT}"/><path d="M-196,-158 h132 v9 h-132Z" fill="${BONE}" opacity="0.5"/>` +
    `<path d="${foot}" fill="${FRAME}"/>` +
    tube(-120, -60, -70, -114) + tube(-120, -60, CRANK.x, CRANK.y, 5) + tube(CRANK.x, CRANK.y, 0, SADDLE_Y) +
    tube(0, SADDLE_Y + 12, 112, -164) + tube(CRANK.x, CRANK.y, 114, -150) + tube(110, -184, 122, -138, 8) +
    wheel(150, -56, 56, wheelDeg * 1.07) +
    tube(122, -138, 150, -56, 6) + tube(110, -184, 88, -182, 6, INK) +
    `<path d="${saddle}" fill="${INK}"/>` +
    `<circle cx="${CRANK.x}" cy="${CRANK.y}" r="9" fill="${INK}"/>` +
    tube(CRANK.x - px, CRANK.y - py, CRANK.x + px, CRANK.y + py, 4, BONE) +
    `<path d="M${(CRANK.x + px - 9).toFixed(1)},${(CRANK.y + py).toFixed(1)} h18" stroke="${INK}" stroke-width="5" stroke-linecap="round"/>` +
    `<path d="M${(CRANK.x - px - 9).toFixed(1)},${(CRANK.y - py).toFixed(1)} h18" stroke="${INK}" stroke-width="5" stroke-linecap="round"/>`
  );
}
