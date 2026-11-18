//! Colour in sRGB, with the upstream port's HSL shading and OKLab vividness.
//!
//! Rune's [`Color`] is linear light, so every colour here is kept in sRGB and
//! converted at the edge. Ported from the upstream `bot-avatars` `color.ts`.

use rune::color::srgb_decode;
use rune::Color;

/// A colour in sRGB, channels in `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub const fn new(r: f32, g: f32, b: f32) -> Rgb {
        Rgb { r, g, b }
    }

    /// `0xrrggbb`.
    pub fn from_hex(value: u32) -> Rgb {
        let channel = |shift: u32| ((value >> shift) & 0xff) as f32 / 255.0;
        Rgb::new(channel(16), channel(8), channel(0))
    }

    /// `#rgb`, `#rrggbb`, `rgb(...)` or `hsl(...)`.
    pub fn from_css(input: &str) -> Option<Rgb> {
        let s = input.trim();
        if let Some(rest) = s.strip_prefix('#') {
            let h = rest.trim();
            if h.len() == 3 || h.len() == 6 {
                let mut digits = String::with_capacity(6);
                if h.len() == 3 {
                    for c in h.chars() {
                        digits.push(c);
                        digits.push(c);
                    }
                } else {
                    digits.push_str(h);
                }
                let value = u32::from_str_radix(&digits, 16).ok()?;
                return Some(Rgb::from_hex(value));
            }
            return None;
        }
        let nums = |s: &str| -> Vec<f32> {
            s.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
                .filter(|part| !part.is_empty())
                .filter_map(|part| part.parse::<f32>().ok())
                .collect()
        };
        let lower = s.to_ascii_lowercase();
        if lower.starts_with("hsl") {
            let n = nums(&lower);
            if n.len() >= 3 {
                return Some(Rgb::from_hsl([n[0] / 360.0, n[1] / 100.0, n[2] / 100.0]));
            }
        }
        if lower.starts_with("rgb") {
            let n = nums(&lower);
            if n.len() >= 3 {
                return Some(Rgb::new(n[0] / 255.0, n[1] / 255.0, n[2] / 255.0));
            }
        }
        None
    }

    /// Rune's linear-light colour for this sRGB value.
    pub fn color(self) -> Color {
        Color {
            r: srgb_decode(self.r),
            g: srgb_decode(self.g),
            b: srgb_decode(self.b),
            a: 1.0,
        }
    }

    /// Rune's colour at an opacity.
    pub fn alpha(self, a: f32) -> Color {
        Color { a, ..self.color() }
    }

    /// WCAG relative luminance, 0–1.
    pub fn luminance(self) -> f32 {
        let lin = |v: f32| {
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(self.r) + 0.7152 * lin(self.g) + 0.0722 * lin(self.b)
    }

    /// `t` of the way to `other`, straight in sRGB.
    pub fn mix(self, other: Rgb, t: f32) -> Rgb {
        let m = |a: f32, b: f32| a + (b - a) * t;
        Rgb::new(m(self.r, other.r), m(self.g, other.g), m(self.b, other.b))
    }

    pub fn to_hsl(self) -> [f32; 3] {
        let (r, g, b) = (self.r, self.g, self.b);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if max == min {
            return [0.0, 0.0, l];
        }
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
        let h = if max == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        [h / 6.0, s, l]
    }

    pub fn from_hsl([h, s, l]: [f32; 3]) -> Rgb {
        if s <= 0.0 {
            return Rgb::new(l, l, l);
        }
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        let f = |t: f32| {
            let t = (t.rem_euclid(1.0) + 1.0).rem_euclid(1.0);
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        };
        Rgb::new(f(h + 1.0 / 3.0), f(h), f(h - 1.0 / 3.0))
    }

    /// A shade of this colour: `dl` moves the lightness, `ds` the saturation.
    /// Darker shades gain a little saturation so they stay rich rather than
    /// going grey, the way a painted surface falls into shadow.
    pub fn shade(self, dl: f32, ds: f32) -> Rgb {
        let [h, s, l] = self.to_hsl();
        let s = (s + ds + if dl < 0.0 { -dl * 0.25 } else { 0.0 }).clamp(0.0, 1.0);
        Rgb::from_hsl([h, s, (l + dl).clamp(0.0, 1.0)])
    }

    /// A more vivid version, `t` 0–1: chroma raised along its own hue by up to
    /// half again, deepening by at most 15% where the screen cannot show that
    /// chroma at the colour's own lightness.
    pub fn richer(self, t: f32) -> Rgb {
        if !(t > 0.0) {
            return self;
        }
        let to_linear = |u: f32| {
            if u <= 0.04045 {
                u / 12.92
            } else {
                ((u + 0.055) / 1.055).powf(2.4)
            }
        };
        let to_srgb = |u: f32| {
            if u <= 0.003_130_8 {
                12.92 * u
            } else {
                1.055 * u.powf(1.0 / 2.4) - 0.055
            }
        };
        let oklab = |r: f32, g: f32, b: f32| -> [f32; 3] {
            let l = (0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_993 * b).cbrt();
            let m = (0.211_903_5 * r + 0.680_699_55 * g + 0.107_396_956 * b).cbrt();
            let s = (0.088_302_46 * r + 0.281_718_84 * g + 0.629_978_7 * b).cbrt();
            [
                0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
                1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
                0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
            ]
        };
        let from_oklab = |l_b: f32, a: f32, b: f32| -> [f32; 3] {
            let l = (l_b + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
            let m = (l_b - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
            let s = (l_b - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
            [
                4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_93 * s,
                -1.268_438 * l + 2.609_757_4 * m - 0.341_319_4 * s,
                -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
            ]
        };
        let in_gamut = |c: [f32; 3]| c.iter().all(|v| (-1e-4..=1.0 + 1e-4).contains(v));

        let lab = oklab(to_linear(self.r), to_linear(self.g), to_linear(self.b));
        let [lightness, a, b] = lab;
        let chroma = (a * a + b * b).sqrt();
        if chroma < 1e-4 {
            return self;
        }
        let (ua, ub) = (a / chroma, b / chroma);
        let want = chroma * (1.0 + 0.5 * t);
        let mut best: Option<[f32; 3]> = None;
        let mut best_c = chroma;
        for k in 0..=6 {
            let lk = lightness * (1.0 - (0.15 * t * k as f32) / 6.0);
            let (mut lo, mut hi) = (0.0_f32, want);
            for _ in 0..18 {
                let mid = (lo + hi) / 2.0;
                if in_gamut(from_oklab(lk, ua * mid, ub * mid)) {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            if lo > best_c + 0.004 * k as f32 {
                best_c = lo;
                best = Some(from_oklab(lk, ua * lo, ub * lo));
            }
            if lo >= want - 1e-4 {
                break;
            }
        }
        match best {
            None => self,
            Some(out) => Rgb::new(
                to_srgb(out[0].clamp(0.0, 1.0)),
                to_srgb(out[1].clamp(0.0, 1.0)),
                to_srgb(out[2].clamp(0.0, 1.0)),
            ),
        }
    }
}

/// The ink a face takes on a body: dark, or light on a dark body.
pub const DARK_INK: Rgb = Rgb::new(
    0x1e as f32 / 255.0,
    0x1a as f32 / 255.0,
    0x33 as f32 / 255.0,
);
pub const LIGHT_INK: Rgb = Rgb::new(
    0xf7 as f32 / 255.0,
    0xf5 as f32 / 255.0,
    0xf2 as f32 / 255.0,
);

pub fn auto_ink(body: Rgb) -> Rgb {
    if body.luminance() < 0.13 {
        LIGHT_INK
    } else {
        DARK_INK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Rgb, b: Rgb) -> bool {
        (a.r - b.r).abs() < 0.01 && (a.g - b.g).abs() < 0.01 && (a.b - b.b).abs() < 0.01
    }

    #[test]
    fn hex_and_css_parse() {
        assert!(close(
            Rgb::from_css("#41C4FF").unwrap(),
            Rgb::from_hex(0x41c4ff)
        ));
        assert!(close(
            Rgb::from_css("#fff").unwrap(),
            Rgb::new(1.0, 1.0, 1.0)
        ));
        assert!(close(
            Rgb::from_css("rgb(65 196 255)").unwrap(),
            Rgb::from_hex(0x41c4ff)
        ));
        assert!(Rgb::from_css("tomato").is_none());
    }

    #[test]
    fn shading_keeps_hue_and_moves_lightness() {
        let base = Rgb::from_hex(0x41c4ff);
        let dark = base.shade(-0.3, 0.05);
        assert!(dark.luminance() < base.luminance());
        let [hue, _, _] = dark.to_hsl();
        let [base_hue, _, _] = base.to_hsl();
        assert!((hue - base_hue).abs() < 0.01);
    }

    #[test]
    fn ink_flips_on_dark_bodies() {
        assert_eq!(auto_ink(Rgb::from_hex(0x111111)), LIGHT_INK);
        assert_eq!(auto_ink(Rgb::from_hex(0x41c4ff)), DARK_INK);
    }

    #[test]
    fn richer_raises_chroma_without_leaving_gamut() {
        let base = Rgb::from_hex(0x41c4ff);
        let vivid = base.richer(0.9);
        for v in [vivid.r, vivid.g, vivid.b] {
            assert!((0.0..=1.0).contains(&v), "channel {v} out of gamut");
        }
        assert_ne!(vivid, base);
    }
}
