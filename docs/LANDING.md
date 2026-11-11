# Landing page: "Kin in Purani Dilli"

The page is a scroll-driven 3D journey. The camera travels one continuous route through a stylised Old Delhi, from the mouth of Chandni Chowk, down Nai Sarak into Chawri Bazar, and up the steps to Jama Masjid. Kin, the character, makes the trip with us, and each stretch of street shows one thing the engine does.

Reference for the feel: a scroll fly-through site where every section is a place the camera moves into, with a single caption near the bottom of the screen and almost no other UI. Ours differs in having one connected world instead of separate scenes, and a character who carries the story.

## Principles

- **Scroll is the playhead.** Scroll position maps to a time `t` on one master timeline. Camera, character, light and props are all functions of `t`, so scrolling back rewinds everything exactly. This is the engine's central idea, shown by the page itself.
- **One place, one route, one character.** No floating abstract shapes. Everything on screen belongs to the street: shutters, rickshaws, wires, signboards, brass pots, paper, pigeons, kites.
- **The street demonstrates the feature.** A spring is a hanging signboard that bounces. A morph is brass being beaten into a new shape. Text morphing is a painted signboard changing script.
- **Little UI.** A wordmark, one button, a caption block low on the screen, and the timeline bar. The world does the rest.
- **Affection and accuracy.** This is a real neighbourhood. Get the character of the place right (see "Getting the place right") and keep it warm, never a caricature.
- **Built to become a portfolio.** Shopfronts along the route are slots. Later, each can hold a project: the camera stops, the shutter rolls up, the work is inside.

## Kin

Approved design: see `design/kin/sheet.html` (rig in `design/kin/kin.js`). An eleven-year-old from a gali off Chawri Bazar, drawn as a layered paper cut-out: big soft head, small body in a slate-blue kurta, white and grey scarf, thin limbs, a red kite on the back. No outlines; edges come from the shadow each sheet casts on the one below. The kite is the only saturated colour in a frame. Emotion comes from posture, timing and small dot eyes.

The rig is 2D and pose-driven (a pose is data, drawing is a pure function of it). In the street Kin is either a camera-facing layered cut-out placed in the 3D scene, or the whole world is built as layered paper to match; this is decided when the grey-box street exists. Paper grain must be baked into textures for the page, since live SVG filters are too slow.

Kin's actions are clips and procedural poses blended by the master timeline: `idle`, `notice`, `walk`, `pedal`, `hop`, `reach`, `duck`, `carry`, `look-up`, `sit`, `wave`. Scarf and arms get follow-through from a lagged secondary motion that is itself a function of `t` (computed from position at `t - dt`), so it stays scrubbable.

Animation principles to honour: anticipation, squash and stretch on hops and landings, follow-through on the scarf, arcs, eyes lead the head and the head leads the body, holds so poses read.

## The route

| # | Place | Time of day | What happens | Engine idea | Code shown |
|---|---|---|---|---|---|
| 1 | **Mouth of Chandni Chowk** | Before dawn, blue haze | Wide establishing shot down the empty street: shutters down, wires overhead, the far end lost in mist. Kin stands frozen mid-step like a paused frame. Caption invites the scroll. On first scroll Kin blinks and looks at us. | Scene and nodes | `s.add(...)` |
| 2 | **Chandni Chowk** | Sunrise, warm side light | Kin climbs onto a cycle rickshaw and pedals. The ride eases in and out; ghost frames of the rickshaw show the spacing. Shutters roll up one after another as we pass (stagger). Chai steam, a paratha stall, the first crowd silhouettes. | Tween, easing, stagger | `s.play(rick.move_to(..), 2.0, ease::in_out_cubic)` |
| 3 | **Under the wires** | Morning | The camera tilts up into the canopy of cables, hanging signboards, lanterns and bunting. Kin hops off; everything they brush past swings and settles. A signboard has two dials (stiffness, damping) the visitor can turn; its bounce and a plotted response curve update live. Kites tug on their strings above. | Closed-form springs | `spring().stiffness(k).damping(c)` |
| 4 | **Nai Sarak into Chawri Bazar** | Midday, the lane narrows | Brass and copper shops. A craftsman's hammer falls and a brass disc becomes a lota, then a bell, then a lamp. Kin carries a stack of vessels that reshape in their arms. Light glints travel across the metal. | Path morphing | `shape.morph_to(...)` |
| 5 | **Chawri Bazar paper market** | Afternoon, dusty gold | Wedding cards, paper reams, hand-painted signboards. A painter's board morphs "नमस्ते" to "آداب" to "Hello"; loose letters blow off a stall and Kin catches them into a word. | Text as paths, shaping | `title.text_to("Hello")` |
| 6 | **Steps of Jama Masjid** | Golden hour to dusk | The lane opens onto the wide steps and the sky. Pigeons lift off in a flock; scrolling back lands every bird where it started. Kin notices the timeline bar, sits on a step and drags the playhead: the sun moves, shadows swing, the flock reverses. The visitor can drag too. | Every frame is a pure function of `t` | `timeline.evaluate(t)` |
| 7 | **Dusk over the old city** | Blue hour, lamps on | The camera rises to see the domes and minarets in silhouette against the sky, kites and strings of lights over the rooftops. The view letterboxes into a film frame; a render bar fills; it becomes a finished clip. Kin waves. Call to action. | Export | `kinetic render scene -o film.mp4` |

