//! Animated bot avatars for agents: eighteen simple 3D shapes with living
//! faces — round eyes that blink and glance — and three states an agent can be
//! in: idle, working and sleeping. Each one turns its head, looks around, hops
//! and flips, and every state change is a cross-animation.
//!
//! Ported from [`bot-avatars`](https://github.com/Jakubantalik/Libraries.dev/tree/main/packages/bot-avatars)
//! (MIT, © Jakub Antalik) onto Rune's vector [`canvas`](rune::canvas): the rig,
//! the shapes and the face are ported onto Rune's canvas, with a pixel layer
//! for the glossy plastic and plush fabric bodies.
//!
//! ```
//! use coderune_avatars::{BotAvatar, BotAvatarSim, BotAvatarState, BotAvatarType};
//!
//! let avatar = BotAvatar::new(BotAvatarType::Clover);
//! let mut sim = BotAvatarSim::new(0.4, BotAvatarState::Working);
//! sim.update(1.0 / 60.0);
//! let view = avatar.view(&sim, 48.0);
//! ```

mod color;
mod engine;
mod material;
mod presets;
mod render;
mod shapes;
mod svg;

use std::collections::HashMap;
use std::sync::LazyLock;

pub use color::Rgb;
pub use engine::{BotAvatarSim, JumpConfig, Pose, Weights};
pub use render::Look;
use rune::{canvas, el, El, Pixels, Shape};
use svg::Outline;

/// The eighteen body shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BotAvatarType {
    Clover,
    Flower,
    Triangle,
    Square,
    Blob,
    Ghost,
    Circle,
    Drop,
    Star,
    Droid,
    Mech,
    Alien,
    Hexagon,
    Cat,
    Cloud,
    Pill,
    Pebble,
    Puddle,
}

impl BotAvatarType {
    pub const ALL: [BotAvatarType; 18] = [
        BotAvatarType::Clover,
        BotAvatarType::Flower,
        BotAvatarType::Triangle,
        BotAvatarType::Square,
        BotAvatarType::Blob,
        BotAvatarType::Ghost,
        BotAvatarType::Circle,
        BotAvatarType::Drop,
        BotAvatarType::Star,
        BotAvatarType::Droid,
        BotAvatarType::Mech,
        BotAvatarType::Alien,
        BotAvatarType::Hexagon,
        BotAvatarType::Cat,
        BotAvatarType::Cloud,
        BotAvatarType::Pill,
        BotAvatarType::Pebble,
        BotAvatarType::Puddle,
    ];

    /// A display name, for labels.
    pub fn label(self) -> &'static str {
        self.preset().label
    }

    /// The type's shipped look.
    pub fn preset(self) -> BotAvatarPreset {
        presets::preset(self)
    }

    /// A stable 0–1 seed, so two avatars with the same id never blink in step.
    pub fn seed(self) -> f32 {
        (self as u32 as f32 + 0.5) / BotAvatarType::ALL.len() as f32
    }
}

/// What the face is made of: the eyes alone, or eyes and a small mouth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BotAvatarFace {
    /// The eyes alone.
    #[default]
    Eyes,
    /// Eyes and a mouth that changes with the state.
    Mouth,
}

/// What the bot is doing. Each state is a resting pose plus its own motion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BotAvatarState {
    /// Turns to look around, blinks, and jumps with a full turn now and then.
    #[default]
    Idle,
    /// Hops all the time — every third hop a spin — with a wide smile and the
    /// odd laugh.
    Working,
    /// Head down, lids shut, slow breaths and the odd nod.
    Sleeping,
}

impl BotAvatarState {
    pub const ALL: [BotAvatarState; 3] = [
        BotAvatarState::Idle,
        BotAvatarState::Working,
        BotAvatarState::Sleeping,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BotAvatarState::Idle => "idle",
            BotAvatarState::Working => "working",
            BotAvatarState::Sleeping => "sleeping",
        }
    }

    pub(crate) fn index(self) -> usize {
        match self {
            BotAvatarState::Idle => 0,
            BotAvatarState::Working => 1,
            BotAvatarState::Sleeping => 2,
        }
    }
}

/// How the landing squash of a jump plays out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SquashEase {
    /// All at once, then eases off.
    Sharp,
    /// A quick press that recovers without a wobble.
    #[default]
    Pulse,
    /// Eases in and out.
    Soft,
    /// Overshoots into a stretch and settles.
    Bouncy,
}

