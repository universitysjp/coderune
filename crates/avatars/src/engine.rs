//! The rig: a pose is a handful of numbers — head yaw, pitch and roll, a
//! position, squash, how open the eyes are, where they look — plus blend
//! weights for the three states.
//!
//! A [`BotAvatarSim`] advances a pose through time: each state sets targets and
//! wanders around them, runs its own events (a flip, a hop, a nod, a blink),
//! and a state change eases from one set of targets to the next, so it starts
//! and ends softly. Ported from the upstream `bot-avatars` `engine.ts`.

use crate::{BotAvatarState, SquashEase};

/// A hop's period, in seconds.
const HOP_T: f32 = 0.68;
/// The height of the spinning hop, in body units.
const HOP_SPIN_H: f32 = 26.0;
/// A flip's crouch, measured in hop periods.
const FLIP_PRE: f32 = 0.2;

const TO_DEG: f32 = std::f32::consts::PI / 180.0;
const TAU: f32 = std::f32::consts::TAU;

/// The numbers of a jump: an idle flip's, and a click's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JumpConfig {
    /// How high, in body units (the body is 100 tall).
    pub height: f32,
    /// Seconds in the air.
    pub time: f32,
    /// How much the body stretches in the air.
    pub stretch: f32,
    /// How much it squashes on the ground, before take-off and on landing.
    pub squash: f32,
    /// Seconds the landing squash takes, contact to recovered.
    pub squash_time: f32,
    pub squash_ease: SquashEase,
    /// Seconds the body holds its deepest squash on the ground.
    pub ground_time: f32,
    pub ground_ease: SquashEase,
    /// Seconds the body takes to rise from its deepest squash to its shape.
    pub rise_time: f32,
    pub rise_ease: SquashEase,
    /// Seconds a click's jump takes for its landing squash.
    pub click_squash_time: f32,
    /// Whole turns in the air.
    pub spin: f32,
    /// Degrees of lean into the jump.
    pub lean: f32,
    /// Seconds between idle jumps, give or take 40%; 0 for none.
    pub every: f32,
    /// When the landing squash begins: seconds before (negative) or after
    /// touch-down.
    pub land: f32,
}

impl JumpConfig {
    pub const DEFAULT: JumpConfig = JumpConfig {
        height: HOP_SPIN_H,
        time: HOP_T,
        stretch: 1.0,
        squash: 1.15,
        squash_time: 0.37,
        squash_ease: SquashEase::Pulse,
        ground_time: 0.11,
        ground_ease: SquashEase::Pulse,
        rise_time: 0.33,
        rise_ease: SquashEase::Pulse,
        click_squash_time: 0.24,
        spin: 1.0,
        lean: 6.0,
        every: 8.0,
        land: 0.0,
    };
}

impl Default for JumpConfig {
    fn default() -> JumpConfig {
        JumpConfig::DEFAULT
    }
}

/// Where in its time each easing reaches the deepest squash.
fn squash_peak(ease: SquashEase) -> f32 {
    match ease {
        SquashEase::Sharp => 0.0,
        SquashEase::Pulse => 2.0 / 7.0,
        SquashEase::Soft => 0.5,
        SquashEase::Bouncy => 0.144,
    }
}

fn rise_pulse(v: f32, ease: SquashEase) -> f32 {
    if v <= 0.0 {
        return 1.0;
    }
    if v >= 1.0 {
        return 0.0;
    }
    match ease {
        SquashEase::Sharp => (1.0 - v) * (1.0 - v),
        SquashEase::Soft => 0.5 + 0.5 * (std::f32::consts::PI * v).cos(),
        SquashEase::Bouncy => (-3.2 * v).exp() * (5.4 * v).cos() - v * v * v * 0.026,
        SquashEase::Pulse => {
            let k = 4.2 * v;
            (1.0 + k) * (-k).exp() - v * v * v * 0.078
        }
    }
}

fn squash_pulse(u: f32, ease: SquashEase) -> f32 {
    if u <= 0.0 || u >= 1.0 {
        return 0.0;
    }
    let x = match ease {
        SquashEase::Sharp => (1.0 - u) * (1.0 - u),
        SquashEase::Soft => (std::f32::consts::PI * u).sin().powi(2),
        SquashEase::Bouncy => (-3.15 * u).exp() * (8.43 * u).sin() / 0.596,
        SquashEase::Pulse => {
            let k = 7.0 * u;
            k * k * (2.0 - k).exp() / 4.0
        }
    };
    let tail = if u > 0.85 {
        1.0 - (u - 0.85) / 0.15
    } else {
        1.0
    };
    x * tail * tail * (3.0 - 2.0 * tail)
}

/// The weight settling through the hold, as a gain on the deepest squash.
fn ground_shape(v: f32, ease: SquashEase) -> f32 {
    1.0 + 0.25 * squash_pulse(v, ease)
}

