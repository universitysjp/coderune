//! Soft body forms and pose-dependent materials. The cached mesh gives both
//! surfaces the same silhouette and carries the pile through turns and flips.

use crate::render::{Look, Projection};
use crate::svg::{point_in_polygon, Outline};
use crate::{body_outline, body_parts, BotAvatarType, Rgb};
use std::sync::OnceLock;

#[derive(Clone, Copy)]
pub(crate) enum Material {
    Plastic,
    Fabric,
}

const GRID: usize = 43;
const STEP: f32 = 3.0;
const START: f32 = -13.0;
const GRID_AREA: usize = GRID * GRID;
const MAX_IMAGE_SIDE: f32 = 384.0;

static FORMS: [OnceLock<Vec<Form>>; BotAvatarType::ALL.len()] =
    [const { OnceLock::new() }; BotAvatarType::ALL.len()];

struct Form {
    vertices: Vec<Vertex>,
    triangles: Vec<[usize; 3]>,
    depth_scale: f32,
}

struct Vertex {
    x: f32,
    y: f32,
    distance: f32,
    height: f32,
    slope: [f32; 2],
}

impl Form {
    fn bake(outline: &Outline, depth_scale: f32) -> Self {
        let mut vertices = Vec::with_capacity(GRID_AREA);
        for y in 0..GRID {
            for x in 0..GRID {
                let point = [START + x as f32 * STEP, START + y as f32 * STEP];
                vertices.push(Vertex {
                    x: point[0] - 50.0,
                    y: point[1] - 50.0,
                    distance: signed_distance(outline, point),
                    height: 0.0,
                    slope: [0.0; 2],
                });
            }
        }

        // Solve the Poisson equation with zero height at the outline. Taking
        // the square root rounds the lobes without creases along their axes.
        let mut potential = vec![0.0_f32; GRID_AREA];
        for _ in 0..96 {
            for y in 1..GRID - 1 {
                for x in 1..GRID - 1 {
                    let i = y * GRID + x;
                    if vertices[i].distance > 0.0 {
                        potential[i] = (1.0
                            + potential[i - 1]
                            + potential[i + 1]
                            + potential[i - GRID]
                            + potential[i + GRID])
                            * 0.25;
                    }
                }
            }
        }

        let peak = potential.iter().copied().fold(1.0_f32, f32::max);
        let heights: Vec<_> = potential
            .iter()
            .zip(&vertices)
            .map(|(&v, node)| {
                // Interpolation meets small and concave boundaries gently.
                (v / peak).sqrt() * (node.distance.max(0.0) / STEP).min(1.0).sqrt()
            })
            .collect();
        for y in 1..GRID - 1 {
            for x in 1..GRID - 1 {
                let i = y * GRID + x;
                vertices[i].height = heights[i];
                vertices[i].slope = [
                    (heights[i + 1] - heights[i - 1]) / (2.0 * STEP),
                    (heights[i + GRID] - heights[i - GRID]) / (2.0 * STEP),
                ];
            }
        }

        let mut triangles = Vec::new();
        let mut used = [false; GRID_AREA];
        for y in 0..GRID - 1 {
            for x in 0..GRID - 1 {
                let i = y * GRID + x;
                for triangle in [[i, i + 1, i + GRID], [i + 1, i + GRID + 1, i + GRID]] {
                    if triangle
                        .iter()
                        .any(|&index| vertices[index].distance > -2.5)
                    {
                        for index in triangle {
                            used[index] = true;
                        }
                        triangles.push(triangle);
                    }
                }
            }
        }
        // Keep only vertices referenced by a triangle in the material.
        let mut indices = [0; GRID_AREA];
        let mut mesh = Vec::new();
        for (i, vertex) in vertices.into_iter().enumerate() {
            if used[i] {
                indices[i] = mesh.len();
                mesh.push(vertex);
            }
        }
        for triangle in &mut triangles {
            *triangle = triangle.map(|i| indices[i]);
        }
        Self {
            vertices: mesh,
            triangles,
            depth_scale,
        }
    }
}

fn signed_distance(outline: &Outline, point: [f32; 2]) -> f32 {
    // The maximum of the signed distances treats overlapping subpaths as a
    // union. Taking the distance to their internal edges would cut seams.
    outline
        .subpaths
        .iter()
        .map(|subpath| subpath.points.as_slice())
        .filter(|points| points.len() >= 3)
        .map(|points| {
            let distance = points
                .iter()
                .copied()
                .zip(points.iter().copied().cycle().skip(1))
                .map(|(a, b)| segment_distance_squared(point, a, b))
                .fold(f32::INFINITY, f32::min)
                .sqrt();
            if point_in_polygon(point, points) {
                distance
            } else {
                -distance
            }
        })
        .fold(f32::NEG_INFINITY, f32::max)
}

