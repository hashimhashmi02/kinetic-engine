//! Text that moves. The title's letters pop up from their baseline one after another, a
//! greeting turns from Hello into World and then into नमस्ते glyph by glyph, and the letters
//! wave before everything fades out.
//!
//! Text is shaped with the bundled Poppins while the scene builds, so the timeline holds
//! only paths and plays the same everywhere.

use kinetic_core::color::palette;
use kinetic_core::{spring, Action, Ease, LinearRgba, Scene, Spring, Text, Timeline};

/// Writes the scene into `s`.
pub fn scene(s: &mut Scene) {
    let marigold = LinearRgba::from_hex(0xf2a03d);
    let bone = LinearRgba::from_hex(0xe2d6bf);

    let title = s.add_text(Text::new("Kinetic").bold().size(150.0).at((0.0, 150.0)));
    let subtitle = s.add_text(
        Text::new("every frame is a function of time")
            .size(32.0)
            .letter_spacing(1.0)
            .fill(bone)
            .opacity(0.0)
            .at((0.0, 40.0)),
    );
    let greeting = s.add_text(
        Text::new("Hello").bold().size(120.0).fill(marigold).opacity(0.0).at((0.0, -170.0)),
    );

    // The title's letters start flat on their baseline and pop up one by one.
    let letters = s.glyphs(title);
    s.parallel(letters.iter().map(|g| g.scale_to(0.0)), 0.0);
    s.stagger(letters.iter().map(|g| g.scale_to(1.0)), Spring::WOBBLY, 0.07);
    s.play(subtitle.fade_to(1.0), (0.6, Ease::OutCubic));

    // Hello turns into World, then into Namaste in Devanagari.
    s.play(greeting.fade_to(1.0), (0.4, Ease::OutCubic));
    s.wait(0.3);
    s.play(greeting.text_to("World"), (0.9, Ease::InOutCubic));
    s.wait(0.4);
    s.parallel(
        [greeting.text_to("नमस्ते"), greeting.fill_to(palette::red())],
        spring().stiffness(140.0).damping(12.0),
    );
    s.wait(0.3);

    // A wave runs along the title: each letter tips and comes back.
    s.stagger(
        letters.iter().map(|g| Action::all([g.rotate_to(0.18), g.scale_to(1.12)])),
        (0.25, Ease::OutCubic),
        0.06,
    );
    s.stagger(
        letters.iter().map(|g| Action::all([g.rotate_to(0.0), g.scale_to(1.0)])),
        (0.35, Ease::InOutCubic),
        0.06,
    );

    // Everything leaves.
    s.parallel([title.fade_to(0.0), subtitle.fade_to(0.0), greeting.fade_to(0.0)], 0.6);
    s.wait(0.3);
}

kinetic_core::kinetic_scene!(scene);

/// The compiled scene.
pub fn build() -> Timeline {
    let mut s = Scene::new();
    scene(&mut s);
    s.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetic_core::DVec2;

    #[test]
    fn builds_the_same_timeline_every_time() {
        assert_eq!(build(), build());
    }

    #[test]
    fn title_letters_start_flat_and_everything_ends_faded() {
        let tl = build();
        let start = tl.evaluate(0.0);
        assert!(start.nodes[..7].iter().all(|n| n.props.scale == DVec2::ZERO));
        let end = tl.evaluate(tl.duration);
        // Glyphs left over from a longer word are already shrunk to nothing.
        assert!(end.nodes.iter().all(|n| n.props.opacity == 0.0 || n.props.scale == DVec2::ZERO));
    }

    #[test]
    fn the_greeting_ends_in_devanagari() {
        let greeting = Text::new("नमस्ते").bold().size(120.0).glyphs().len();
        let mut s = Scene::new();
        scene(&mut s);
        // The last text added is the greeting; its glyphs now spell the Devanagari word.
        let tl = s.build();
        let shown = tl.evaluate(tl.duration - 1.0);
        let visible = shown.nodes.iter().filter(|n| n.props.scale != DVec2::ZERO).count();
        let others = 7 + Text::new("every frame is a function of time").glyphs().len();
        assert_eq!(visible, others + greeting);
    }

    #[test]
    fn exported_entry_points_return_the_same_timeline() {
        use kinetic_core::abi::{SceneBytes, ABI_VERSION, STATUS_OK};
        assert_eq!(kinetic_scene_abi_version(), ABI_VERSION);
        let mut out = SceneBytes::EMPTY;
        // SAFETY: out is a valid SceneBytes; the bytes are read, then handed back once.
        let timeline = unsafe {
            assert_eq!(kinetic_scene_build(&mut out), STATUS_OK);
            let timeline = kinetic_core::wire::decode(out.as_slice()).expect("decode");
            kinetic_scene_free(out);
            timeline
        };
        assert_eq!(timeline, build());
    }
}
