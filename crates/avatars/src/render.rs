//! The renderer: a body built from a stack of its own outline, spaced along a
//! depth axis with a pillow profile, projected with the head's yaw and pitch,
//! so the stack reads as a rounded solid that turns, flips and shows its side.
//! The face lives on the front cap and follows it.
//!
//! Rune paints flats, so the upstream canvas gradients are banded: the front
//! of the body is split into a few slices along the light's axis and each band
//! takes the colour the gradient has there. At avatar sizes the bands read as
//! a gradient and cost a handful of polygon fills.

use crate::color::Rgb;
use crate::engine::Pose;
use crate::material::{self, BodyImage, Material};
use crate::svg::{Affine, Outline};
use crate::{BotAvatarFace, BotAvatarShading, BotAvatarType};
use rune::{LineCap, Shape};

/// The canvas is drawn larger than the avatar's layout box, so a hop or a
/// flip can leave the box without being clipped.
pub const OVERSCAN: f32 = 1.5;
/// The body's centre sits this fraction of the box below the canvas centre:
/// hops and flips need the room above, not below.
pub const RISE: f32 = 0.1;

/// Copies through the depth.
const SLICES: usize = 17;
/// The stock half-depth, in body units.
const HALF_DEPTH: f32 = 15.0;
/// The cap scale at the ends of the pillow, at the stock rim width.
const CAP: f32 = 0.9;

/// Radius of the sphere the face is drawn on, in body units.
const FACE_R: f32 = 30.0;
/// Big, plain dark eyes: they carry the face at 24px.
const EYE_GAP: f32 = 25.0;
const EYE_RX: f32 = 6.3;

const WHIRL_SEGMENTS: usize = 34;
const WHIRL_SPAN: f32 = std::f32::consts::PI * 1.55;
const WHIRL_RX: f32 = 57.0;
const WHIRL_RATIO: f32 = 0.4;
const WHIRL_TILT: f32 = -0.28;

fn profile(z: f32, cap: f32) -> f32 {
    let inside = 1.0 - z * z;
    cap + (1.0 - cap) * if inside > 0.0 { inside.sqrt() } else { 0.0 }
}

/// The resolved look of one avatar: everything the renderer needs that the
/// pose does not move.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub body: Rgb,
    pub ink: Rgb,
    pub face: BotAvatarFace,
    pub face_x: f32,
    pub face_y: f32,
    pub face_scale: f32,
    pub shading: BotAvatarShading,
    pub shadow: f32,
    pub highlight: f32,
    pub depth: f32,
    /// Degrees clockwise from the top.
    pub light: f32,
    pub rim: f32,
    pub whirl_strength: f32,
    pub whirl_size: f32,
    pub whirl_width: f32,
    pub whirl_length: f32,
    pub whirl_tilt: f32,
}

impl Look {
    /// A type's stock look at the given shading.
    pub fn new(kind: crate::BotAvatarType, shading: BotAvatarShading) -> Look {
        let preset = kind.preset();
        let vivid = (preset.saturation.clamp(1.5, 2.5) - 1.5).max(0.0);
        let body = if preset.brightness == 1.0 && preset.saturation == 1.0 {
            Rgb::from_css(preset.color).unwrap_or_else(|| Rgb::from_hex(0x41c4ff))
        } else {
            Rgb::from_css(preset.color)
                .unwrap_or_else(|| Rgb::from_hex(0x41c4ff))
                .shade(
                    (preset.brightness.clamp(0.0, 2.0) - 1.0) * 0.35,
                    (preset.saturation.clamp(0.0, 1.5) - 1.0) * 0.5,
                )
        }
        .richer(vivid);
        let fabric = shading == BotAvatarShading::Fabric;
        Look {
            body,
            ink: Rgb::new(1.0, 1.0, 1.0),
            face: preset.face,
            face_x: preset.face_x,
            face_y: preset.face_y,
            face_scale: preset.face_scale,
            shading,
            shadow: if fabric { 1.15 } else { 0.35 },
            highlight: if fabric { 1.45 } else { 1.3 },
            depth: 0.65,
            light: if fabric { 295.0 } else { 300.0 },
            rim: if fabric { 0.6 } else { 0.5 },
            whirl_strength: 0.0,
            whirl_size: 1.0,
            whirl_width: 1.0,
            whirl_length: 1.0,
            whirl_tilt: 1.0,
        }
    }
}