fn segment_distance_squared(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (x, y) = (b[0] - a[0], b[1] - a[1]);
    let (px, py) = (p[0] - a[0], p[1] - a[1]);
    let t = ((px * x + py * y) / (x * x + y * y).max(f32::EPSILON)).clamp(0.0, 1.0);
    (px - t * x).powi(2) + (py - t * y).powi(2)
}

struct Light {
    key: [f32; 3],
    halfway: [f32; 3],
    body: Rgb,
    shadow: f32,
    highlight: f32,
    rim: f32,
}

impl Light {
    fn new(look: &Look) -> Self {
        let (sin, cos) = look.light.to_radians().sin_cos();
        let key = normalize([sin, -cos, 0.85]);
        Self {
            key,
            halfway: normalize([key[0], key[1], 1.0 + key[2]]),
            body: look.body,
            shadow: look.shadow,
            highlight: look.highlight,
            rim: look.rim,
        }
    }

    fn shade(&self, normal: [f32; 3], material: Material) -> Rgb {
        let lambert = ((dot(normal, self.key) + 0.35) / 1.35).clamp(0.0, 1.0);
        let grazing = (1.0 - normal[2].abs()).powi(3);
        let shade = 1.0 - (0.38 + 0.26 * self.shadow) * (1.0 - lambert);
        let base = Rgb::new(
            self.body.r * shade,
            self.body.g * shade,
            self.body.b * shade,
        );
        let tint = Rgb::new(0.92, 0.97, 1.0);
        let glint = match material {
            Material::Plastic => {
                let specular = dot(normal, self.halfway).max(0.0);
                self.highlight * (0.48 * specular.powi(36) + 0.18 * specular.powi(8))
                    + self.rim * 0.22 * grazing
            }
            Material::Fabric => 0.06 * self.highlight * lambert + self.rim * 0.27 * grazing,
        };
        base.mix(tint, glint.clamp(0.0, 0.88))
    }
}

#[derive(Clone, Copy)]
struct Projected {
    screen: [f32; 3],
    distance: f32,
    fibre: [f32; 2],
    color: Rgb,
}

impl Projected {
    fn new(
        vertex: &Vertex,
        projection: &Projection,
        depth: f32,
        side: f32,
        scale: f32,
        light: &Light,
        material: Material,
    ) -> Self {
        let z = vertex.height * depth * side;
        let [x, y, screen_depth] = projection.point([vertex.x, vertex.y, z]);
        let [dx, dy] = vertex.slope.map(|s| s * depth);
        // Normals use the surface's stock thickness, before projection.
        let n = normalize(projection.normal([-dx, -dy, side]));
        Self {
            screen: [x * scale, y * scale, screen_depth],
            distance: vertex.distance,
            fibre: [vertex.x + z * 0.54, vertex.y - z * 0.31],
            color: light.shade(n, material),
        }
    }
}

struct Raster {
    size: usize,
    rgba: Vec<u8>,
    depth: Vec<f32>,
    feather: f32,
}

impl Raster {
    fn new(size: usize, feather: f32) -> Self {
        Self {
            size,
            rgba: vec![0; size * size * 4],
            depth: vec![f32::NEG_INFINITY; size * size],
            feather,
        }
    }

