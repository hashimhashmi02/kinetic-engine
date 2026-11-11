# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Astro static site with GSAP ScrollTrigger; Three.js for the scroll journey. Chosen with the user during planning, revised after they shared 3D and illustrated story-site references. Whether the world renders as full 3D or as layered 2.5D illustration is still open (see Capabilities and Constraints). Deploys to GitHub Pages.

## Users

Primary: clients and recruiters looking at Hashim Hashmi's work. They arrive from a link, give it a minute or less, and leave with an impression of craft. The page has to impress first and explain second.

Secondary: developers and motion people who find the Kinetic engine on GitHub and want to know what it does.

## Product Purpose

Kinetic is a native, GPU-accelerated motion engine written in Rust: animations are written as code, previewed live, scrubbed on a timeline and exported to MP4. The landing page presents the engine and, later, becomes Hashim's personal portfolio. Success is a visitor who remembers the page and the character an hour later and believes the person who made it can build hard, beautiful things.

## Positioning

Every frame is a pure function of time. Scene code compiles to a timeline up front, springs are solved in closed form, and shapes and text share one path pipeline, so anything can be scrubbed, rewound and morphed. The page demonstrates this by being scrubbable itself: scroll is the playhead.

## Operating Context

- The page is a scroll-driven journey through Old Delhi: Chandni Chowk, Nai Sarak, Chawri Bazar, the steps of Jama Masjid. The route and chapter list are in `docs/LANDING.md`.
- A character, Kin, leads the journey. Each stretch of street demonstrates one engine capability.
- Shopfronts along the route are reserved as slots for portfolio projects in a later phase.

## Capabilities and Constraints

- Engine status: early. Colour, interpolation and easing exist; renderer, timeline, morphing, text and export are planned (`docs/MILESTONES.md`). The page must not claim features as shipped before they are.
- Scroll must rewind everything exactly; no one-shot effects that cannot reverse.
- Fallback required for no WebGL and for reduced motion.
- Undecided: full 3D world versus layered 2.5D illustration. The user's two references are one of each.
- Undecided: final wording of Hindi and Urdu signboard text; needs the user's confirmation before launch.

## Brand Commitments

- Project name: Kinetic. Repository: `hashimhashmi02/kinetic-engine`.
- Portfolio owner: Hashim Hashmi, personal.
- Character: **Kin**, an Old Delhi kid of about 10 to 12 who acts as a local guide. Street-smart, cheeky, fast, warm. Knows every shortcut in the gali. Must be a properly designed character with a recognisable silhouette, not a generic mascot.
- Kin design approved by the user (11 Nov 2026): layered paper cut-out, no outlines, grained sheets with soft shadows; bright warm skin, slate-blue kurta, white and grey scarf, indigo shorts, Hawai chappals, red kite on the back as the only saturated colour, small dot eyes, side-swept fringe with one tuft. Source of truth: `design/kin/kin.js` and `design/kin/sheet.html`. An earlier bold comic-print version was rejected.
- References the user supplied as inspiration (not to be copied): a scroll-driven 3D fly-through studio site, and an illustrated paper-theatre story site with a strongly designed child character.
- Jama Masjid is shown from outside (exterior, steps, skyline) and treated with respect.

## Evidence on Hand

- The engine source in this repository and its tests.
- No customers, testimonials, benchmarks or portfolio projects yet. None may be invented.

## Product Principles

1. The page proves the engine's idea by behaving like it: time is scrubbable.
2. One place, one route, one character. Everything on screen belongs to the street.
3. Affection and accuracy for a real neighbourhood over exotic shorthand.
4. Honest about what is built and what is planned.

## Accessibility & Inclusion

Captions and code are real text in reading order. Reduced-motion and no-WebGL visitors get a complete still version. Script text is correctly shaped and reviewed by a reader of that script.