struct Palette {
    base: Rgb,
    near: Rgb,
    light: Rgb,
    dark: Rgb,
    crisp_mix: Vec<Rgb>,
    smooth_mix: Vec<Rgb>,
}

fn mix_hsl(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let pa = a.to_hsl();
    let pb = b.to_hsl();
    Rgb::from_hsl([
        pa[0] + (pb[0] - pa[0]) * t,
        pa[1] + (pb[1] - pa[1]) * t,
        pa[2] + (pb[2] - pa[2]) * t,
    ])
}

fn palette(color: Rgb, shadow: f32, highlight: f32) -> Palette {
    let far = color.shade(-0.3 * shadow, 0.05 * shadow);
    let near = color.shade(-0.12 * shadow, 0.03 * shadow);
    let mut crisp_mix = Vec::with_capacity(SLICES);
    let mut smooth_mix = Vec::with_capacity(SLICES);
    for j in 0..SLICES {
        let t = j as f32 / (SLICES - 1) as f32;
        // Past 0.6 the lit gradient takes over; the value is unused.
        crisp_mix.push(if t > 0.6 {
            near
        } else {
            mix_hsl(far, near, t / 0.6)
        });
        smooth_mix.push(if t >= 0.5 {
            color
        } else {
            mix_hsl(far, color, t / 0.5)
        });
    }
    Palette {
        base: color,
        near,
        light: color.shade(0.04 * highlight, 0.0),
        dark: color.shade(-0.3 * shadow, 0.05 * shadow),
        crisp_mix,
        smooth_mix,
    }
}

fn sample_stops(stops: &[(f32, Rgb)], t: f32) -> Rgb {
    if stops.is_empty() {
        return Rgb::new(0.0, 0.0, 0.0);
    }
    if t <= stops[0].0 {
        return stops[0].1;
    }
    for pair in stops.windows(2) {
        let (t0, c0) = pair[0];
        let (t1, c1) = pair[1];
        if t <= t1 {
            let k = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
            return c0.mix(c1, k);
        }
    }
    stops[stops.len() - 1].1
}

/// Fills `points` with `stops` spread along the gradient axis `from` → `to`,
/// band by band.
///
/// The axis is fixed in canvas space rather than taken from each polygon, so
/// the bands of two nested polygons line up: without that, the rim and the cap
/// would each stretch the gradient to their own bounds and a seam would show
/// where they meet.
fn banded(
    mut shape: Shape,
    points: &[[f32; 2]],
    from: [f32; 2],
    to: [f32; 2],
    stops: &[(f32, Rgb)],
    bands: usize,
) -> Shape {
    if points.len() < 3 {
        return shape;
    }
    let axis = [to[0] - from[0], to[1] - from[1]];
    let len2 = axis[0] * axis[0] + axis[1] * axis[1];
    if len2 < 1e-6 {
        return shape.fill(points.to_vec(), sample_stops(stops, 0.5).color());
    }
    let project = |p: [f32; 2]| ((p[0] - from[0]) * axis[0] + (p[1] - from[1]) * axis[1]) / len2;
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for p in points {
        let t = project(*p);
        min = min.min(t);
        max = max.max(t);
    }
    let (first, last) = (
        ((min * bands as f32).floor() as i32).max(-1),
        ((max * bands as f32).ceil() as i32).min(bands as i32),
    );
    // A hair of overlap keeps the hard interior edges of adjacent bands from
    // reading as seams: the later band's fill covers the earlier one's edge.
    let step = 1.0 / bands as f32;
    for k in first..last {
        let lo = k as f32 * step;
        let hi = lo + step * 1.55;
        let clipped = clip_projected(points, from, axis, len2, lo, hi);
        if clipped.len() < 3 {
            continue;
        }
        let color = sample_stops(stops, (lo + step * 0.5).clamp(0.0, 1.0));
        shape = shape.fill(clipped, color.color());
    }
    shape
}