/// The hop's shaping: squash on the ground, stretch at the top.
fn hop_squash(a: f32) -> f32 {
    let near = a.abs().min((a - 1.0).abs());
    (-(near / 0.11).powi(2)).exp()
}

/// A blend of the three states that sums to 1.
pub type Weights = [f32; 3];

/// A rig pose. Angles are radians, `yaw > 0` turns the face to the viewer's
/// right and `pitch > 0` looks up; positions are in body units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    /// Body-box units (the 100x100 design space).
    pub x: f32,
    pub y: f32,
    pub sx: f32,
    pub sy: f32,
    /// 0 shut … 1 open, before blinks.
    pub eye_open: f32,
    /// How far each lid is down right now, 0 … 1.
    pub blink_l: f32,
    pub blink_r: f32,
    pub look_x: f32,
    pub look_y: f32,
    /// The breathing cycle, −1 … 1.
    pub breath: f32,
    /// Working only: how far the eyes have closed into a laugh, 0 … 1.
    pub laugh: f32,
    /// The cartoon whirl round a spinning body: strength, and where its head
    /// is, in radians round the ring.
    pub whirl: f32,
    pub whirl_angle: f32,
    /// Blend weights: idle, working, sleeping.
    pub w: Weights,
}

impl Pose {
    pub const REST: Pose = Pose {
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
        x: 0.0,
        y: 0.0,
        sx: 1.0,
        sy: 1.0,
        eye_open: 1.0,
        blink_l: 0.0,
        blink_r: 0.0,
        look_x: 0.0,
        look_y: 0.0,
        breath: 0.0,
        laugh: 0.0,
        whirl: 0.0,
        whirl_angle: 0.0,
        w: [1.0, 0.0, 0.0],
    };

    /// The still pose of a state, for reduced motion and the first paint.
    pub fn rest(state: BotAvatarState) -> Pose {
        let rest = Rest::of(state);
        Pose {
            pitch: rest.pitch,
            roll: rest.roll,
            y: rest.y,
            look_x: rest.look_x,
            look_y: rest.look_y,
            w: weights_of(state),
            ..Pose::REST
        }
    }
}

fn weights_of(state: BotAvatarState) -> Weights {
    let mut w = [0.0; 3];
    w[state.index()] = 1.0;
    w
}

/// How long a switch takes. Settling down takes longer than getting to work:
/// coming back to idle the body has to let go of the hops.
fn switch_to(state: BotAvatarState) -> f32 {
    match state {
        BotAvatarState::Idle => 1.2,
        BotAvatarState::Working => 0.7,
        BotAvatarState::Sleeping => 1.4,
    }
}

const SWITCH_FROM_SLEEP: f32 = 1.0;

/// Deterministic per-instance randomness (mulberry32).
#[derive(Clone, Copy)]
struct Rng {
    a: u32,
}

impl Rng {
    fn new(seed: u64) -> Rng {
        // The upstream seed is a float times a 32-bit constant, then truncated.
        let a = ((seed as f64 * 2_654_435_761.0) as u64 % (1u64 << 32)) as u32;
        Rng {
            a: if a == 0 { 1 } else { a },
        }
    }

    fn next(&mut self) -> f32 {
        self.a = self.a.wrapping_add(0x6d2b_79f5);
        let mut t = self.a;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        ((t ^ (t >> 14)) as f32) / 4_294_967_296.0
    }
}

fn approach(cur: f32, target: f32, rate: f32, dt: f32) -> f32 {
    cur + (target - cur) * (1.0 - (-rate * dt).exp())
}

fn ease_in_out(p: f32) -> f32 {
    if p < 0.5 {
        4.0 * p * p * p
    } else {
        1.0 - (-2.0 * p + 2.0).powi(3) / 2.0
    }
}

fn ease_sine(p: f32) -> f32 {
    0.5 - 0.5 * (std::f32::consts::PI * p).cos()
}

