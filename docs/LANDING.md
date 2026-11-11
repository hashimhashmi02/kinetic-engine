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

## Technical approach (as built)

The street is layered cut paper, to match Kin, so it is drawn as SVG. Things that fly are drawn in Three.js over it.

- **Astro** static site in `site/`. **GSAP ScrollTrigger** scrubs one number, the master time `t`, from the scroll position; `render(t)` in `site/src/world/main.js` draws everything from `t` alone. The "Let it play" mode is a GSAP tween that scrolls the page.
- **Three.js** (`site/src/world/sky3d.js`, loaded after first paint): pigeons with hinged wings, kites folded along the spine with strings, and drifting paper flecks, in real depth with a perspective camera framed to match the SVG. Without WebGL, SVG pigeons and flecks stand in.
- **Cycle rickshaw** (`site/src/world/rickshaw.js`): Kin walks to it, rides it down the lane and steps off. His feet are placed on the pedals with a two-bone leg solve; wheels and cranks turn with distance travelled.
- **Glyph morph** (`site/scripts/glyphs.mjs`, `site/src/world/glyphmorph.js`): HarfBuzz shapes the three signboard words at build time and exports the real glyph contours. On the page, contours are resampled, wound the same way, paired left to right, start-aligned and interpolated; unpaired contours grow from or shrink to a point. This is the method planned for the engine.
- **Street** (`site/src/world/street.js`): a seeded generator emits four parallax layers (far skyline, mid buildings, near shopfronts, foreground poles and cables). Paper shadows are offset copies of each sheet, which is far cheaper than filters over a street this long. Pieces away from the camera are hidden.
- **Kin** is the approved rig from `design/kin/kin.js`, imported as is. Walking is procedural: the stride phase comes from distance travelled, so feet do not slide when easing changes speed.
- **Light**: sky gradient, a multiply layer for the colour of the light, and a screen layer of lamp glows that fades in at night. All are keyframed tracks on `t`.
- **Chapter mechanics**: shutters open as Kin reaches them, with keyframe diamonds on the road showing eased spacing; signboards swing on the closed-form spring from the engine docs, with live stiffness and damping; the brass piece morphs between four resampled outlines; pigeons fly on per-bird functions of `t`; the last chapter fills the sky with kites, letterboxes and fills a render bar.
- **Borrowed from the references**: one centred caption low in the frame and almost no chrome; a thin timeline along the bottom; a "Let it play" mode that scrolls for the visitor; drifting flecks of paper.
- **Fallbacks**: with reduced motion the page holds one key frame per chapter and cuts between them; the auto mode is hidden. Captions and code are real text in reading order, chapter markers are buttons, the playhead is a slider.
- Deployed by `.github/workflows/pages.yml` to GitHub Pages under `/kinetic-engine/`.

Difference from the first plan: the street itself is SVG paper layers, not a modelled 3D scene.

## Still to do

- Signboard words in Hindi and Urdu need the user's confirmation.
- Photo references were not gathered; the street is drawn from general knowledge of the area.
- Performance and accessibility have not been measured (Lighthouse), and the page has not been tried on a real phone. The Three.js chunk is about 530 KB before compression.
- Reduced-motion mode is written but has not been exercised in a browser.
- Portfolio phase: named shopfronts that open onto projects.

## Done when

- Scrolling forwards and backwards through all seven chapters shows no pops, teleports or state that fails to rewind.
- Each chapter's caption, code snippet and street action clearly show the same engine idea.
- Frame rate and transfer budgets are met on a laptop and a phone, checked in a browser.
- The fallback path works with WebGL disabled and with reduced motion.
- Script text on signboards has been confirmed by the user.
- The page is live on GitHub Pages and linked from the README.