/// Clips a polygon to the band `lo <= t <= hi` of the gradient axis.
fn clip_projected(
    points: &[[f32; 2]],
    from: [f32; 2],
    axis: [f32; 2],
    len2: f32,
    lo: f32,
    hi: f32,
) -> Vec<[f32; 2]> {
    // Work in a space where the axis is +x and `t` is the x coordinate.
    let scale = len2.sqrt();
    let dir = [axis[0] / scale, axis[1] / scale];
    let t = |p: [f32; 2]| ((p[0] - from[0]) * dir[0] + (p[1] - from[1]) * dir[1]) / scale;
    let mut poly = points.to_vec();
    for (offset, keep_greater) in [(lo, true), (hi, false)] {
        let mut out = Vec::with_capacity(poly.len() + 2);
        for i in 0..poly.len() {
            let a = poly[i];
            let b = poly[(i + 1) % poly.len()];
            let (sa, sb) = (t(a) - offset, t(b) - offset);
            if (sa >= 0.0) == keep_greater {
                out.push(a);
            }
            if (sa >= 0.0) != (sb >= 0.0) {
                let k = sa / (sa - sb);
                out.push([a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k]);
            }
        }
        if out.len() < 3 {
            return Vec::new();
        }
        poly = out;
    }
    poly
}

fn fill_outline(mut shape: Shape, outline: &Outline, color: Rgb) -> Shape {
    for sub in &outline.subpaths {
        if sub.points.len() >= 3 {
            shape = shape.fill(sub.points.clone(), color.color());
        }
    }
    shape
}

/// Draws one frame into a shape whose coordinates are canvas points, the
/// canvas being `size * OVERSCAN` square. `pose` is the avatar's rig pose.
pub fn frame(outline: &Outline, parts: &[Outline], look: &Look, pose: &Pose, size: f32) -> Shape {
    layers(outline, parts, look, pose, size, None).front
}

pub(crate) struct Projection {
    yaw: (f32, f32),
    pitch: (f32, f32),
    roll: (f32, f32),
    body: Affine,
}

impl Projection {
    fn rotate(&self, [x, y, z]: [f32; 3]) -> [f32; 3] {
        let (sy, cy) = self.yaw;
        let (sp, cp) = self.pitch;
        let depth = cy * z - sy * x;
        [cy * x + sy * z, cp * y - sp * depth, sp * y + cp * depth]
    }

    pub(crate) fn point(&self, point: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = self.rotate(point);
        let [x, y] = self.body.apply([x, y]);
        [x, y, z]
    }

    pub(crate) fn normal(&self, normal: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = self.rotate(normal);
        let (sr, cr) = self.roll;
        [cr * x - sr * y, sr * x + cr * y, z]
    }
}

pub(crate) struct Layers {
    pub back: Shape,
    pub front: Shape,
    pub image: Option<BodyImage>,
}