/// How the body is lit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BotAvatarShading {
    /// A lit rim with a clean edge round the front, vector-style.
    #[default]
    Crisp,
    /// A soft shadow and highlight across the whole form, no edge.
    Smooth,
    /// The depth alone, no lighting.
    Flat,
    /// Glossy toy plastic with pose-dependent highlights.
    Plastic,
    /// A softly lit body covered in fine plush pile.
    Fabric,
}

impl BotAvatarShading {
    fn material(self) -> Option<material::Material> {
        match self {
            Self::Crisp | Self::Smooth | Self::Flat => None,
            Self::Plastic => Some(material::Material::Plastic),
            Self::Fabric => Some(material::Material::Fabric),
        }
    }
}

/// A type's shipped look: its label, body colour and where the face sits.
#[derive(Clone, Copy, Debug)]
pub struct BotAvatarPreset {
    pub label: &'static str,
    /// The type's own body colour, `#rrggbb`.
    pub color: &'static str,
    pub face: BotAvatarFace,
    /// Where the face sits, in the 100x100 body box.
    pub face_x: f32,
    pub face_y: f32,
    /// Face scale: shapes with a small middle wear a smaller face.
    pub face_scale: f32,
    pub brightness: f32,
    pub saturation: f32,
}

/// The flattened outlines, parsed once from the generated path data.
struct Catalog {
    bodies: HashMap<&'static str, Outline>,
    parts: HashMap<&'static str, Vec<Outline>>,
}

fn catalog() -> &'static Catalog {
    static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
        let mut bodies = HashMap::new();
        let mut parts = HashMap::new();
        for kind in BotAvatarType::ALL {
            bodies.insert(kind.path(), svg::parse(kind.path()));
            if let Some(path) = kind.parts() {
                // Each `M` starts its own piece, so each rounds on its own.
                let pieces: Vec<Outline> = path
                    .split("M")
                    .filter(|piece| !piece.trim().is_empty())
                    .map(|piece| svg::parse(&format!("M{piece}")))
                    .collect();
                parts.insert(kind.path(), pieces);
            }
        }
        Catalog { bodies, parts }
    });
    &CATALOG
}

fn body_outline(kind: BotAvatarType) -> &'static Outline {
    &catalog().bodies[kind.path()]
}

fn body_parts(kind: BotAvatarType) -> &'static [Outline] {
    catalog()
        .parts
        .get(kind.path())
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

/// One avatar's look, as a builder over a type's preset.
#[derive(Clone, Copy, Debug)]
pub struct BotAvatar {
    kind: BotAvatarType,
    shading: BotAvatarShading,
    color: Option<Rgb>,
    ink: Option<Rgb>,
    face: Option<BotAvatarFace>,
    depth: f32,
    light: Option<f32>,
    contrast: Option<(f32, f32)>,
    rim: Option<f32>,
    whirl: f32,
}

impl BotAvatar {
    pub fn new(kind: BotAvatarType) -> BotAvatar {
        BotAvatar {
            kind,
            shading: BotAvatarShading::Crisp,
            color: None,
            ink: None,
            face: None,
            depth: 0.65,
            light: None,
            contrast: None,
            rim: None,
            whirl: 0.0,
        }
    }

    pub fn kind(&self) -> BotAvatarType {
        self.kind
    }

    pub fn shading(mut self, shading: BotAvatarShading) -> BotAvatar {
        self.shading = shading;
        self
    }

    /// A body colour, instead of the type's own.
    pub fn color(mut self, color: Rgb) -> BotAvatar {
        self.color = Some(color);
        self
    }

    /// Face ink. Defaults to pure white on any background.
    pub fn ink(mut self, ink: Rgb) -> BotAvatar {
        self.ink = Some(ink);
        self
    }

    pub fn face(mut self, face: BotAvatarFace) -> BotAvatar {
        self.face = Some(face);
        self
    }

    /// Thickness of the body, what shows when it turns or flips.
    pub fn depth(mut self, depth: f32) -> BotAvatar {
        self.depth = depth.clamp(0.2, 2.0);
        self
    }

    /// Where the light comes from, in degrees clockwise from the top.
    pub fn light(mut self, degrees: f32) -> BotAvatar {
        self.light = Some(degrees);
        self
    }

    /// Strength of the shadow side and the lit side, 0–2.
    pub fn contrast(mut self, shadow: f32, highlight: f32) -> BotAvatar {
        self.contrast = Some((shadow.clamp(0.0, 2.0), highlight.clamp(0.0, 2.0)));
        self
    }

