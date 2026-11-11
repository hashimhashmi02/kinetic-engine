// The Three.js layer: things that fly. Pigeons with hinged wings, kites folded along the
// spine, and drifting flecks of paper live in real depth over the SVG street, so they
// shrink and slow with distance instead of faking it.
//
// Scene units are stage units with y flipped (SVG y grows down). An object at z = 0 lines
// up with the near street layer exactly; negative z is further away. Like everything
// else on the page, every position is a function of the master time t.

import * as THREE from "three";
import { clamp, lerp, outCubic, rng, smooth, span } from "./math.js";

const DIST = 1500;
const PAPER = { side: THREE.DoubleSide };

function tri(points, color) {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.Float32BufferAttribute(points.flat(), 3));
  return new THREE.Mesh(geometry, new THREE.MeshBasicMaterial({ color, ...PAPER }));
}

function makeBird() {
  const bird = new THREE.Group();
  bird.add(tri([[-13, 0, 0], [0, 5, 0], [0, -4, 0], [0, 5, 0], [15, 1, 0], [0, -4, 0]], "#9a9aa6"));
  bird.add(tri([[13, 4, 0], [20, 3, 0], [14, 0, 0]], "#8a8a96"));
  // Wings are hinged along the body and reach toward and away from the camera.
  const wingA = tri([[-7, 0, 0], [8, 0, 0], [-1, 0, 22]], "#b4b4c0");
  const wingB = tri([[-7, 0, 0], [8, 0, 0], [-1, 0, -22]], "#77788a");
  bird.add(wingA, wingB);
  bird.userData = { wingA, wingB };
  return bird;
}

function makeKite(color, dark) {
  const kite = new THREE.Group();
  const half = (side, tone) => {
    const mesh = tri([[0, 46, 0], [side * 38, 6, 0], [0, -46, 0]], tone);
    mesh.rotation.y = side * 0.38; // folded along the spine
    return mesh;
  };
  kite.add(half(-1, color), half(1, dark));
  kite.add(tri([[-7, -40, 0], [7, -40, 0], [0, -64, 0]], "#e2d6bf"));
  return kite;
}