pub(crate) fn layers(
    outline: &Outline,
    parts: &[Outline],
    look: &Look,
    pose: &Pose,
    size: f32,
    material: Option<(BotAvatarType, Material)>,
) -> Layers {
    let full = size * OVERSCAN;
    let scale = size / 100.0;

    let half_depth = HALF_DEPTH * look.depth;
    let rim = look.rim.clamp(0.0, 2.0);
    let cap = 1.0 - (1.0 - CAP) * rim;
    let la = look.light.to_radians();
    let (lx, ly) = (la.sin(), -la.cos());
    let (sy, cy0) = pose.yaw.sin_cos();
    let (sp, cp0) = pose.pitch.sin_cos();
    let facing = cy0 * cp0;
    let floor = |v: f32| {
        if v.abs() < 0.22 {
            if v < 0.0 {
                -0.22
            } else {
                0.22
            }
        } else {
            v
        }
    };
    let (cy, cp) = (floor(cy0), floor(cp0));

    // Body space: the box centre plus the pose's offset, its roll and squash.
    // The squash and stretch scale about the body's base (y = 50), so a
    // landing keeps the feet on the ground and a stretch rises from the base.
    let (sr, cr) = pose.roll.sin_cos();
    let (kx, ky) = (pose.sx * scale, pose.sy * scale);
    let lift = 50.0 * (1.0 - pose.sy) * scale;
    let body = Affine {
        a: cr * kx,
        b: sr * kx,
        c: -sr * ky,
        d: cr * ky,
        e: full / 2.0 + pose.x * scale - sr * lift,
        f: full / 2.0 + RISE * size + pose.y * scale + cr * lift,
    };
    // The width of a body-unit stroke on screen.
    let width = (kx * ky).abs().sqrt();
    let image = material.map(|(kind, material)| {
        let projection = Projection {
            yaw: (sy, cy0),
            pitch: (sp, cp0),
            roll: (sr, cr),
            body,
        };
        material::draw(kind, material, &projection, look, full)
    });

    // The far half of the whirl sits behind everything.
    let rear_whirl = whirl(Shape::new(), pose, look, lx, ly, false, body, width);
    let (back, mut shape) = if image.is_some() {
        (rear_whirl, Shape::new())
    } else {
        (Shape::new(), rear_whirl)
    };
    if image.is_none() {
        let mode = look.shading;
        let bands = if size < 40.0 { 7 } else { 12 };
        let palette = palette(look.body, look.shadow, look.highlight);
        let gradient = (
            body.apply([lx * 56.0, ly * 56.0]),
            body.apply([-lx * 56.0, -ly * 56.0]),
        );
        // Thin parts (antennae) sit behind the body at a fraction of its depth.
        for part in parts {
            shape = solid_slices(
                shape,
                part,
                mode,
                &palette,
                gradient,
                cap,
                cy,
                sy,
                cp,
                sp,
                half_depth * 0.4,
                facing,
                body,
                bands,
            );
        }
        shape = solid_slices(
            shape, outline, mode, &palette, gradient, cap, cy, sy, cp, sp, half_depth, facing,
            body, bands,
        );
    }

    // Clip the face against the rendered body so the eyes slide out of sight
    // at its silhouette as the head turns.
    if facing > -0.2 {
        let face_affine = Affine::scale(look.face_scale, look.face_scale)
            .then(Affine::translate(look.face_x - 50.0, look.face_y - 50.0))
            .then(body);
        let front = image.is_none().then(|| {
            let zf = if facing >= 0.0 { 1.0 } else { -1.0 };
            let sf = profile(zf, cap);
            let to_face = Affine {
                a: cy * sf,
                b: sy * sp * sf,
                c: 0.0,
                d: cp * sf,
                e: zf * sy * half_depth - 50.0 * cy * sf,
                f: -zf * cy * sp * half_depth - 50.0 * sy * sp * sf - 50.0 * cp * sf,
            }
            .then(body)
            .then(face_affine.inverse());
            outline.transformed(|p| to_face.apply(p))
        });
        let contains = |point| match &image {
            Some(image) => image.covers(face_affine.apply(point), full),
            None => front.as_ref().is_some_and(|front| front.contains(point)),
        };
        shape = face(
            shape,
            pose,
            look,
            contains,
            face_affine,
            width * look.face_scale,
        );
    }

    // The near half of the whirl passes in front of the face.
    shape = whirl(shape, pose, look, lx, ly, true, body, width);

    Layers {
        back,
        front: shape,
        image,
    }
}

#[allow(clippy::too_many_arguments)]
fn solid_slices(
    mut shape: Shape,
    outline: &Outline,
    mode: BotAvatarShading,
    palette: &Palette,
    gradient: ([f32; 2], [f32; 2]),
    cap: f32,
    cy: f32,
    sy: f32,
    cp: f32,
    sp: f32,
    half_depth: f32,
    facing: f32,
    body: Affine,
    bands: usize,
) -> Shape {
    let last = SLICES - 1;
    let order = if facing >= 0.0 { 1.0 } else { -1.0 };
    // The near slices share one gradient, so only the largest of them (the lit
    // rim) and the cap paint it; the rest would only repaint the same colours.
    let lit_stops = [
        (0.0, palette.light),
        (0.45, palette.near),
        (1.0, palette.dark),
    ];

    for j in 0..SLICES {
        let k = if order > 0.0 { j } else { last - j };
        let z = -1.0 + 2.0 * k as f32 / last as f32;
        let s = profile(z, cap);
        let near = j as f32 / last as f32;
        let m0 = cy * s;
        let m1 = sy * sp * s;
        let m3 = cp * s;
        let e = z * sy * half_depth - 50.0 * m0;
        let fo = -z * cy * sp * half_depth - 50.0 * m1 - 50.0 * m3;
        let affine = Affine {
            a: m0,
            b: m1,
            c: 0.0,
            d: m3,
            e,
            f: fo,
        }
        .then(body);
        let points = outline.transformed(|p| affine.apply(p));
        let is_cap = j == last;
        // The near slices share one gradient and are nearly nested, so the
        // largest of them carries the lit rim; the rest would only repaint it.
        let lit_rim = mode == BotAvatarShading::Crisp && !is_cap && (near - 0.6).abs() < 0.03;
        let hidden = mode == BotAvatarShading::Crisp && !is_cap && near > 0.6 && !lit_rim;
        match mode {
            BotAvatarShading::Flat => shape = fill_outline(shape, &points, palette.base),
            BotAvatarShading::Smooth | BotAvatarShading::Plastic | BotAvatarShading::Fabric => {
                shape = fill_outline(shape, &points, palette.smooth_mix[j]);
            }
            BotAvatarShading::Crisp if hidden => {}
            BotAvatarShading::Crisp if is_cap || lit_rim => {
                for sub in &points.subpaths {
                    if sub.points.len() >= 3 {
                        shape = banded(
                            shape,
                            &sub.points,
                            gradient.0,
                            gradient.1,
                            &lit_stops,
                            bands,
                        );
                    }
                }
            }
            BotAvatarShading::Crisp => shape = fill_outline(shape, &points, palette.crisp_mix[j]),
        }
    }
    shape
}