    /// Strength of the rim, 0–2.
    pub fn rim(mut self, rim: f32) -> BotAvatar {
        self.rim = Some(rim.clamp(0.0, 2.0));
        self
    }

    /// The cartoon whirl round a spin: `0` off (the default), `1` the stock
    /// strength.
    pub fn whirl(mut self, strength: f32) -> BotAvatar {
        self.whirl = strength.clamp(0.0, 2.0);
        self
    }

    /// The resolved look the renderer uses.
    pub fn look(&self) -> Look {
        let mut look = Look::new(self.kind, self.shading);
        if let Some(color) = self.color {
            look.body = color;
        }
        if let Some(ink) = self.ink {
            look.ink = ink;
        }
        if let Some(face) = self.face {
            look.face = face;
        }
        if let Some((shadow, highlight)) = self.contrast {
            look.shadow = shadow;
            look.highlight = highlight;
        }
        look.depth = self.depth;
        look.light = self.light.unwrap_or(look.light);
        look.rim = self.rim.unwrap_or(look.rim);
        look.whirl_strength = self.whirl;
        look
    }

    /// The vector version of a frame, in the canvas' own points. Pixel
    /// materials use smooth shading here. Use [Self::view] to draw them.
    pub fn shape(&self, pose: &Pose, size: f32) -> Shape {
        let outline = body_outline(self.kind);
        if outline.is_empty() {
            return Shape::new();
        }
        render::frame(outline, body_parts(self.kind), &self.look(), pose, size)
    }

    /// A live avatar as a view `size` points square. The caller advances the
    /// sim (see [`BotAvatarSim::update`]).
    pub fn view(&self, sim: &BotAvatarSim, size: f32) -> El {
        self.build(sim.pose(), size, None)
    }

    /// The still pose of `state`, for reduced motion and first paint.
    pub fn still(&self, state: BotAvatarState, size: f32) -> El {
        self.build(&Pose::rest(state), size, None)
    }

    fn build(&self, pose: &Pose, size: f32, surface: Option<&Pixels>) -> El {
        // The canvas is larger than the layout box so a hop or a flip is not
        // clipped; the box keeps the avatar's own size in the layout.
        let full = size * render::OVERSCAN;
        let inset = -(full - size) / 2.0;
        let portrait = el().size(size, size).inert().label(self.kind.label());
        let layer = |el: El| el.size(full, full).absolute(inset, inset);
        let Some(material) = self
            .shading
            .material()
            .filter(|_| full.is_finite() && full > 0.0)
        else {
            return portrait.child(layer(canvas(self.shape(pose, size))));
        };

        let outline = body_outline(self.kind);
        let look = self.look();
        let frame = render::layers(
            outline,
            body_parts(self.kind),
            &look,
            pose,
            size,
            Some((self.kind, material)),
        );
        let mut portrait = portrait.child(layer(canvas(frame.back)));
        if let Some(image) = frame.image {
            let pixels = surface.cloned().unwrap_or_default();
            pixels.set(image.size, image.size, image.rgba);
            portrait = portrait.child(layer(el().pixels(&pixels)));
        }
        portrait.child(layer(canvas(frame.front)))
    }
}

/// The sims behind a set of avatars, keyed by an agent's id, and one clock for
/// all of them.
///
/// The app owns one stage. Each frame (or on admission) it calls
/// [`set`](Self::set) for the avatars on screen and [`tick`](Self::tick) with
/// the frame time; the pose of each is then drawn with
/// [`BotAvatar::view`].
#[derive(Default)]
pub struct BotAvatarStage {
    sims: HashMap<String, Entry>,
    last: Option<f64>,
}

struct Entry {
    sim: BotAvatarSim,
    state: BotAvatarState,
    kind: BotAvatarType,
    surface: Pixels,
}

impl BotAvatarStage {
    pub fn new() -> BotAvatarStage {
        BotAvatarStage::default()
    }

    /// Registers `kind` for `id` in `state`, creating its sim on first sight
    /// and easing it when the state changes.
    pub fn set(&mut self, id: &str, kind: BotAvatarType, state: BotAvatarState) {
        if let Some(entry) = self.sims.get_mut(id) {
            if entry.kind != kind {
                entry.kind = kind;
                entry.sim = BotAvatarSim::new(kind.seed(), state);
            } else if entry.state != state {
                entry.state = state;
                entry.sim.set_state(state, self.last.is_none());
            }
            return;
        }
        let sim = BotAvatarSim::new(kind.seed(), state);
        self.sims.insert(
            id.to_owned(),
            Entry {
                sim,
                state,
                kind,
                surface: Pixels::new(),
            },
        );
    }

