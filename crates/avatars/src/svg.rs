//! SVG path data to polygons, for the body outlines.
//!
//! The upstream outlines are SVG path strings. Rune's [`rune::Path`] fills one
//! subpath at a time and has no arc command, so the port flattens each outline
//! once into the polygon soup the slice stack needs, then transforms those
//! points per frame.

/// One flattened subpath. `closed` subpaths repeat no point.
#[derive(Clone, Debug, PartialEq)]
pub struct Subpath {
    pub points: Vec<[f32; 2]>,
    pub closed: bool,
}

/// A flattened outline in the design box.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outline {
    pub subpaths: Vec<Subpath>,
}

/// The longest a flattened chord may stray from the curve it replaces, in
/// design units (the body is 100 tall).
const TOLERANCE: f32 = 0.3;

impl Outline {
    /// Every point, transformed by `f`, as fills ready for [`rune::Shape`].
    pub fn transformed(&self, f: impl Fn([f32; 2]) -> [f32; 2]) -> Outline {
        Outline {
            subpaths: self
                .subpaths
                .iter()
                .map(|sub| Subpath {
                    points: sub.points.iter().map(|p| f(*p)).collect(),
                    closed: sub.closed,
                })
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.subpaths.iter().all(|sub| sub.points.len() < 3)
    }

    /// Whether `p` falls inside any subpath (even-odd and nonzero agree for
    /// the stock outlines, which do not overlap).
    pub fn contains(&self, p: [f32; 2]) -> bool {
        self.subpaths
            .iter()
            .any(|sub| sub.points.len() >= 3 && point_in_polygon(p, &sub.points))
    }
}

pub fn point_in_polygon(p: [f32; 2], points: &[[f32; 2]]) -> bool {
    let mut inside = false;
    let mut j = points.len() - 1;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[j]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// A 2×3 affine, column-major like a canvas transform.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Affine {
    pub const IDENTITY: Affine = Affine {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub fn translate(x: f32, y: f32) -> Affine {
        Affine {
            e: x,
            f: y,
            ..Affine::IDENTITY
        }
    }

    pub fn scale(x: f32, y: f32) -> Affine {
        Affine {
            a: x,
            d: y,
            ..Affine::IDENTITY
        }
    }

    pub fn rotate(radians: f32) -> Affine {
        let (s, c) = radians.sin_cos();
        Affine {
            a: c,
            b: s,
            c: -s,
            d: c,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Applies `self` first, then `other`, the way a canvas chains
    /// `translate` and `scale`.
    pub fn then(self, other: Affine) -> Affine {
        Affine {
            a: other.a * self.a + other.c * self.b,
            b: other.b * self.a + other.d * self.b,
            c: other.a * self.c + other.c * self.d,
            d: other.b * self.c + other.d * self.d,
            e: other.a * self.e + other.c * self.f + other.e,
            f: other.b * self.e + other.d * self.f + other.f,
        }
    }

    pub fn apply(self, p: [f32; 2]) -> [f32; 2] {
        [
            self.a * p[0] + self.c * p[1] + self.e,
            self.b * p[0] + self.d * p[1] + self.f,
        ]
    }

    /// The inverse, or the identity for a degenerate transform.
    pub fn inverse(self) -> Affine {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-9 {
            return Affine::IDENTITY;
        }
        let (a, b, c, d) = (self.d / det, -self.b / det, -self.c / det, self.a / det);
        Affine {
            a,
            b,
            c,
            d,
            e: -(a * self.e + c * self.f),
            f: -(b * self.e + d * self.f),
        }
    }
}

/// Parses SVG path data into flattened subpaths.
pub fn parse(d: &str) -> Outline {
    let mut parser = Parser::new(d);
    parser.run()
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
    cursor: [f32; 2],
    start: [f32; 2],
    previous_control: Option<[f32; 2]>,
    previous_command: char,
    outline: Outline,
    current: Vec<[f32; 2]>,
}

#[derive(Clone, Copy, Debug)]
enum Token {
    Command(char),
    Number(f32),
}

impl Parser {
    fn new(d: &str) -> Parser {
        Parser {
            tokens: tokenize(d),
            at: 0,
            cursor: [0.0, 0.0],
            start: [0.0, 0.0],
            previous_control: None,
            previous_command: ' ',
            outline: Outline::default(),
            current: Vec::new(),
        }
    }

    fn run(&mut self) -> Outline {
        while let Some(token) = self.peek() {
            let command = match token {
                Token::Command(c) => {
                    self.at += 1;
                    c
                }
                // A repeated coordinate set repeats the last drawing command.
                Token::Number(_) => match self.previous_command {
                    'M' => 'L',
                    'm' => 'l',
                    c if c != ' ' && c != 'Z' && c != 'z' => c,
                    _ => {
                        self.at += 1;
                        continue;
                    }
                },
            };
            if !self.command(command) {
                break;
            }
            self.previous_command = command;
        }
        self.flush(false);
        Outline {
            subpaths: std::mem::take(&mut self.outline.subpaths)
                .into_iter()
                .map(|sub| Subpath {
                    points: simplify(&sub.points),
                    closed: sub.closed,
                })
                .filter(|sub| sub.points.len() >= 2)
                .collect(),
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.at).copied()
    }

    fn number(&mut self) -> f32 {
        match self.tokens.get(self.at).copied() {
            Some(Token::Number(v)) => {
                self.at += 1;
                v
            }
            _ => 0.0,
        }
    }

    fn pair(&mut self) -> [f32; 2] {
        let x = self.number();
        let y = self.number();
        [x, y]
    }

    fn flush(&mut self, closed: bool) {
        if self.current.len() >= 2 {
            let mut points = std::mem::take(&mut self.current);
            if closed {
                if let (Some(first), Some(last)) = (points.first().copied(), points.last().copied())
                {
                    if (first[0] - last[0]).abs() < 1e-4 && (first[1] - last[1]).abs() < 1e-4 {
                        points.pop();
                    }
                }
            }
            self.outline.subpaths.push(Subpath { points, closed });
        }
        self.current.clear();
    }

    /// Runs one command; `false` stops a malformed path.
    fn command(&mut self, command: char) -> bool {
        let relative = command.is_ascii_lowercase();
        let upper = command.to_ascii_uppercase();
        let to = |p: [f32; 2], cursor: [f32; 2]| {
            if relative {
                [p[0] + cursor[0], p[1] + cursor[1]]
            } else {
                p
            }
        };
        let mut control: Option<[f32; 2]> = None;
        match upper {
            'M' => {
                self.flush(false);
                let p = to(self.pair(), self.cursor);
                self.cursor = p;
                self.start = p;
                self.current.push(p);
            }
            'L' => {
                let p = to(self.pair(), self.cursor);
                self.line_to(p);
            }
            'H' => {
                let x = self.number();
                let p = if relative {
                    [self.cursor[0] + x, self.cursor[1]]
                } else {
                    [x, self.cursor[1]]
                };
                self.line_to(p);
            }
            'V' => {
                let y = self.number();
                let p = if relative {
                    [self.cursor[0], self.cursor[1] + y]
                } else {
                    [self.cursor[0], y]
                };
                self.line_to(p);
            }
            'C' => {
                let c1 = to(self.pair(), self.cursor);
                let c2 = to(self.pair(), self.cursor);
                let p = to(self.pair(), self.cursor);
                control = Some(c2);
                self.cubic_to(c1, c2, p);
            }
            'S' => {
                let c1 = match self.previous_control {
                    Some(prev)
                        if matches!(self.previous_command.to_ascii_uppercase(), 'C' | 'S') =>
                    {
                        [
                            2.0 * self.cursor[0] - prev[0],
                            2.0 * self.cursor[1] - prev[1],
                        ]
                    }
                    _ => self.cursor,
                };
                let c2 = to(self.pair(), self.cursor);
                let p = to(self.pair(), self.cursor);
                control = Some(c2);
                self.cubic_to(c1, c2, p);
            }
            'Q' => {
                let c = to(self.pair(), self.cursor);
                let p = to(self.pair(), self.cursor);
                control = Some(c);
                self.quad_to(c, p);
            }
            'T' => {
                let c = match self.previous_control {
                    Some(prev)
                        if matches!(self.previous_command.to_ascii_uppercase(), 'Q' | 'T') =>
                    {
                        [
                            2.0 * self.cursor[0] - prev[0],
                            2.0 * self.cursor[1] - prev[1],
                        ]
                    }
                    _ => self.cursor,
                };
                let p = to(self.pair(), self.cursor);
                control = Some(c);
                self.quad_to(c, p);
            }
            'A' => {
                let rx = self.number();
                let ry = self.number();
                let rotation = self.number();
                let large = self.number() != 0.0;
                let sweep = self.number() != 0.0;
                let p = to(self.pair(), self.cursor);
                self.arc_to(rx, ry, rotation, large, sweep, p);
            }
            'Z' => {
                let start = self.start;
                if self.current.len() >= 2 {
                    self.flush(true);
                }
                self.cursor = start;
                self.previous_control = None;
                self.previous_command = command;
                return true;
            }
            _ => return false,
        }
        self.previous_control = control;
        true
    }

    fn line_to(&mut self, p: [f32; 2]) {
        self.current.push(p);
        self.cursor = p;
    }

    fn quad_to(&mut self, c: [f32; 2], p: [f32; 2]) {
        let from = self.cursor;
        let c1 = [
            from[0] + 2.0 / 3.0 * (c[0] - from[0]),
            from[1] + 2.0 / 3.0 * (c[1] - from[1]),
        ];
        let c2 = [
            p[0] + 2.0 / 3.0 * (c[0] - p[0]),
            p[1] + 2.0 / 3.0 * (c[1] - p[1]),
        ];
        self.cubic_to(c1, c2, p);
    }

    fn cubic_to(&mut self, c1: [f32; 2], c2: [f32; 2], p: [f32; 2]) {
        let from = self.cursor;
        let mut out = Vec::new();
        flatten_cubic(from, c1, c2, p, 0, &mut out);
        for point in out {
            self.current.push(point);
        }
        self.cursor = p;
    }

    fn arc_to(&mut self, rx: f32, ry: f32, rotation: f32, large: bool, sweep: bool, p: [f32; 2]) {
        let from = self.cursor;
        let (mut rx, mut ry) = (rx.abs(), ry.abs());
        if rx < 1e-6 || ry < 1e-6 {
            self.line_to(p);
            return;
        }
        let phi = rotation.to_radians();
        let (sin_phi, cos_phi) = phi.sin_cos();
        let dx2 = (from[0] - p[0]) / 2.0;
        let dy2 = (from[1] - p[1]) / 2.0;
        let x1 = cos_phi * dx2 + sin_phi * dy2;
        let y1 = -sin_phi * dx2 + cos_phi * dy2;
        let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
        if lambda > 1.0 {
            let scale = lambda.sqrt();
            rx *= scale;
            ry *= scale;
        }
        let num = (rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1).max(0.0);
        let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
        let mut factor = if den <= 0.0 { 0.0 } else { (num / den).sqrt() };
        if large == sweep {
            factor = -factor;
        }
        let cx1 = factor * rx * y1 / ry;
        let cy1 = -factor * ry * x1 / rx;
        let cx = cos_phi * cx1 - sin_phi * cy1 + (from[0] + p[0]) / 2.0;
        let cy = sin_phi * cx1 + cos_phi * cy1 + (from[1] + p[1]) / 2.0;
        let angle =
            |ux: f32, uy: f32, vx: f32, vy: f32| (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
        let start = angle(1.0, 0.0, (x1 - cx1) / rx, (y1 - cy1) / ry);
        let mut delta = angle(
            (x1 - cx1) / rx,
            (y1 - cy1) / ry,
            (-x1 - cx1) / rx,
            (-y1 - cy1) / ry,
        );
        if !sweep && delta > 0.0 {
            delta -= std::f32::consts::TAU;
        } else if sweep && delta < 0.0 {
            delta += std::f32::consts::TAU;
        }
        // A circular arc uses fewer, wider cubics; an ellipse keeps 90° steps.
        let ratio = (rx / ry).max(ry / rx);
        let step = if ratio < 1.02 {
            std::f32::consts::FRAC_PI_2
        } else {
            std::f32::consts::FRAC_PI_4
        };
        let pieces = (delta.abs() / step).ceil().max(1.0) as usize;
        let each = delta / pieces as f32;
        let point = |t: f32| -> [f32; 2] {
            let (sin_t, cos_t) = t.sin_cos();
            [
                cx + cos_phi * rx * cos_t - sin_phi * ry * sin_t,
                cy + sin_phi * rx * cos_t + cos_phi * ry * sin_t,
            ]
        };
        for i in 1..=pieces {
            let t0 = start + each * (i - 1) as f32;
            let t1 = start + each * i as f32;
            let k = 4.0 / 3.0 * ((t1 - t0) / 4.0).tan();
            let derivative = |t: f32| -> [f32; 2] {
                let (sin_t, cos_t) = t.sin_cos();
                [
                    cos_phi * -rx * sin_t - sin_phi * ry * cos_t,
                    sin_phi * -rx * sin_t + cos_phi * ry * cos_t,
                ]
            };
            let p0 = point(t0);
            let p1 = point(t1);
            let d0 = derivative(t0);
            let d1 = derivative(t1);
            self.cubic_to(
                [p0[0] + k * d0[0], p0[1] + k * d0[1]],
                [p1[0] - k * d1[0], p1[1] - k * d1[1]],
                p1,
            );
        }
        self.cursor = p;
    }
}

/// Whether `e` at `i` starts an exponent (`e-3`, `e5`).
fn exponent_at(bytes: &[u8], i: usize) -> bool {
    match bytes.get(i + 1) {
        Some(c) if c.is_ascii_digit() => true,
        Some(b'-') | Some(b'+') => bytes.get(i + 2).is_some_and(u8::is_ascii_digit),
        _ => false,
    }
}

fn tokenize(d: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = d.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_whitespace() || c == ',' {
            i += 1;
            continue;
        }
        if c.is_ascii_alphabetic() {
            tokens.push(Token::Command(c));
            i += 1;
            continue;
        }
        let start = i;
        if c == '-' || c == '+' {
            i += 1;
        }
        let mut seen_dot = false;
        while i < bytes.len() {
            let c = bytes[i] as char;
            if c.is_ascii_digit() {
                i += 1;
            } else if c == '.' && !seen_dot {
                seen_dot = true;
                i += 1;
            } else if (c == 'e' || c == 'E') && exponent_at(bytes, i) {
                i += 2;
            } else {
                break;
            }
        }
        if let Ok(value) = d[start..i].parse::<f32>() {
            tokens.push(Token::Number(value));
        }
    }
    tokens
}

fn flatten_cubic(
    from: [f32; 2],
    c1: [f32; 2],
    c2: [f32; 2],
    to: [f32; 2],
    depth: u32,
    out: &mut Vec<[f32; 2]>,
) {
    if depth >= 12 || flat_enough(from, c1, c2, to) {
        out.push(to);
        return;
    }
    let mid = |a: [f32; 2], b: [f32; 2]| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    let ab = mid(from, c1);
    let bc = mid(c1, c2);
    let cd = mid(c2, to);
    let abc = mid(ab, bc);
    let bcd = mid(bc, cd);
    let centre = mid(abc, bcd);
    flatten_cubic(from, ab, abc, centre, depth + 1, out);
    flatten_cubic(centre, bcd, cd, to, depth + 1, out);
}

fn flat_enough(from: [f32; 2], c1: [f32; 2], c2: [f32; 2], to: [f32; 2]) -> bool {
    let chord = [to[0] - from[0], to[1] - from[1]];
    let length = (chord[0] * chord[0] + chord[1] * chord[1]).sqrt();
    let distance = |p: [f32; 2]| {
        if length < 1e-6 {
            ((p[0] - from[0]).powi(2) + (p[1] - from[1]).powi(2)).sqrt()
        } else {
            let cross = (p[0] - from[0]) * chord[1] - (p[1] - from[1]) * chord[0];
            (cross / length).abs()
        }
    };
    distance(c1) <= TOLERANCE && distance(c2) <= TOLERANCE
}

/// Ramer–Douglas–Peucker: drops points a polyline does not need.
fn simplify(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0, points.len() - 1)];
    while let Some((first, last)) = stack.pop() {
        let (a, b) = (points[first], points[last]);
        let mut worst = 0.0;
        let mut worst_at = first;
        for i in first + 1..last {
            let p = points[i];
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let length = (dx * dx + dy * dy).sqrt();
            let distance = if length < 1e-6 {
                ((p[0] - a[0]).powi(2) + (p[1] - a[1]).powi(2)).sqrt()
            } else {
                ((dx * (a[1] - p[1]) - (a[0] - p[0]) * dy) / length).abs()
            };
            if distance > worst {
                worst = distance;
                worst_at = i;
            }
        }
        if worst > TOLERANCE * 0.5 {
            keep[worst_at] = true;
            stack.push((first, worst_at));
            stack.push((worst_at, last));
        }
    }
    points
        .iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, p)| *p)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_flattens_to_a_polygon_near_the_radius() {
        let outline = parse("M50 8A42 42 0 1 1 50 92A42 42 0 1 1 50 8Z");
        assert_eq!(outline.subpaths.len(), 1);
        let points = &outline.subpaths[0].points;
        assert!(points.len() > 12, "got {} points", points.len());
        assert!(points.len() < 80, "too many points: {}", points.len());
        for p in points {
            let r = ((p[0] - 50.0).powi(2) + (p[1] - 50.0).powi(2)).sqrt();
            assert!((r - 42.0).abs() < 0.6, "point {p:?} radius {r}");
        }
        assert!(outline.contains([50.0, 50.0]));
        assert!(!outline.contains([95.0, 95.0]));
    }

    #[test]
    fn relative_commands_and_close() {
        let outline = parse("m10 10 h20 v20 h-20 z");
        assert_eq!(outline.subpaths.len(), 1);
        assert_eq!(
            outline.subpaths[0].points,
            vec![[10.0, 10.0], [30.0, 10.0], [30.0, 30.0], [10.0, 30.0]]
        );
        assert!(outline.subpaths[0].closed);
    }

    #[test]
    fn separate_subpaths_stay_separate() {
        let outline =
            parse("M4 62A7 7 0 1 1 18 62A7 7 0 1 1 4 62ZM82 62A7 7 0 1 1 96 62A7 7 0 1 1 82 62Z");
        assert_eq!(outline.subpaths.len(), 2);
    }

    #[test]
    fn affine_composes_like_a_canvas() {
        let m = Affine::translate(10.0, 0.0).then(Affine::scale(2.0, 2.0));
        assert_eq!(m.apply([1.0, 1.0]), [22.0, 2.0]);
    }
}