    fn draw(&mut self, [a, b, c]: [Projected; 3], material: Material, side: f32) {
        let signed_area = edge(a.screen, b.screen, c.screen);
        if signed_area * side < 1e-5 {
            return;
        }
        // Convert one pixel of screen coverage to the outline's distance
        // units. On a sideways turn the two surfaces meet at the middle of
        // the projected body, where a fixed feather would cut a seam.
        let ab = b.distance - a.distance;
        let ac = c.distance - a.distance;
        let dx =
            (ab * (c.screen[1] - a.screen[1]) - ac * (b.screen[1] - a.screen[1])) / signed_area;
        let dy =
            (ac * (b.screen[0] - a.screen[0]) - ab * (c.screen[0] - a.screen[0])) / signed_area;
        let feather = (dx.abs() + dy.abs()).clamp(0.01, 2.0 * self.feather);
        let bounds = |axis: usize| {
            let low = a.screen[axis].min(b.screen[axis]).min(c.screen[axis]);
            let high = a.screen[axis].max(b.screen[axis]).max(c.screen[axis]);
            (low.floor().max(0.0) as usize)..(high.ceil().min(self.size as f32) as usize)
        };
        let barycentric = |p| {
            let u = edge(b.screen, c.screen, p) / signed_area;
            let v = edge(c.screen, a.screen, p) / signed_area;
            [u, v, 1.0 - u - v]
        };
        let columns = bounds(0);
        for y in bounds(1) {
            for x in columns.clone() {
                let [u, v, w] = barycentric([x as f32 + 0.5, y as f32 + 0.5, 0.0]);
                if u.min(v).min(w) < -1e-5 {
                    continue;
                }
                let mix = |a, b, c| u * a + v * b + w * c;
                let z = mix(a.screen[2], b.screen[2], c.screen[2]);
                let pixel = y * self.size + x;
                if z < self.depth[pixel] {
                    continue;
                }
                let distance = mix(a.distance, b.distance, c.distance);
                let (edge_offset, tone) = match material {
                    Material::Plastic => (0.0, 1.0),
                    Material::Fabric => {
                        let fx = mix(a.fibre[0], b.fibre[0], c.fibre[0]);
                        let fy = mix(a.fibre[1], b.fibre[1], c.fibre[1]);
                        let tuft = noise(fx * 1.35, fy * 0.68);
                        let thread = noise(fx * 3.6 + 0.4 * tuft, fy * 1.1);
                        (
                            0.8 + 1.7 * (thread - 0.5),
                            0.74 + 0.19 * tuft + 0.21 * thread,
                        )
                    }
                };
                let alpha = (0.5 + (distance + edge_offset) / feather).clamp(0.0, 1.0);
                if alpha <= 0.0 {
                    continue;
                }
                self.depth[pixel] = z;
                let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
                self.rgba[pixel * 4..pixel * 4 + 4].copy_from_slice(&[
                    byte(tone * mix(a.color.r, b.color.r, c.color.r)),
                    byte(tone * mix(a.color.g, b.color.g, c.color.g)),
                    byte(tone * mix(a.color.b, b.color.b, c.color.b)),
                    byte(alpha),
                ]);
            }
        }
    }
}

fn edge(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> f32 {
    (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = dot(v, v).sqrt();
    v.map(|c| c / len)
}

fn noise(x: f32, y: f32) -> f32 {
    let hash = |x: i32, y: i32| {
        let n = (x as u32).wrapping_mul(0x9e37_79b1) ^ (y as u32).wrapping_mul(0x85eb_ca77);
        ((n ^ (n >> 16)).wrapping_mul(0x7feb_352d) >> 24) as f32 / 255.0
    };
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (u, v) = (x - ix as f32, y - iy as f32);
    let lerp = |a, b, t| a + (b - a) * t;
    lerp(
        lerp(hash(ix, iy), hash(ix + 1, iy), u),
        lerp(hash(ix, iy + 1), hash(ix + 1, iy + 1), u),
        v,
    )
}

pub(crate) struct BodyImage {
    pub size: u32,
    pub rgba: Vec<u8>,
}

impl BodyImage {
    pub fn covers(&self, point: [f32; 2], full: f32) -> bool {
        let [x, y] = point.map(|coordinate| coordinate * self.size as f32 / full);
        if !(0.0..self.size as f32).contains(&x) || !(0.0..self.size as f32).contains(&y) {
            return false;
        }
        let offset = (y as usize * self.size as usize + x as usize) * 4;
        self.rgba[offset + 3] >= 192
    }
}

pub(crate) fn draw(
    kind: BotAvatarType,
    material: Material,
    projection: &Projection,
    look: &Look,
    full: f32,
) -> BodyImage {
    let image_side = (full * 2.0).ceil().clamp(1.0, MAX_IMAGE_SIDE) as usize;
    let scale = image_side as f32 / full;
    let feather = (100.0 * crate::render::OVERSCAN / image_side as f32).clamp(0.6, 3.0);
    let mut raster = Raster::new(image_side, feather);
    let light = Light::new(look);
    let forms = FORMS[kind as usize].get_or_init(|| {
        body_parts(kind)
            .iter()
            .map(|outline| Form::bake(outline, 0.4))
            .chain(std::iter::once(Form::bake(body_outline(kind), 1.0)))
            .collect()
    });

    for form in forms {
        let depth = 24.0 * look.depth * form.depth_scale;
        let mut vertices = Vec::with_capacity(form.vertices.len());
        for side in [-1.0, 1.0] {
            vertices.clear();
            vertices.extend(form.vertices.iter().map(|vertex| {
                Projected::new(vertex, projection, depth, side, scale, &light, material)
            }));
            for triangle in &form.triangles {
                raster.draw(triangle.map(|index| vertices[index]), material, side);
            }
        }
    }
    BodyImage {
        size: image_side as u32,
        rgba: raster.rgba,
    }
}