/// A place on the face's sphere, turned by the head: its screen offset, its
/// foreshortening, and how much it faces the viewer.
struct Spot {
    x: f32,
    y: f32,
    sx: f32,
    sy: f32,
    z: f32,
}

fn on_sphere(x: f32, y: f32, yaw: f32, pitch: f32) -> Spot {
    let lon = (-x / FACE_R).clamp(-1.0, 1.0).asin() + yaw;
    let lat = (-y / FACE_R).clamp(-1.0, 1.0).asin() + pitch;
    let cl = lat.cos();
    Spot {
        x: FACE_R * lon.sin() * cl,
        y: -FACE_R * lat.sin(),
        sx: lon.cos(),
        sy: cl,
        z: lon.cos() * cl,
    }
}

const EYE_STEPS: usize = 8;

/// An eye's curve as a short polyline: endpoints at ±x0, y0, a control point
/// at 0, cy.
fn eye_points(x0: f32, y0: f32, cy: f32) -> Vec<[f32; 2]> {
    let mut points = Vec::with_capacity(EYE_STEPS + 1);
    points.push([-x0, y0]);
    for i in 1..=EYE_STEPS {
        let t = i as f32 / EYE_STEPS as f32;
        let mt = 1.0 - t;
        points.push([
            mt * mt * -x0 + t * t * x0,
            (mt * mt + t * t) * y0 + 2.0 * mt * t * cy,
        ]);
    }
    points
}

/// The mouth's outline: corners at ±hw on y = 0, two cubic edges, round caps
/// at the corners.
fn mouth_points(hw: f32, t0: f32, a: f32, yt: f32, ab: f32, yb: f32) -> Vec<[f32; 2]> {
    let sin = 0.684_f32.sin();
    let cos = 0.684_f32.cos();
    let (nx, ny) = (t0 * sin, t0 * cos);
    let (ltx, lty) = (-hw + nx, -ny);
    let (rtx, rty) = (hw - nx, -ny);
    let (lbx, lby) = (-hw - nx, ny);
    let (rbx, rby) = (hw + nx, ny);
    let (cx, cy) = (4.0 / 3.0 * t0 * cos, 4.0 / 3.0 * t0 * sin);
    let mut points = vec![[ltx, lty]];
    cubic(
        &mut points,
        [ltx, lty],
        [-hw + a * hw, yt - t0],
        [hw - a * hw, yt - t0],
        [rtx, rty],
        6,
    );
    cubic(
        &mut points,
        [rtx, rty],
        [rtx + cx, rty - cy],
        [rbx + cx, rby - cy],
        [rbx, rby],
        6,
    );
    cubic(
        &mut points,
        [rbx, rby],
        [hw - ab * hw, yb + t0],
        [-hw + ab * hw, yb + t0],
        [lbx, lby],
        6,
    );
    cubic(
        &mut points,
        [lbx, lby],
        [lbx - cx, lby - cy],
        [ltx - cx, lty - cy],
        [ltx, lty],
        6,
    );
    points
}