export function createSky(canvas, steps) {
  const renderer = new THREE.WebGLRenderer({ canvas, alpha: true, antialias: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(30, 1, 10, 6000);
  let visibleHeight = 900;

  // Pigeons on the mosque steps.
  const r = rng(77);
  const birds = [];
  for (let i = 0; i < 30; i++) {
    const step = steps[r.int(0, steps.length - 1)];
    const mesh = makeBird();
    mesh.scale.setScalar(r.range(0.8, 1.15));
    scene.add(mesh);
    birds.push({
      mesh, x: r.range(step.x0, step.x1), y: step.y - 5, z: r.range(-40, 60),
      dx: r.range(-460, 560), dy: r.range(260, 640), dz: r.range(-700, 260), delay: r.range(0, 0.26), flip: r() < 0.5 ? -1 : 1, phase: r.range(0, 6),
    });
  }

  // Kites: a few over the whole route, and a sky full of them at dusk.
  const tones = [["#c9973f", "#a3782c"], ["#5f8f8a", "#487370"], ["#d9c48f", "#b7a26f"], ["#7a5f8f", "#5f4873"], ["#d8402f", "#a92c22"], ["#9fb3a8", "#7f958a"]];
  const kites = [];
  const strings = [];
  for (let i = 0; i < 22; i++) {
    const dusk = i >= 9;
    const [color, dark] = tones[i % tones.length];
    const mesh = makeKite(color, dark);
    const kite = {
      mesh, dusk,
      x: dusk ? r.range(9500, 11100) : 1300 + i * 1050 + r.range(-200, 200),
      y: r.range(150, 330), z: dusk ? r.range(-900, -120) : r.range(-1100, -300),
      phase: r.range(0, 6), size: r.range(0.8, 1.3), delay: r.range(0, 0.4),
    };
    mesh.scale.setScalar(kite.size);
    scene.add(mesh);
    const line = new THREE.Line(
      new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(), new THREE.Vector3()]),
      new THREE.LineBasicMaterial({ color: "#efe7d6", transparent: true, opacity: 0.45 }),
    );
    scene.add(line);
    kites.push(kite);
    strings.push(line);
  }

  // Paper flecks.
  const COUNT = 80;
  const flecks = new THREE.InstancedMesh(new THREE.PlaneGeometry(9, 5), new THREE.MeshBasicMaterial({ ...PAPER, transparent: true, opacity: 0.75 }), COUNT);
  const fleckData = [];
  const tint = new THREE.Color();
  for (let i = 0; i < COUNT; i++) {
    fleckData.push({ x: r.range(0, 2600), y: r.range(140, 820), z: r.range(-700, 420), v: r.range(260, 640), ph: r.range(0, 6), s: r.range(0.5, 1.4) });
    flecks.setColorAt(i, tint.set(i % 6 ? "#efe7d6" : "#d8402f"));
  }
  flecks.frustumCulled = false;
  scene.add(flecks);
  const dummy = new THREE.Object3D();

  function resize() {
    const w = canvas.clientWidth, h = canvas.clientHeight;
    if (!w || !h) return;
    renderer.setSize(w, h, false);
    // Match the SVG's "xMidYMax slice" framing at z = 0.
    visibleHeight = h / Math.max(w / 1600, h / 900);
    camera.aspect = w / h;
    camera.fov = (2 * Math.atan(visibleHeight / 2 / DIST) * 180) / Math.PI;
    camera.updateProjectionMatrix();
  }
  resize();

  function render(t, cam, rise) {
    camera.position.set(cam + 800, -(900 - visibleHeight / 2 + rise), DIST);

    for (const b of birds) {
      const e = outCubic(span(t, 5.42 + b.delay, 5.98 + b.delay));
      const flying = e > 0;
      b.mesh.position.set(b.x + b.dx * e, -(b.y - b.dy * e - Math.sin(e * Math.PI) * 50), b.z + b.dz * e);
      b.mesh.rotation.set(0, b.flip < 0 ? Math.PI : 0, flying ? 0.35 * (1 - e) : 0);
      const flap = flying ? Math.sin(e * 46 + b.phase) * 1.1 : -1.35;
      b.mesh.userData.wingA.rotation.x = -flap;
      b.mesh.userData.wingB.rotation.x = flap;
    }

    kites.forEach((k, i) => {
      const up = k.dusk ? smooth(span(t, 6.2 + k.delay, 6.75 + k.delay)) : 1;
      const sway = Math.sin(t * 4.2 + k.phase), bob = Math.cos(t * 3.1 + k.phase);
      const x = k.x + sway * 26, y = lerp(700, k.y, up) + bob * 14;
      k.mesh.visible = up > 0.001;
      k.mesh.position.set(x, -y, k.z);
      k.mesh.rotation.set(0.25 + bob * 0.12, sway * 0.5, sway * 0.3 - 0.2);
      k.mesh.scale.setScalar(k.size * clamp(up * 1.4));
      const pts = strings[i].geometry.attributes.position;
      pts.setXYZ(0, x, -y - 40 * k.size, k.z);
      pts.setXYZ(1, k.x - 110, -330, k.z); // down to the rooftops
      pts.needsUpdate = true;
      strings[i].visible = k.mesh.visible;
    });

    fleckData.forEach((f, i) => {
      const x = cam - 500 + ((((f.x - t * f.v) % 2600) + 2600) % 2600);
      dummy.position.set(x, -(f.y + Math.sin(t * 2.3 + f.ph) * 46), f.z);
      dummy.rotation.set(t * 3 * f.s + f.ph, t * 2 * f.s, t * 2.4 + f.ph);
      dummy.scale.setScalar(f.s);
      dummy.updateMatrix();
      flecks.setMatrixAt(i, dummy.matrix);
    });
    flecks.instanceMatrix.needsUpdate = true;

    renderer.render(scene, camera);
  }

  return { render, resize };
}