fn smoothstep(a: f32, b: f32, v: f32) -> f32 {
    let x = ((v - a) / (b - a)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// A value that drifts: picks a new target inside its range every hold, and
/// eases toward it.
struct Wander {
    value: f32,
    vel: f32,
    target: f32,
    next: f32,
    amp: f32,
    hold_min: f32,
    hold_max: f32,
    rate: f32,
    spring: bool,
}

impl Wander {
    fn new(amp: f32, hold_min: f32, hold_max: f32, rate: f32, spring: bool) -> Wander {
        Wander {
            value: 0.0,
            vel: 0.0,
            target: 0.0,
            next: 0.0,
            amp,
            hold_min,
            hold_max,
            rate,
            spring,
        }
    }

    fn update(&mut self, t: f32, dt: f32, rng: &mut Rng) {
        if t >= self.next {
            self.target = (rng.next() * 2.0 - 1.0) * self.amp;
            self.next = t + self.hold_min + rng.next() * (self.hold_max - self.hold_min);
        }
        if self.spring {
            let w = self.rate * 1.6;
            let z = 0.9;
            self.vel += (w * w * (self.target - self.value) - 2.0 * z * w * self.vel) * dt;
            self.value += self.vel * dt;
        } else {
            self.value = approach(self.value, self.target, self.rate, dt);
        }
    }

    /// Aims at a value and stays there: the rig picks the next one.
    fn aim(&mut self, v: f32) {
        self.target = v;
        self.next = f32::INFINITY;
    }

    fn set(&mut self, amp: f32, hold_min: f32, hold_max: f32, rate: f32) {
        self.amp = amp;
        self.hold_min = hold_min;
        self.hold_max = hold_max;
        self.rate = rate;
        self.next = 0.0;
    }
}

/// A one-shot event: progress 0 → 1 over its duration, then idle at −1.
struct Event {
    p: f32,
    duration: f32,
}

impl Event {
    fn new(duration: f32) -> Event {
        Event { p: -1.0, duration }
    }

    fn fire(&mut self) {
        self.p = 0.0;
    }

    fn active(&self) -> bool {
        self.p >= 0.0
    }

    fn update(&mut self, dt: f32) {
        if self.p < 0.0 {
            return;
        }
        self.p += dt / self.duration;
        if self.p >= 1.0 {
            self.p = -1.0;
        }
    }
}

fn flip_pre(j: &JumpConfig, poked: bool) -> f32 {
    if poked {
        j.click_squash_time
    } else {
        FLIP_PRE * j.time
    }
}

fn flip_duration(j: &JumpConfig, poked: bool) -> f32 {
    flip_pre(j, poked)
        + j.time
        + squash_peak(j.squash_ease)
            * if poked {
                j.click_squash_time
            } else {
                j.squash_time
            }
        + j.ground_time.max(0.0)
        + j.rise_time
        + j.land.max(0.0)
        + 0.05
}

/// The idle gaze: how far the head swings, and how long it stays.
const GAZE_YAW: f32 = 35.0 * TO_DEG;
const GAZE_PITCH: f32 = 14.0 * TO_DEG;
const GAZE_ROLL: f32 = 3.2 * TO_DEG;
const GAZE_HOLD_MIN: f32 = 2.6;
const GAZE_HOLD_MAX: f32 = 4.4;

/// Resting targets per state. Everything the wanderers and events add sits on
/// top of these.
struct Rest {
    pitch: f32,
    roll: f32,
    y: f32,
    look_x: f32,
    look_y: f32,
}

impl Rest {
    fn of(state: BotAvatarState) -> Rest {
        match state {
            BotAvatarState::Idle => Rest {
                pitch: 0.0,
                roll: 0.0,
                y: 0.0,
                look_x: 0.0,
                look_y: 0.0,
            },
            BotAvatarState::Working => Rest {
                pitch: 5.0 * TO_DEG,
                roll: 0.0,
                y: 0.0,
                look_x: 0.0,
                look_y: 0.0,
            },
            BotAvatarState::Sleeping => Rest {
                pitch: -16.0 * TO_DEG,
                roll: 6.0 * TO_DEG,
                y: 3.0,
                look_x: 0.0,
                look_y: 1.0,
            },
        }
    }
}

/// One avatar's animation.
pub struct BotAvatarSim {
    pose: Pose,
    state: BotAvatarState,
    rng: Rng,
    t: f32,
    w_from: Weights,
    tr: f32,
    tr_duration: f32,
    yaw_w: Wander,
    pitch_w: Wander,
    roll_w: Wander,
    look_x_w: Wander,
    look_y_w: Wander,
    blink: Event,
    blink_at: f32,
    blink_again: bool,
    dart: Event,
    dart_at: f32,
    dart_x: f32,
    dart_y: f32,
    flip: Event,
    flip_poked: bool,
    jump: JumpConfig,
    flip_at: f32,
    flip_side: f32,
    nod: Event,
    nod_at: f32,
    hop_phase: f32,
    hop_count: u32,
    hop_gain: f32,
    laugh: Event,
    laugh_at: f32,
    prev_yaw: f32,
    jelly: f32,
    jelly_v: f32,
    gaze_lead: f32,
    gaze_dir: [f32; 2],
    gaze_at: f32,
    turn: f32,
    breath_phase: f32,
    ptr_x: f32,
    ptr_y: f32,
    ptr_s: f32,
    ptr_target_x: f32,
    ptr_target_y: f32,
    ptr_target_s: f32,
    base_yaw: f32,
}

impl BotAvatarSim {
    /// A sim for `seed` (0–1, so two avatars side by side never blink in step)
    /// starting in `state`.
    pub fn new(seed: f32, state: BotAvatarState) -> BotAvatarSim {
        let seed = ((seed.clamp(0.0, 1.0) * 1e6) as u64) + 1;
        let mut rng = Rng::new(seed);
        let mut sim = BotAvatarSim {
            pose: Pose::REST,
            state,
            t: 0.0,
            w_from: [1.0, 0.0, 0.0],
            tr: 1.0,
            tr_duration: 1.2,
            yaw_w: Wander::new(36.0 * TO_DEG, 1.1, 2.6, 3.0, true),
            pitch_w: Wander::new(10.0 * TO_DEG, 1.1, 2.6, 2.6, true),
            roll_w: Wander::new(5.0 * TO_DEG, 1.6, 3.2, 2.0, true),
            look_x_w: Wander::new(3.6, 0.5, 2.0, 14.0, false),
            look_y_w: Wander::new(2.4, 0.5, 2.0, 14.0, false),
            blink: Event::new(0.17),
            blink_at: 0.0,
            blink_again: false,
            dart: Event::new(0.12),
            dart_at: 0.0,
            dart_x: 0.0,
            dart_y: 0.0,
            flip: Event::new(0.0),
            flip_poked: false,
            jump: JumpConfig::DEFAULT,
            flip_at: 0.0,
            flip_side: 1.0,
            nod: Event::new(1.7),
            nod_at: 0.0,
            hop_phase: 0.0,
            hop_count: 0,
            hop_gain: 0.0,
            laugh: Event::new(0.8),
            laugh_at: 0.0,
            prev_yaw: 0.0,
            jelly: 0.0,
            jelly_v: 0.0,
            gaze_lead: 0.0,
            gaze_dir: [0.0, 0.0],
            gaze_at: 0.0,
            turn: 1.0,
            breath_phase: 0.0,
            ptr_x: 0.0,
            ptr_y: 0.0,
            ptr_s: 0.0,
            ptr_target_x: 0.0,
            ptr_target_y: 0.0,
            ptr_target_s: 0.0,
            base_yaw: 0.0,
            rng: Rng::new(1),
        };
        // Every instance starts somewhere else in its loops, and its event
        // timers count from there, so nothing fires on the first tick.
        sim.t = rng.next() * 10.0;
        sim.hop_phase = rng.next();
        sim.breath_phase = rng.next();
        sim.blink_at = sim.t + 1.0 + rng.next() * 3.0;
        sim.flip_at = sim.flip_at_with(&mut rng, 1.0);
        sim.nod_at = sim.t + 3.0 + rng.next() * 4.0;
        sim.dart_at = sim.t + 1.0 + rng.next() * 2.0;
        sim.laugh_at = sim.t + 0.6 + rng.next() * 1.5;
        sim.rng = rng;
        sim.flip.duration = flip_duration(&sim.jump, false);
        sim.set_state(state, true);
        sim
    }

    pub fn state(&self) -> BotAvatarState {
        self.state
    }

    pub fn pose(&self) -> &Pose {
        &self.pose
    }

    pub fn jump(&self) -> JumpConfig {
        self.jump
    }

    /// Eases to a new state, or snaps for the first paint.
    pub fn set_state(&mut self, next: BotAvatarState, immediate: bool) {
        if next == self.state && !immediate {
            return;
        }
        let from = self.state;
        self.state = next;
        if immediate {
            self.pose.w = weights_of(next);
            self.tr = 1.0;
        } else {
            self.w_from = self.pose.w;
            self.tr = 0.0;
            self.tr_duration = if from == BotAvatarState::Sleeping {
                SWITCH_FROM_SLEEP
            } else {
                switch_to(next)
            };
        }
        match next {
            BotAvatarState::Idle => {
                self.yaw_w.set(GAZE_YAW, 2.6, 5.4, 2.0);
                self.pitch_w.set(GAZE_PITCH, 2.8, 5.8, 1.8);
                self.roll_w.set(GAZE_ROLL, 3.4, 6.6, 1.5);
                self.gaze_at = 0.0;
                self.gaze_dir = [0.0, 0.0];
                self.look_x_w.set(3.6, 0.6, 2.2, 13.0);
                self.look_y_w.set(2.4, 0.6, 2.2, 13.0);
                self.flip_at = self.next_flip(0.6);
            }
            BotAvatarState::Working => {
                self.yaw_w.set(16.0 * TO_DEG, 0.9, 1.8, 4.0);
                self.pitch_w.set(3.0 * TO_DEG, 1.2, 2.4, 3.0);
                self.roll_w.set(0.0, 1.0, 2.0, 3.0);
                self.look_x_w.set(2.0, 0.5, 1.2, 12.0);
                self.look_y_w.set(1.0, 0.5, 1.2, 12.0);
                self.hop_phase = 0.0;
                self.hop_count = 0;
                self.laugh_at = self.t + 0.5 + self.rng.next() * 1.2;
            }
            BotAvatarState::Sleeping => {
                self.yaw_w.set(7.0 * TO_DEG, 3.0, 6.0, 0.7);
                self.pitch_w.set(3.0 * TO_DEG, 3.0, 6.0, 0.7);
                self.roll_w.set(2.0 * TO_DEG, 3.0, 6.0, 0.6);
                self.look_x_w.set(0.0, 2.0, 4.0, 2.0);
                self.look_y_w.set(0.0, 2.0, 4.0, 2.0);
                self.nod_at = self.t + 2.5 + self.rng.next() * 4.0;
            }
        }
    }

    /// Where the pointer is, relative to the head (−1 … 1 across a head
    /// width), and how strongly to follow it (0 lets go).
    pub fn set_pointer(&mut self, x: f32, y: f32, strength: f32) {
        self.ptr_target_x = x.clamp(-1.2, 1.2);
        self.ptr_target_y = y.clamp(-1.2, 1.2);
        self.ptr_target_s = strength.clamp(0.0, 1.0);
    }

    /// A hop and a full turn, right now, whatever the state.
    pub fn poke(&mut self) {
        if self.flip.active() && self.flip.p < 0.6 {
            return;
        }
        self.flip_poked = true;
        self.flip.duration = flip_duration(&self.jump, true);
        self.flip_side = if self.rng.next() < 0.5 { -1.0 } else { 1.0 };
        self.flip.fire();
        self.flip_at = self.next_flip(1.1);
    }

    /// How far the head turns to the side while idle: 1 as the gaze has it, 0
    /// keeps it facing forward.
    pub fn set_turn(&mut self, k: f32) {
        let next = k.max(0.0);
        if next == self.turn {
            return;
        }
        self.turn = next;
        if self.state == BotAvatarState::Idle {
            self.gaze_at = 0.0;
        }
    }

    /// The jump's numbers; any subset, applied over the current ones.
    pub fn set_jump(&mut self, j: JumpConfig) {
        let every = self.jump.every;
        self.jump = j;
        if self.jump.every != every {
            self.flip_at = self.next_flip(1.0);
        }
    }

    /// Where the head looks next. From a corner it mostly swings straight
    /// across to the opposite one.
    fn next_gaze(&mut self) -> [f32; 2] {
        let [px, py] = self.gaze_dir;
        if px != 0.0 || py != 0.0 {
            let p = self.rng.next();
            if p < 0.66 {
                return [-px, -py];
            }
            if p < 0.85 {
                return [-px, py];
            }
            return [0.0, 0.0];
        }
        let corners = [[1.0, -1.0], [-1.0, 1.0], [-1.0, -1.0], [1.0, 1.0]];
        corners[(self.rng.next() * 4.0) as usize % 4]
    }

    /// When the next idle jump is due: `every` seconds, give or take 40%.
    fn next_flip(&mut self, k: f32) -> f32 {
        let mut rng = self.rng;
        let next = self.flip_at_with(&mut rng, k);
        self.rng = rng;
        next
    }

    fn flip_at_with(&self, rng: &mut Rng, k: f32) -> f32 {
        let every = self.jump.every;
        if every > 0.0 {
            self.t + every * k * (0.625 + rng.next() * 0.75)
        } else {
            f32::INFINITY
        }
    }

    /// Advances by `dt` seconds (already scaled by the speed).
    pub fn update(&mut self, dt: f32) {
        let dt = dt.min(0.05);
        self.t += dt;
        let t = self.t;
        let mut p = self.pose;

        if self.tr < 1.0 {
            self.tr = (self.tr + dt / self.tr_duration).min(1.0);
            let e = ease_sine(self.tr);
            for (i, w) in p.w.iter_mut().enumerate() {
                let target = if i == self.state.index() { 1.0 } else { 0.0 };
                *w = self.w_from[i] + (target - self.w_from[i]) * e;
            }
        }
        let [wd, ww, ws] = p.w;

        // Rest targets, blended.
        let mut rest = Rest {
            pitch: 0.0,
            roll: 0.0,
            y: 0.0,
            look_x: 0.0,
            look_y: 0.0,
        };
        for (i, weight) in p.w.iter().enumerate() {
            let r = Rest::of(BotAvatarState::ALL[i]);
            rest.pitch += r.pitch * weight;
            rest.roll += r.roll * weight;
            rest.y += r.y * weight;
            rest.look_x += r.look_x * weight;
            rest.look_y += r.look_y * weight;
        }

        // The idle gaze: a place to look, a while to stay, then the swing.
        if self.state == BotAvatarState::Idle && t >= self.gaze_at {
            let [gx, gy] = self.next_gaze();
            self.gaze_dir = [gx, gy];
            let reach = 0.84 + self.rng.next() * 0.16;
            self.yaw_w.aim(gx * GAZE_YAW * reach * self.turn);
            self.pitch_w.aim(gy * GAZE_PITCH * reach);
            self.roll_w.aim(gx * GAZE_ROLL * reach * self.turn);
            self.gaze_at = t + GAZE_HOLD_MIN + self.rng.next() * (GAZE_HOLD_MAX - GAZE_HOLD_MIN);
        }

        self.yaw_w.update(t, dt, &mut self.rng);
        self.pitch_w.update(t, dt, &mut self.rng);
        self.roll_w.update(t, dt, &mut self.rng);
        self.look_x_w.update(t, dt, &mut self.rng);
        self.look_y_w.update(t, dt, &mut self.rng);

        // Following the pointer: the eyes lead, the head turns after them.
        self.ptr_s = approach(self.ptr_s, self.ptr_target_s, 8.0, dt);
        self.ptr_x = approach(self.ptr_x, self.ptr_target_x, 14.0, dt);
        self.ptr_y = approach(self.ptr_y, self.ptr_target_y, 14.0, dt);
        let ps = self.ptr_s;
        let quiet = 1.0 - 0.75 * ps;

        self.base_yaw = approach(
            self.base_yaw,
            self.yaw_w.value * quiet + 22.0 * TO_DEG * self.ptr_x * ps,
            5.0,
            dt,
        );
        let base_pitch = rest.pitch + self.pitch_w.value * quiet - 12.0 * TO_DEG * self.ptr_y * ps;
        let base_roll = rest.roll + self.roll_w.value * quiet;
        let base_y = rest.y;
        let base_look_x = rest.look_x + self.look_x_w.value * quiet + 4.5 * self.ptr_x * ps;
        let base_look_y = rest.look_y + self.look_y_w.value * quiet + 3.0 * self.ptr_y * ps;

        // ── events ──
        let mut spin = 0.0;
        let mut hop_y = 0.0;
        let mut sx = 1.0;
        let mut sy = 1.0;
        let mut pitch_add = 0.0;
        let mut roll_add = 0.0;
        let mut blink_close = 0.0;
        let mut look_x_add = 0.0;
        let mut look_y_add = 0.0;
        let mut laugh: f32 = 0.0;
        let mut whirl: f32 = 0.0;
        let mut whirl_angle = 0.0;
        let envelope = |q: f32| smoothstep(0.1, 0.26, q) * (1.0 - smoothstep(0.66, 0.9, q));
        let ring_angle = |q: f32| TAU * (1.5 * q + 0.9 * ease_in_out(q));

        // Blinks: idle and working blink; a double blink now and then.
        if t >= self.blink_at && !self.blink.active() && wd + ww > 0.5 {
            self.blink.fire();
            self.blink_again = !self.blink_again && self.rng.next() < 0.22;
            self.blink_at = t + if self.blink_again {
                0.28
            } else {
                2.2 + self.rng.next() * 2.6
            };
        }
        self.blink.update(dt);
        if self.blink.active() {
            blink_close = (std::f32::consts::PI * self.blink.p).sin();
        }

        // Eye darts: a quick glance to the side and back.
        if t >= self.dart_at && !self.dart.active() && wd + ww > 0.5 {
            self.dart.fire();
            self.dart_x = (self.rng.next() * 2.0 - 1.0) * 4.0;
            self.dart_y = (self.rng.next() * 2.0 - 1.0) * 2.0;
            self.dart.duration = 0.25 + self.rng.next() * 0.45;
            self.dart_at = t + 1.2 + self.rng.next() * 2.6;
        }
        self.dart.update(dt);
        if self.dart.active() {
            let q = self.dart.p;
            let hold = if q < 0.15 {
                q / 0.15
            } else if q > 0.8 {
                (1.0 - q) / 0.2
            } else {
                1.0
            };
            look_x_add += self.dart_x * hold * (wd + ww);
            look_y_add += self.dart_y * hold * (wd + ww);
        }

        // Idle: a full turn now and then, with a jump.
        if self.state == BotAvatarState::Idle && t >= self.flip_at && !self.flip.active() {
            self.flip_poked = false;
            self.flip.duration = flip_duration(&self.jump, false);
            self.flip_side = if self.rng.next() < 0.5 { -1.0 } else { 1.0 };
            self.flip.fire();
            self.flip_at = self.next_flip(1.0);
        }
        self.flip.update(dt);
        if self.flip.active() {
            let j = self.jump;
            let pre_s = flip_pre(&j, self.flip_poked);
            let a = (self.flip.p * self.flip.duration - pre_s) / j.time;
            let q = a.clamp(0.0, 1.0);
            let arc = (std::f32::consts::PI * q).sin();
            spin += TAU * j.spin * ease_in_out(q);
            hop_y -= j.height * arc;
            let tl = (a - 1.0) * j.time - j.land;
            let squash_time = if self.flip_poked {
                j.click_squash_time
            } else {
                j.squash_time
            };
            let crouch = |u: f32| {
                if self.flip_poked {
                    u * u * (3.0 - 2.0 * u)
                } else {
                    hop_squash(u - 1.0)
                }
            };
            let hold = j.ground_time.max(0.0);
            let peak_t = squash_peak(j.squash_ease) * squash_time;
            let rise = tl - peak_t - hold;
            let depth = if tl <= peak_t {
                squash_pulse(tl / squash_time, j.squash_ease)
            } else if rise <= 0.0 {
                ground_shape((tl - peak_t) / hold, j.ground_ease)
            } else {
                rise_pulse(rise / j.rise_time, j.rise_ease)
            };
            let land = if a < 0.0 {
                crouch((1.0 + (a * j.time) / pre_s).max(0.0))
            } else if tl > 0.0 {
                depth
            } else {
                hop_squash(a)
            };
            let land = land * j.squash;
            sx += 0.16 * land - 0.06 * arc * j.stretch;
            sy += -0.18 * land + 0.09 * arc * j.stretch;
            roll_add += self.flip_side * j.lean * TO_DEG * arc;
            if j.spin > 0.0 {
                laugh = laugh.max(arc);
                whirl = whirl.max(envelope(q));
                whirl_angle = ring_angle(q);
            }
        }

        // Working: hops all the time; every third one spins. Leaving the
        // state, the hop under way keeps the strength it had.
        let j = self.jump;
        let exit_hold = j.ground_time.max(0.0);
        let exit_for = exit_hold + j.rise_time;
        if self.state == BotAvatarState::Working {
            self.hop_gain = ww;
        }
        if self.hop_gain > 0.02 && (self.state == BotAvatarState::Working || self.hop_phase > 0.0) {
            self.hop_phase += dt / HOP_T;
            if self.hop_phase >= 1.0 {
                if self.state == BotAvatarState::Working {
                    self.hop_phase -= 1.0;
                    self.hop_count += 1;
                } else if (self.hop_phase - 1.0) * HOP_T >= exit_for {
                    self.hop_phase = 0.0;
                    self.hop_gain = 0.0;
                }
            }
            let g = self.hop_gain;
            let q = self.hop_phase.min(1.0);
            let arc = (std::f32::consts::PI * q).sin();
            let spinning = self.hop_count % 3 == 2;
            let h = if spinning { HOP_SPIN_H } else { 18.0 };
            hop_y -= h * arc * g;
            let exit_t = if self.state != BotAvatarState::Working && self.hop_phase > 1.0 {
                (self.hop_phase - 1.0) * HOP_T
            } else {
                -1.0
            };
            let land = if exit_t < 0.0 {
                hop_squash(self.hop_phase)
            } else if exit_t < exit_hold {
                ground_shape(exit_t / exit_hold, j.ground_ease)
            } else {
                rise_pulse((exit_t - exit_hold) / j.rise_time, j.rise_ease)
            };
            sx += (0.16 * land - 0.06 * arc) * g;
            sy += (-0.18 * land + 0.09 * arc) * g;
            if spinning {
                spin += TAU * ease_in_out(q) * g;
                laugh = laugh.max(arc * g);
                if envelope(q) * g > whirl {
                    whirl = envelope(q) * g;
                    whirl_angle = ring_angle(q);
                }
            }
            roll_add += (if self.hop_count % 2 == 0 { 1.0 } else { -1.0 }) * 6.0 * TO_DEG * arc * g;
        }

        // Working: now and then a laugh shuts the eyes into arcs.
        if self.state == BotAvatarState::Working && t >= self.laugh_at && !self.laugh.active() {
            self.laugh.fire();
            self.laugh.duration = 0.6 + self.rng.next() * 0.5;
            self.laugh_at = t + 1.6 + self.rng.next() * 2.2;
        }
        self.laugh.update(dt);
        if self.laugh.active() {
            let q = self.laugh.p;
            laugh = laugh.max(if q < 0.18 {
                q / 0.18
            } else if q > 0.78 {
                (1.0 - q) / 0.22
            } else {
                1.0
            });
        }

        // Sleeping: the head drops, then jerks back up.
        if self.state == BotAvatarState::Sleeping && t >= self.nod_at && !self.nod.active() {
            self.nod.fire();
            self.nod_at = t + 4.0 + self.rng.next() * 4.0;
        }
        self.nod.update(dt);
        if self.nod.active() {
            let q = self.nod.p;
            let dip = if q < 0.72 {
                ease_in_out(q / 0.72)
            } else {
                1.0 - ease_in_out((q - 0.72) / 0.28)
            };
            pitch_add -= 13.0 * TO_DEG * dip * ws;
        }

        // Breathing, always, deeper and slower asleep.
        self.breath_phase += dt / (3.6 + 1.2 * ws);
        let breath = (self.breath_phase * TAU).sin();
        p.breath = breath;
        sx += breath * (0.008 + 0.014 * ws);
        sy += breath * (0.012 + 0.02 * ws);
        let bob = (t * TAU / 3.4).sin() * 2.0 * (1.0 - ws);

        // ── compose ──
        p.yaw = self.base_yaw + spin;

        // The jelly: the faster the head turns, the more the body stretches
        // along the turn, on a spring that overshoots and settles.
        let mut dyaw = self.base_yaw - self.prev_yaw;
        dyaw = ((dyaw + std::f32::consts::PI) % TAU + TAU) % TAU - std::f32::consts::PI;
        self.prev_yaw = self.base_yaw;
        let rate = if dt > 0.0 { dyaw.abs() / dt } else { 0.0 };
        let lead_target = if dt > 0.0 {
            ((dyaw / dt) * 2.4).clamp(-2.2, 2.2)
        } else {
            0.0
        };
        self.gaze_lead = approach(self.gaze_lead, lead_target, 9.0, dt);
        let jelly_target = (0.055 * rate).min(0.22);
        let (omega, zeta) = (16.0, 0.45);
        self.jelly_v +=
            (omega * omega * (jelly_target - self.jelly) - 2.0 * zeta * omega * self.jelly_v) * dt;
        self.jelly += self.jelly_v * dt;
        let jelly = self.jelly.clamp(-0.08, 0.28) * 0.6;
        sx *= 1.0 + jelly;
        sy *= 1.0 - 0.55 * jelly;

        p.pitch = base_pitch + pitch_add;
        p.roll = base_roll + roll_add;
        p.x = 0.0;
        p.y = base_y + hop_y + bob;
        p.sx = sx;
        p.sy = sy;
        p.eye_open = 1.0;
        p.laugh = approach(p.laugh, laugh, 30.0, dt);
        p.blink_l = blink_close;
        p.blink_r = blink_close;
        p.look_x = base_look_x + look_x_add + self.gaze_lead;
        p.look_y = base_look_y + look_y_add;
        p.whirl = whirl;
        p.whirl_angle = whirl_angle;
        self.pose = p;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(state: BotAvatarState) -> BotAvatarSim {
        let mut sim = BotAvatarSim::new(0.4, state);
        for _ in 0..600 {
            sim.update(1.0 / 60.0);
        }
        sim
    }

    #[test]
    fn poses_stay_finite_and_in_range() {
        for state in BotAvatarState::ALL {
            let sim = settle(state);
            let p = sim.pose();
            for v in [
                p.yaw, p.pitch, p.roll, p.x, p.y, p.sx, p.sy, p.look_x, p.look_y,
            ] {
                assert!(v.is_finite(), "{state:?} pose has a non-finite value");
            }
            assert!(p.yaw.abs() < std::f32::consts::TAU);
            assert!(p.sx > 0.2 && p.sx < 2.5, "sx {}", p.sx);
            assert!(p.sy > 0.2 && p.sy < 2.5, "sy {}", p.sy);
            let sum: f32 = p.w.iter().sum();
            assert!((sum - 1.0).abs() < 1e-3, "weights do not sum to 1: {sum}");
        }
    }

    #[test]
    fn a_state_switch_eases_and_lands_on_the_new_state() {
        let mut sim = settle(BotAvatarState::Idle);
        assert!(sim.pose().w[0] > 0.99);
        sim.set_state(BotAvatarState::Sleeping, false);
        for _ in 0..10 {
            sim.update(1.0 / 60.0);
        }
        let mid = sim.pose().w;
        assert!(
            mid[0] < 0.99 && mid[2] > 0.0,
            "switch should have started: {mid:?}"
        );
        for _ in 0..120 {
            sim.update(1.0 / 60.0);
        }
        let end = sim.pose().w;
        assert!(end[2] > 0.99, "switch should have landed: {end:?}");
        // Sleeping rests with the head down.
        assert!(sim.pose().pitch < 0.0);
    }

    #[test]
    fn seeds_do_not_share_a_phase() {
        let a = settle(BotAvatarState::Idle);
        let mut other = BotAvatarSim::new(0.83, BotAvatarState::Idle);
        other.update(1.0 / 60.0);
        assert_ne!(a.pose().yaw.to_bits(), other.pose().yaw.to_bits());
        // The same seed always lands on the same phase.
        let twin = settle(BotAvatarState::Idle);
        assert_eq!(a.pose().yaw.to_bits(), twin.pose().yaw.to_bits());
    }

    #[test]
    fn working_hops() {
        let mut sim = settle(BotAvatarState::Working);
        let mut lowest = 0.0_f32;
        let mut highest = 0.0_f32;
        for _ in 0..120 {
            sim.update(1.0 / 60.0);
            lowest = lowest.min(sim.pose().y);
            highest = highest.max(sim.pose().y);
        }
        assert!(lowest < -8.0, "no hop: lowest {lowest}");
        assert!(highest > 0.0);
    }
}