fn cubic(
    out: &mut Vec<[f32; 2]>,
    from: [f32; 2],
    c1: [f32; 2],
    c2: [f32; 2],
    to: [f32; 2],
    steps: usize,
) {
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let mt = 1.0 - t;
        let (a, b, c, d) = (mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t);
        out.push([
            a * from[0] + b * c1[0] + c * c2[0] + d * to[0],
            a * from[1] + b * c1[1] + c * c2[1] + d * to[1],
        ]);
    }
}

fn face(
    mut shape: Shape,
    pose: &Pose,
    look: &Look,
    contains: impl Fn([f32; 2]) -> bool,
    affine: Affine,
    unit: f32,
) -> Shape {
    let [wd, ww, ws] = pose.w;
    let ink = look.ink;
    let ey = if look.face == BotAvatarFace::Mouth {
        -3.5
    } else {
        1.0
    };
    let half = EYE_GAP / 2.0;
    let (look_x, look_y) = (pose.look_x, pose.look_y);
    let (yaw, pitch) = (pose.yaw, pose.pitch);

    // Each eye is one stroked curve, so every look is the same shape with
    // different numbers, and a blend of the numbers is a real morph.
    let past = |v: f32, d: f32| {
        if v.abs() <= d {
            0.0
        } else {
            v.signum() * (v.abs() - d) / (1.0 - d)
        }
    };
    let clamp1 = |v: f32| v.clamp(-1.0, 1.0);
    let up = past(clamp1(-pose.pitch / 0.26 - pose.look_y / 7.0), 0.34);
    let side = past(clamp1(pose.look_x / 4.5), 0.4).abs();
    let tall = (1.0 + 0.55 * up - 0.1 * side).max(0.3);
    let wide = 1.0 - 0.05 * up + 0.12 * side;
    let open = wd + ww * (1.0 - pose.laugh);
    let laugh = ww * pose.laugh;
    let lift = (-pose.y).max(0.0) / 26.0;
    let sag = 0.5 + 0.5 * pose.breath;

    for which in [-1.0_f32, 1.0] {
        let lid = if which < 0.0 {
            pose.blink_l
        } else {
            pose.blink_r
        };
        let e = (pose.eye_open * (1.0 - lid)).clamp(0.0, 1.0);
        let k_open = open * e;
        let k_shut = open * (1.0 - e);
        let k_laugh = laugh;
        let k_sleep = ws;
        let x0 = k_open * 0.01 + k_shut * 5.4 + k_laugh * 6.2 + k_sleep * 6.0;
        let y0 = k_open * 1.1 * tall
            + k_shut * 0.6
            + k_laugh * (2.2 - lift * 1.5)
            + k_sleep * (-1.4 + sag);
        let control = k_open * -3.3 * tall
            + k_shut * 0.6
            + k_laugh * (-11.4 - 4.0 * lift)
            + k_sleep * (5.4 + 2.0 * sag);
        let w = k_open * EYE_RX * 2.0 * wide + k_shut * 2.8 + k_laugh * 4.4 + k_sleep * 4.0;
        let dx = look_x * (k_open + 0.5 * (k_shut + k_laugh));
        let dy = look_y * (k_open + 0.5 * k_shut);
        let spot = on_sphere(which * half + dx, ey + dy, yaw, pitch);
        if spot.z <= 0.02 {
            continue;
        }
        let at = Affine::translate(spot.x, spot.y)
            .then(Affine::scale(spot.sx.max(0.02), spot.sy.max(0.02)));
        let place = at.then(affine);
        if !contains([spot.x, spot.y]) {
            continue;
        }
        let alpha = (spot.z * 5.0).min(1.0);
        let points: Vec<[f32; 2]> = eye_points(x0, y0, control)
            .into_iter()
            .map(|p| place.apply(p))
            .collect();
        let stroke = w * unit * (spot.sx.abs() * spot.sy.abs()).sqrt();
        if stroke > 0.2 {
            shape = shape.stroke_with(points, false, stroke, LineCap::Round, ink.alpha(alpha));
        }
    }

    if look.face == BotAvatarFace::Mouth {
        let mx = look_x * 0.35;
        let kd = (0.6 + 0.4 * wd) * (1.0 + 0.06 * pose.breath);
        let kw = (0.6 + 0.4 * ww) * (1.0 + (0.25 * (-pose.y).max(0.0)) / 26.0);
        let r = 2.7 * ws * (1.0 + 0.25 * pose.breath);
        let b = |d: f32, w: f32, s: f32| wd * d + ww * w + ws * s;
        let points = mouth_points(
            b(6.5 * kd, 9.5 * kw, r),
            b(1.9, 0.0, 0.0),
            b(2.0 / 3.0, 2.0 / 3.0, 0.0),
            b(3.53 * kd, 1.6 * kw, (-4.0 * r) / 3.0),
            b(2.0 / 3.0, 0.0, 0.0),
            b(3.53 * kd, 17.3 * kw, (4.0 * r) / 3.0),
        );
        let y = b(12.5, 11.6, 15.5);
        let spot = on_sphere(mx, y, yaw, pitch);
        if spot.z > 0.02 {
            let at = Affine::translate(spot.x, spot.y)
                .then(Affine::scale(spot.sx.max(0.02), spot.sy.max(0.02)));
            let place = at.then(affine);
            if contains([spot.x, spot.y]) {
                let alpha = (spot.z * 5.0).min(1.0);
                let world: Vec<[f32; 2]> = points.into_iter().map(|p| place.apply(p)).collect();
                shape = shape.fill(world, ink.alpha(alpha));
            }
        }
    }
    shape
}

