use std::{
    io::Write,
    time::{Duration, SystemTime},
};

use anyhow::{Context, Result};
use crossterm::{cursor, queue, style, terminal};
use hex_color::HexColor;
use palette::RgbHue;
use palette::{FromColor, Hsl, Srgb};
use rand::{Rng, RngCore};
use rand_xoshiro::Xoshiro256PlusPlus;
use rand_xoshiro::rand_core::SeedableRng;

#[derive(Clone)]
struct Glyph {
    character: char,
    color: HexColor,
}

impl Glyph {
    fn new(character: char, color: HexColor) -> Self {
        Self { character, color }
    }

    fn new_random<R: Rng>(rand: &mut R, color: HexColor) -> Self {
        let characters = "ﾊﾐﾋｰｳｼﾅﾓﾆｻﾜﾂｵﾘｱﾎﾃﾏｹﾒｴｶｷﾑﾕﾗｾﾈｽﾀﾇﾍｦｲｸｺｿﾁﾄﾉﾌﾔﾖﾙﾚﾛﾝ¦*+-,.;";
        Self {
            character: characters
                .chars()
                .nth(rand.random_range(0..characters.chars().count()))
                .unwrap(),
            color,
        }
    }

    fn fade_color(&mut self) {
        let rgb = Srgb::new(
            self.color.r as f32 / 255.0,
            self.color.g as f32 / 255.0,
            self.color.b as f32 / 255.0,
        );

        let mut hsl: Hsl = Hsl::from_color(rgb);

        hsl.saturation = (hsl.saturation * 0.9).clamp(0.0, 1.0);
        hsl.lightness = (hsl.lightness * 0.93).clamp(0.0, 1.0);

        let new_rgb: Srgb = Srgb::from_color(hsl);

        let new_r = new_rgb.red * 255.0;
        let new_g = new_rgb.green * 255.0;
        let new_b = new_rgb.blue * 255.0;

        self.color = HexColor::rgb(new_r as u8, new_g as u8, new_b as u8);
    }
    fn empty() -> Self {
        Self {
            character: ' ',
            color: HexColor::rgb(0, 0, 0),
        }
    }

    fn render<W: Write>(&self, out: &mut W) -> Result<()> {
        queue!(
            out,
            style::SetBackgroundColor(style::Color::Rgb {
                r: (0),
                g: (0),
                b: (0)
            })
        )?;
        queue!(
            out,
            style::SetForegroundColor(style::Color::Rgb {
                r: self.color.r,
                g: self.color.g,
                b: self.color.b
            })
        )?;
        queue!(out, style::Print(self.character.to_string()))
            .context("write glyph to unicode chars")?;

        Ok(())
    }
}
#[derive(Clone)]
struct Column {
    height: u16,
    base_color: HexColor,
    glyphs: Vec<Glyph>,
    active_index: usize,
}

impl Column {
    fn new(height: u16, base_color: HexColor) -> Self {
        Self {
            height,
            base_color,
            glyphs: vec![Glyph::empty(); height as usize],
            active_index: 0,
        }
    }

    fn render<W: Write>(&self, out: &mut W, y: u16) -> Result<()> {
        self.glyphs[y as usize].render(out);
        Ok(())
    }

    fn step<R: Rng>(&mut self, rand: &mut R) {
        if self.active_index == 0 && rand.random::<f32>() > 0.1 {
            return;
        }

        for glyph in &mut self.glyphs {
            glyph.fade_color();
        }

        let base_color = HexColor::rgb(0, 150, 255);
        let base_color2 = HexColor::rgb(0, 255, 43);
        let chosen = choose_random(base_color, base_color2);

        // just put a single color here instead of randoming between blue and green :)

        self.glyphs[self.active_index] = Glyph::new_random(rand, chosen);
        self.active_index += 1;
        if self.active_index >= self.height as usize {
            self.active_index = 0;
        }
    }
}

struct MatrixWaterFall {
    width: u16,
    height: u16,
    base_color: HexColor,
    columns: Vec<Column>,
}
impl MatrixWaterFall {
    fn new(width: u16, height: u16, base_color: HexColor) -> Self {
        Self {
            width,
            height,
            base_color,
            /// TO DO Columns here
            columns: vec![Column::new(height, base_color); width as usize],
        }
    }

    fn render<W: Write>(&self, out: &mut W) -> Result<()> {
        queue!(out, cursor::Hide);
        queue!(out, cursor::MoveTo(0, 0));
        for y in 0..self.height {
            for column in &self.columns {
                column.render(out, y)?;
            }
        }
        queue!(out, style::ResetColor)?;
        // queue!(out, cursor::Show)?;
        out.flush().context("flush output")?;
        Ok(())
    }

    fn step<R: Rng>(&mut self, rand: &mut R) {
        for column in &mut self.columns {
            column.step(rand);
        }
    }
}

fn choose_random<T: Clone>(a: T, b: T) -> T {
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.5) { a } else { b }
}

fn main() -> Result<()> {
    let (width, height) = terminal::size().context("determine teminal size")?;
    /// default matrix green color hex code
    let base_color = HexColor::rgb(0, 150, 255);
    let base_color2 = HexColor::rgb(0, 255, 43);
    let chosen = choose_random(base_color, base_color2);

    let mut waterfall = MatrixWaterFall::new(width, height, chosen);
    let mut stdout = std::io::stdout();

    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time to have passed since unix epoch")
        .as_micros() as u64;
    let mut rand = Xoshiro256PlusPlus::seed_from_u64(seed);

    loop {
        waterfall.render(&mut stdout)?;
        waterfall.step(&mut rand);
        std::thread::sleep(Duration::from_millis(80));
    }
}