    /// Advances every sim to `now` (seconds, monotonic). Returns whether
    /// anything moved, so a caller can ask the runtime for another frame.
    pub fn tick(&mut self, now: f64, speed: f32) -> bool {
        let dt = match self.last {
            Some(last) => ((now - last) as f32 * speed).clamp(0.0, 0.1),
            None => {
                self.last = Some(now);
                return false;
            }
        };
        self.last = Some(now);
        if dt <= 0.0 {
            return false;
        }
        for entry in self.sims.values_mut() {
            entry.sim.update(dt);
        }
        !self.sims.is_empty()
    }

    pub fn pose(&self, id: &str) -> Option<&Pose> {
        self.sims.get(id).map(|entry| entry.sim.pose())
    }

    pub fn sim(&self, id: &str) -> Option<&BotAvatarSim> {
        self.sims.get(id).map(|entry| &entry.sim)
    }

    pub fn sim_mut(&mut self, id: &str) -> Option<&mut BotAvatarSim> {
        self.sims.get_mut(id).map(|entry| &mut entry.sim)
    }

    /// Draws the portrait for one registered ID with its reusable pixel
    /// surface. Give each portrait its own ID when drawing multiple views.
    pub fn view(&self, id: &str, avatar: BotAvatar, fallback: BotAvatarState, size: f32) -> El {
        match self.sims.get(id) {
            Some(entry) => avatar.build(entry.sim.pose(), size, Some(&entry.surface)),
            None => avatar.still(fallback, size),
        }
    }

    /// Drops the sims not in `keep`, so a list that shrinks stops animating
    /// what left the screen.
    pub fn retain(&mut self, keep: impl Fn(&str) -> bool) {
        self.sims.retain(|id, _| keep(id));
    }

    /// Forgets every sim, and the clock with them.
    pub fn clear(&mut self) {
        self.sims.clear();
        self.last = None;
    }

    pub fn is_empty(&self) -> bool {
        self.sims.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_face_ink_is_white_on_light_and_dark_bodies() {
        for color in [0x41c4ff, 0x111111] {
            let avatar = BotAvatar::new(BotAvatarType::Clover).color(Rgb::from_hex(color));
            assert_eq!(avatar.look().ink, Rgb::from_hex(0xffffff));
        }
    }

    #[test]
    fn every_type_has_a_preset_and_an_outline() {
        for kind in BotAvatarType::ALL {
            let preset = kind.preset();
            assert!(!preset.label.is_empty());
            assert!(Rgb::from_css(preset.color).is_some(), "{kind:?} colour");
            let outline = body_outline(kind);
            assert!(!outline.is_empty(), "{kind:?} has no outline");
            let points: usize = outline.subpaths.iter().map(|s| s.points.len()).sum();
            assert!(points >= 8, "{kind:?} flattened to {points} points");
            assert!(
                points < 400,
                "{kind:?} flattened to too many points: {points}"
            );
        }
    }

    #[test]
    fn antennae_are_separate_pieces() {
        assert_eq!(body_parts(BotAvatarType::Droid).len(), 2);
        assert_eq!(body_parts(BotAvatarType::Mech).len(), 4);
        assert!(body_parts(BotAvatarType::Circle).is_empty());
    }

    #[test]
    fn a_stage_eases_between_states() {
        let mut stage = BotAvatarStage::new();
        stage.set("claude", BotAvatarType::Clover, BotAvatarState::Working);
        assert!(!stage.tick(10.0, 1.0), "the first tick only sets the clock");
        assert!(stage.tick(10.016, 1.0));
        stage.set("claude", BotAvatarType::Clover, BotAvatarState::Sleeping);
        for i in 0..120 {
            stage.tick(10.016 + i as f64 * 0.016, 1.0);
        }
        let pose = stage.pose("claude").expect("pose");
        assert!(pose.w[2] > 0.9, "sleeping weight {:?}", pose.w);
        stage.retain(|id| id == "other");
        assert!(stage.is_empty());
    }

    #[test]
    fn a_frozen_clock_does_not_move() {
        let mut stage = BotAvatarStage::new();
        stage.set("a", BotAvatarType::Blob, BotAvatarState::Idle);
        stage.tick(1.0, 1.0);
        assert!(!stage.tick(1.0, 1.0));
        assert!(stage.tick(1.5, 1.0));
    }
}