/// The cartoon whirl round a spinning body: one tapered trail on a tilted
/// ring, made of the body's own material.
#[allow(clippy::too_many_arguments)]
fn whirl(
    mut shape: Shape,
    pose: &Pose,
    look: &Look,
    lx: f32,
    ly: f32,
    near: bool,
    body: Affine,
    unit: f32,
) -> Shape {
    let k = (pose.whirl * look.whirl_strength).min(1.0);
    if k <= 0.01 || unit <= 0.0 {
        return shape;
    }
    let span = WHIRL_SPAN * look.whirl_length;
    let (base, light, dark, halo) = (
        look.body.shade(0.1, 0.02),
        look.body.shade(0.3, 0.04),
        look.body.shade(-0.22, 0.08),
        look.body.shade(0.2, 0.0),
    );
    let head = -pose.whirl_angle;
    let rx = WHIRL_RX * look.whirl_size;
    let ry = rx * WHIRL_RATIO * look.whirl_tilt * if near { 1.14 } else { 0.86 };
    let light_angle = ly.atan2(lx) - WHIRL_TILT;
    let place = Affine::translate(0.0, 5.0)
        .then(Affine::rotate(WHIRL_TILT))
        .then(body);
    let seg = Seg {
        rx,
        ry,
        place,
        unit,
    };

    let a = |alpha: f32| (alpha * k).clamp(0.0, 1.0);
    if near {
        for i in 0..WHIRL_SEGMENTS {
            let f = i as f32 / WHIRL_SEGMENTS as f32;
            let a1 = head + f * span;
            let a0 = a1 + span / WHIRL_SEGMENTS as f32 + 0.012;
            if ((a0 + a1) / 2.0).sin() <= 0.0 {
                continue;
            }
            let fade = (1.0 - f).powf(1.3);
            let width = (2.0 + 8.0 * fade) * 1.5 * look.whirl_width;
            seg.stroke(
                &mut shape,
                a0,
                a1,
                width,
                rune::Color {
                    a: a(0.2 * fade),
                    ..rune::Color::BLACK
                },
                3.5,
            );
        }
    }
    for i in 0..WHIRL_SEGMENTS {
        let f = i as f32 / WHIRL_SEGMENTS as f32;
        let a1 = head + f * span;
        let a0 = a1 + span / WHIRL_SEGMENTS as f32 + 0.012;
        let mid = (a0 + a1) / 2.0;
        if ((mid.sin() > 0.0) != near) || mid.sin() == 0.0 {
            continue;
        }
        let depth = 0.6 + 0.4 * mid.sin();
        let fade = (1.0 - f).powf(1.3);
        let puff = 1.0 + 0.18 * (f * 9.0 + 1.2).sin();
        let width = (2.0 + 8.0 * fade) * depth * look.whirl_width * puff;
        let alpha = k * (0.3 + 0.7 * fade) * depth;
        let facing = 0.5 + 0.5 * (mid - light_angle).cos();
        seg.stroke(
            &mut shape,
            a0,
            a1,
            width * 2.6,
            halo.alpha(a(alpha * 0.2)),
            0.0,
        );
        seg.stroke(
            &mut shape,
            a0,
            a1,
            width * 0.8,
            dark.alpha(a(alpha * 0.45)),
            width * 0.32,
        );
        seg.stroke(&mut shape, a0, a1, width, base.alpha(a(alpha * 0.72)), 0.0);
        seg.stroke(
            &mut shape,
            a0,
            a1,
            width * 0.62,
            light.alpha(a(alpha * 0.78 * (0.4 + 0.6 * facing))),
            -width * 0.16,
        );
        seg.stroke(
            &mut shape,
            a0,
            a1,
            width * 0.24,
            rune::Color::WHITE.alpha(a(alpha * 0.9 * (0.15 + 0.85 * facing * facing))),
            -width * 0.3,
        );
    }
    shape
}