After the journey: a plain, fast section with the install command, a quickstart snippet, the feature list and links. Footer.

Later (portfolio phase): chapters 2 to 5 gain named shopfronts. Each is a stop with a title painted on its signboard; entering one opens a project view. The route, camera system and timeline bar do not change.

## Getting the place right

Collect photo references for each stretch before modelling, and keep them in `site/reference/` (not shipped). Things that make it read as Old Delhi and not a generic bazaar:

- Dense overhead cable tangles strung between poles and balconies.
- Narrow multi-storey buildings with projecting balconies, jharokhas, carved brackets, peeling paint, rolling steel shutters, and layers of signboards in Devanagari, Urdu and English.
- Cycle rickshaws, handcarts, sacks and crates stacked at shop edges, awnings and tarpaulins.
- Chawri Bazar specifically: brass, copper and paper trade, wedding card shops.
- Jama Masjid: red sandstone and white marble, three domes with black stripes, two tall minarets, the broad flight of steps.

Jama Masjid is a working place of worship. Show its exterior, steps and skyline with care. No jokes at its expense, no characters climbing on it, nothing inside the prayer hall. Kin sits quietly on the public steps. Signboard text in Hindi and Urdu must be real, correctly shaped words reviewed by a reader of each script; the user should confirm these before launch.

## Visual direction

- Stylised low-poly with soft gradients and fog, not photoreal. Depth comes from haze, light direction and layered silhouettes.
- Light is the colour script: blue pre-dawn, peach sunrise, white noon, gold afternoon, amber dusk, deep blue night with warm lamps. Sky, fog colour and sun angle are tracks on the master timeline.
- Captions: small eyebrow label, one or two lines of body text, low centre, with a soft shadow for legibility over any scene.
- Type: one display face with character for the wordmark and chapter names, a clean sans for captions, a mono for code. Self-hosted and subset, including the Devanagari and Arabic-script glyphs used on signboards if rendered as live text.
- Decide exact tokens with the `impeccable` and `frontend-design` skills at build time and record them in `site/src/styles/tokens.css`.

## Technical approach

- **Astro** static site. **Three.js** for the world, **GSAP + ScrollTrigger** for mapping scroll to master time and for DOM captions.
- **Master timeline.** `site/src/world/timeline.ts` defines chapters with start and end times. Everything animated exposes `update(t)`. No per-frame state that depends on the previous frame, except explicitly lagged values computed from `t - dt`.
- **Camera rail.** A Catmull-Rom spline for position and a second for the look-at target, with per-chapter easing so the camera slows at stops. Small pointer parallax is added on top and does not affect `t`.
- **World building.** Modular kit: a few building modules, shutters, balconies, signboards, poles, awnings, stalls, combined procedurally along the route with a seeded random generator. Instanced meshes for repeats. Cables are catenary curves generated between anchor points. Hero pieces (rickshaw, the mosque, Kin) are modelled individually, as glTF with Draco compression or built in code.
- **Streaming.** Only the stretch near the camera is in the scene graph; chapters ahead load during the current one. First paint needs only chapter 1.
- **Lighting.** One directional sun, hemisphere light, fog. Lamps as emissive materials with a bloom pass after dusk. Shadows baked into vertex colours or a single cascaded shadow map on capable devices.
- **Springs on the page** use the same closed-form solution as the engine (ported to TypeScript from `kinetic-core`), so the interactive signboard is an honest demo.
- **Fallbacks.** No WebGL, low-power devices, or `prefers-reduced-motion`: a sequence of pre-rendered stills of each chapter with the same captions and code, no pinning or scrubbing. Phones get a lighter world (fewer instances, no post-processing, lower pixel ratio) and shorter scroll distance.
- **Accessibility.** Captions and code are real DOM text in reading order. Chapter markers on the timeline bar are buttons. The dials are native range inputs. A "skip the journey" link jumps to the plain section.
- **Budgets.** Under 1.5 MB transferred for the first chapter, 60 fps on a mid-range laptop and 30 fps or better on a recent phone, pixel ratio capped at 2.
- Base path configurable for GitHub Pages (`/kinetic-engine/`).

## Build order

1. Scroll to master time, camera rail and timeline bar, in a grey-box street made of cubes.
2. Building kit and procedural street for chapter 2, with fog and the light script.
3. Kin model and action set; the rickshaw.
4. Chapters in route order, each finished (motion, caption, code panel, reverse scrub) before the next begins.
5. Fallback stills, phone tuning, accessibility pass, performance pass.
6. Deploy to GitHub Pages.

## Done when

- Scrolling forwards and backwards through all seven chapters shows no pops, teleports or state that fails to rewind.
- Each chapter's caption, code snippet and street action clearly show the same engine idea.
- Frame rate and transfer budgets are met on a laptop and a phone, checked in a browser.
- The fallback path works with WebGL disabled and with reduced motion.
- Script text on signboards has been confirmed by the user.
- The page is live on GitHub Pages and linked from the README.