/// One stretch of the whirl's ring, stroked at a distance `dy` off the ring.
struct Seg {
    rx: f32,
    ry: f32,
    place: Affine,
    unit: f32,
}

impl Seg {
    fn stroke(
        &self,
        shape: &mut Shape,
        from: f32,
        to: f32,
        width: f32,
        color: rune::Color,
        dy: f32,
    ) {
        if width <= 0.05 {
            return;
        }
        let steps = 5;
        let mut points = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let a = from + (to - from) * i as f32 / steps as f32;
            let (sin, cos) = a.sin_cos();
            points.push(self.place.apply([self.rx * cos, self.ry * sin + dy]));
        }
        *shape = std::mem::take(shape).stroke_with(
            points,
            false,
            width * self.unit,
            LineCap::Round,
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn look(kind: BotAvatarType) -> Look {
        Look::new(kind, BotAvatarShading::Crisp)
    }

    #[test]
    fn every_type_draws_a_body_at_every_state() {
        for kind in BotAvatarType::ALL {
            let look = look(kind);
            let outline = crate::body_outline(kind);
            for state in crate::BotAvatarState::ALL {
                let sim = crate::BotAvatarSim::new(0.3, state);
                let shape = frame(outline, &[], &look, sim.pose(), 64.0);
                assert!(!shape.is_empty(), "{kind:?} drew nothing in {state:?}");
            }
        }
    }

    #[test]
    fn a_turned_head_narrows_the_body() {
        let look = look(BotAvatarType::Circle);
        let outline = crate::body_outline(BotAvatarType::Circle);
        let mut pose = Pose::REST;
        let front = frame(outline, &[], &look, &pose, 100.0);
        pose.yaw = 1.2;
        let turned = frame(outline, &[], &look, &pose, 100.0);
        assert_ne!(front, turned);
    }

    #[test]
    fn material_body_stays_visible_as_the_head_turns_away() {
        let kind = BotAvatarType::Circle;
        let outline = crate::body_outline(kind);
        let size = 64.0;
        let full = size * OVERSCAN;
        for shading in [BotAvatarShading::Plastic, BotAvatarShading::Fabric] {
            let look = Look::new(kind, shading);
            let material = shading.material().unwrap();
            for yaw in [0.0, 1.2, std::f32::consts::FRAC_PI_2, std::f32::consts::PI] {
                let pose = Pose { yaw, ..Pose::REST };
                let frame = layers(outline, &[], &look, &pose, size, Some((kind, material)));
                let image = frame.image.expect("rasterized body");
                assert!(
                    image.covers([full / 2.0, full / 2.0 + RISE * size], full),
                    "{shading:?} leaves a hole at yaw {yaw}"
                );
                assert!(!image.covers([0.5, 0.5], full));
                if yaw == 1.2 {
                    assert!(!frame.front.is_empty(), "the near eye is still visible");
                }
            }
        }
    }

    #[test]
    fn bands_stay_inside_the_polygon() {
        let square = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let shape = banded(
            Shape::new(),
            &square,
            [-10.0, 0.0],
            [20.0, 0.0],
            &[
                (0.0, Rgb::new(1.0, 0.0, 0.0)),
                (1.0, Rgb::new(0.0, 0.0, 1.0)),
            ],
            4,
        );
        assert!(!shape.is_empty());
    }
}
