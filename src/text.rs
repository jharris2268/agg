
use crate::VertexSource;
use crate::paths::Vertex;
use crate::base::RenderingBase;
use crate::color::Rgba8;
use crate::Pixel;
use crate::ft;

#[derive(Debug,PartialEq)]
enum TextStatus {
    Initial,
    NextChar,
    StartGlyph,
    Glyph,
}
#[derive(Debug,Default)]
pub struct GsvText {
    x: f64,
    y: f64,
    start_x: f64,
    width: f64,
    height: f64,
    space: f64,
    line_space: f64,
    //chr: [u8;2],
    text: String,
    //text_buf: String,
    font: Vec<u8>,
    //loaded_font: Vec<u8>,
    //status: TextStatus,
    big_endian: bool,
    flip: bool,
    //indices: Vec<u8>,
    //glyphs: Vec<i8>,
    //bglyphs: Vec<i8>,
    //eglyphs: Vec<i8>,
    //w: f64,
    //h: f64,
}

impl GsvText {
    pub fn new() -> Self {
        Self {
            x: 0.0, y: 0.0, start_x: 0.0, width: 10.0, height: 0.0,
            space: 0.0, line_space: 0.0, font: GsvDefaultFont::get().to_vec(),
            //status: TextStatus::Initial,
            flip: false, big_endian: false,
            //chr: [0_u8;2],
            text: String::new(),
            //w: 1.0, h: 1.0,
        }
    }
    pub fn size(&mut self, height: f64, width: f64) {
        self.height = height;
        self.width = width;
    }
    pub fn space(&mut self, space: f64) {
        self.space = space;
    }
    pub fn line_space(&mut self, line_space: f64) {
        self.line_space = line_space;
    }
    pub fn start_point(&mut self, x: f64, y: f64) {
        self.x = x;
        self.start_x = x;
        self.y = y;
    }
    pub fn flip(&mut self, flip: bool) {
        self.flip = flip;
    }
    pub fn text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}

impl VertexSource for GsvText {
    fn rewind(&self) {
    }
    fn xconvert(&self) -> Vec<Vertex<f64>> {
        let mut out = vec![];
        let mut chars = self.text.char_indices();
        let mut b : usize = 0;
        let mut e : usize = 0;
        let mut x = self.start_x;
        let mut y = self.y;
        let indices = value(&self.font[0..]) as usize;
        let glyphs = indices + 257*2;

        let mut status = TextStatus::Initial;
        let base_height : f64 = f64::from(value(&self.font[4..]));
        let mut hi = self.height / base_height;
        let wi = if self.width == 0.0 {
            hi
        } else {
            self.width / base_height
        };
        if self.flip {
            hi *= -1.0;
        }

        loop {
            match status {
                TextStatus::Initial => {
                    status = TextStatus::NextChar;
                },
                TextStatus::NextChar => {
                    match chars.next() {
                        None => break,
                        Some((_,chr)) => {
                            if chr == '\n' {
                                x = self.start_x;
                                y -= if self.flip {
                                    -(self.height + self.line_space)
                                } else {
                                    self.height + self.line_space
                                };
                            }
                            let mut idx = chr as usize & 0xFF;
                            idx *= 2;
                            b = glyphs + value(&self.font[indices+idx..]) as usize;
                            e = glyphs + value(&self.font[indices+idx+2..]) as usize;
                            status = TextStatus::StartGlyph;
                        }
                    }
                },
                TextStatus::StartGlyph => {
                    out.push(Vertex::move_to(x, y));
                    status = TextStatus::Glyph;
                },
                TextStatus::Glyph => {
                    for i in (b..e).step_by(2) {
                        let dx = i32::from(self.font[i] as i8);
                        let mut yc = self.font[i+1] as i8;
                        let yf = (self.font[i+1] & 0x80) as i8;
                        yc <<= 1;
                        yc >>= 1;
                        let dy = i32::from(yc);
                        x += wi * f64::from(dx);
                        y += hi * f64::from(dy);
                        if yf != 0 {
                            out.push(Vertex::move_to(x, y));
                        } else {
                            out.push(Vertex::line_to(x, y));
                        }
                    }
                    status = TextStatus::NextChar;
                }
            }
        }
        out
    }
}


fn value(v: &[u8]) -> i16 {
    unsafe { std::mem::transmute::<[u8;2],i16>([v[0],v[1]]) }
}

pub struct GsvDefaultFont();

impl GsvDefaultFont {
    pub fn get() -> &'static [u8] {
        &GSV_DEFAULT_FONT_DATA
    }
}



fn string_width(txt: &str, font: &ft::Face) -> f64 {
    let mut width = 0.0;
    for c in txt.chars() {
        let glyph_index = font.get_char_index(c as usize);
        font.load_glyph(glyph_index, ft::face::LoadFlag::DEFAULT).unwrap();
        let glyph = font.glyph();
        glyph.render_glyph(ft::RenderMode::Normal).unwrap();
        let adv = glyph.advance();
        width += adv.x as f64
    }
    width / 64.0
}

pub fn line_height(font: &ft::Face) -> f64 {
    let met = font.size_metrics().unwrap();
    (met.ascender - met.descender) as f64 / 64.0
}

pub fn draw_text<T>(txt: &str, x: i64, y: i64, font: &ft::Face, ren_base: &mut RenderingBase<T>)
    where T: Pixel
{
    let color = Rgba8::new(0,0,0,255);
    let (mut x, mut y) = (x,y);
    let width  = string_width(txt, font);
    let height = line_height(font);
    // Shift to center justification, x and y
    let dx = (width / 2.0).round() as i64;
    let dy = (height / 2.0).round() as i64;
    x -= dx;
    y += dy;
    for c in txt.chars() {
        let glyph_index = font.get_char_index(c as usize);
        font.load_glyph(glyph_index, ft::face::LoadFlag::DEFAULT).unwrap();
        font.glyph().render_glyph(ft::RenderMode::Normal).unwrap();
        let g = font.glyph().bitmap();
        let left = font.glyph().bitmap_left() as i64;
        let top  = font.glyph().bitmap_top() as i64;
        let buf : Vec<_> = g.buffer().iter().map(|&x| x as u64).collect();
        let rows = g.rows() as i64;
        let pitch = g.pitch().abs() as usize;
        let width = g.width() as i64;
        for i in 0 .. rows {
            ren_base.blend_solid_hspan(x + left, y-top+i, width,
                                       color, &buf[pitch*i as usize..]);
        }
        let adv = font.glyph().advance();
        x += (adv.x as f64 / 64.0).round() as i64;
        y += (adv.y as f64 / 64.0).round() as i64;
    }
}


#[derive(Debug)]
pub enum AggFontError {
    /// Freetype Error
    Ft(ft::error::Error),
    Io(String)
}

impl From<ft::error::Error> for AggFontError {
    fn from(err: ft::error::Error) -> Self {
        AggFontError::Ft(err)
    }
}
impl From<String> for AggFontError {
    fn from(err: String) -> Self {
        AggFontError::Io(err)
    }
}

pub fn font(name: &str) -> Result<ft::Face, AggFontError> {
    //let prop = font_loader::system_fonts::FontPropertyBuilder::new().family(name).build();
    //let (font, _) = font_loader::system_fonts::get(&prop).ok_or("error loading font".to_string())?;
    //let lib = ft::Library::init()?;
    //let face = lib.new_memory_face(font, 0)?;
    //Ok(face)
    Err(AggFontError::Io("??".to_string()))
}


#[derive(Debug,Copy,Clone,PartialEq)]
pub enum XAlign {
    Left, Center, Right
}
#[derive(Debug,Copy,Clone,PartialEq)]
pub enum YAlign {
    Top, Center, Bottom
}

pub struct Label<'a> {
    txt: String,
    x: f64,
    y: f64,
    xa: XAlign,
    ya: YAlign,
    color: Rgba8,
    font: &'a ft::Face,
    size: f64,
}

impl<'a> Label<'a> {
    pub fn new(txt: &str, x: f64, y: f64, size: f64, font: &'a ft::Face) -> Result<Self,AggFontError> {
        let resolution = 72;
        font.set_char_size((size * 64.0) as isize, 0, resolution, 0)?;
        Ok(
            Self {
                txt: txt.to_string(), x, y,
                xa: XAlign::Left,
                ya: YAlign::Bottom,
                color: Rgba8::black(),
                size,
                font
            }
        )
    }
    pub fn size(&self) -> (f64, f64) {
        let w = string_width(&self.txt, self.font);
        let h = line_height(self.font);
        (w, h)
    }
    pub fn xalign(mut self, xalign: XAlign) -> Self {
        self.xa = xalign;
        self
    }
    pub fn yalign(mut self, yalign: YAlign) -> Self {
        self.ya = yalign;
        self
    }
    pub fn color(mut self, color: Rgba8) -> Self {
        self.color = color;
        self
    }
    pub fn draw<T>(&mut self, ren: &mut RenderingBase<T>)
        where T: Pixel
    {
        draw_text_subpixel(&self.txt, self.x, self.y,
                           self.xa, self.ya, self.color,
                           self.font, ren);
    }
}

// https://www.freetype.org/freetype2/docs/glyphs/glyphs-5.html
// 2. Subpixel positioning
fn draw_text_subpixel<T>(txt: &str, x: f64, y: f64,
                         xalign: XAlign,
                         yalign: YAlign,
                         color: Rgba8,
                         font: &ft::Face,
                         ren_base: &mut RenderingBase<T>)
    where T: Pixel
{
    let (mut x, mut y) = (x,y);
    let width  = string_width(txt, font);

    let asc = font.size_metrics().unwrap().ascender as f64 / 64.0;
    x += match xalign {
        XAlign::Left => 0.0,
        XAlign::Right => -width,
        XAlign::Center => -width/2.0,
    };
    y += match yalign {
        YAlign::Top => asc,
        YAlign::Bottom => 0.0,
        YAlign::Center => asc / 2.0,
    };

    for c in txt.chars() {
        let glyph_index = font.get_char_index(c as usize);
        font.load_glyph(glyph_index, ft::face::LoadFlag::DEFAULT).unwrap();

        let glyph = font.glyph().get_glyph().unwrap();
        let dt = ft::Vector {
            x: ((x - x.floor()) * 64.0).round() as i64,
            y: ((y - y.floor()) * 64.0).round() as i64
        };
        glyph.transform(None, Some(dt)).unwrap();
        let g = glyph.to_bitmap(ft::RenderMode::Normal, None).unwrap();
        let left = g.left() as i64;
        let top  = g.top() as i64;
        let bit  = g.bitmap();
        let buf : Vec<_> = bit.buffer().iter().map(|&x| x as u64).collect();
        let rows  = bit.rows() as i64;
        let width = bit.width() as i64;
        let pitch = bit.pitch().abs() as usize;
        for i in 0 .. rows {
            ren_base.blend_solid_hspan(x.floor() as i64 + left,
                                       y.floor() as i64 + i - top,
                                       width,
                                       color, &buf[pitch*i as usize..]);
        }

        x += glyph.advance_x() as f64 / 65536.0;
        y += glyph.advance_y() as f64 / 65536.0;
    }
}


const GSV_DEFAULT_FONT_DATA: [u8; 4526] = [
        0x40,0x00,0x6c,0x0f,0x15,0x00,0x0e,0x00,0xf9,0xff,
        0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x0d,0x0a,0x0d,0x0a,0x46,0x6f,0x6e,0x74,0x20,0x28,
        0x63,0x29,0x20,0x4d,0x69,0x63,0x72,0x6f,0x50,0x72,
        0x6f,0x66,0x20,0x32,0x37,0x20,0x53,0x65,0x70,0x74,
        0x65,0x6d,0x62,0x2e,0x31,0x39,0x38,0x39,0x00,0x0d,
        0x0a,0x0d,0x0a,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
        0x02,0x00,0x12,0x00,0x34,0x00,0x46,0x00,0x94,0x00,
        0xd0,0x00,0x2e,0x01,0x3e,0x01,0x64,0x01,0x8a,0x01,
        0x98,0x01,0xa2,0x01,0xb4,0x01,0xba,0x01,0xc6,0x01,
        0xcc,0x01,0xf0,0x01,0xfa,0x01,0x18,0x02,0x38,0x02,
        0x44,0x02,0x68,0x02,0x98,0x02,0xa2,0x02,0xde,0x02,
        0x0e,0x03,0x24,0x03,0x40,0x03,0x48,0x03,0x52,0x03,
        0x5a,0x03,0x82,0x03,0xec,0x03,0xfa,0x03,0x26,0x04,
        0x4c,0x04,0x6a,0x04,0x7c,0x04,0x8a,0x04,0xb6,0x04,
        0xc4,0x04,0xca,0x04,0xe0,0x04,0xee,0x04,0xf8,0x04,
        0x0a,0x05,0x18,0x05,0x44,0x05,0x5e,0x05,0x8e,0x05,
        0xac,0x05,0xd6,0x05,0xe0,0x05,0xf6,0x05,0x00,0x06,
        0x12,0x06,0x1c,0x06,0x28,0x06,0x36,0x06,0x48,0x06,
        0x4e,0x06,0x60,0x06,0x6e,0x06,0x74,0x06,0x84,0x06,
        0xa6,0x06,0xc8,0x06,0xe6,0x06,0x08,0x07,0x2c,0x07,
        0x3c,0x07,0x68,0x07,0x7c,0x07,0x8c,0x07,0xa2,0x07,
        0xb0,0x07,0xb6,0x07,0xd8,0x07,0xec,0x07,0x10,0x08,
        0x32,0x08,0x54,0x08,0x64,0x08,0x88,0x08,0x98,0x08,
        0xac,0x08,0xb6,0x08,0xc8,0x08,0xd2,0x08,0xe4,0x08,
        0xf2,0x08,0x3e,0x09,0x48,0x09,0x94,0x09,0xc2,0x09,
        0xc4,0x09,0xd0,0x09,0xe2,0x09,0x04,0x0a,0x0e,0x0a,
        0x26,0x0a,0x34,0x0a,0x4a,0x0a,0x66,0x0a,0x70,0x0a,
        0x7e,0x0a,0x8e,0x0a,0x9a,0x0a,0xa6,0x0a,0xb4,0x0a,
        0xd8,0x0a,0xe2,0x0a,0xf6,0x0a,0x18,0x0b,0x22,0x0b,
        0x32,0x0b,0x56,0x0b,0x60,0x0b,0x6e,0x0b,0x7c,0x0b,
        0x8a,0x0b,0x9c,0x0b,0x9e,0x0b,0xb2,0x0b,0xc2,0x0b,
        0xd8,0x0b,0xf4,0x0b,0x08,0x0c,0x30,0x0c,0x56,0x0c,
        0x72,0x0c,0x90,0x0c,0xb2,0x0c,0xce,0x0c,0xe2,0x0c,
        0xfe,0x0c,0x10,0x0d,0x26,0x0d,0x36,0x0d,0x42,0x0d,
        0x4e,0x0d,0x5c,0x0d,0x78,0x0d,0x8c,0x0d,0x8e,0x0d,
        0x90,0x0d,0x92,0x0d,0x94,0x0d,0x96,0x0d,0x98,0x0d,
        0x9a,0x0d,0x9c,0x0d,0x9e,0x0d,0xa0,0x0d,0xa2,0x0d,
        0xa4,0x0d,0xa6,0x0d,0xa8,0x0d,0xaa,0x0d,0xac,0x0d,
        0xae,0x0d,0xb0,0x0d,0xb2,0x0d,0xb4,0x0d,0xb6,0x0d,
        0xb8,0x0d,0xba,0x0d,0xbc,0x0d,0xbe,0x0d,0xc0,0x0d,
        0xc2,0x0d,0xc4,0x0d,0xc6,0x0d,0xc8,0x0d,0xca,0x0d,
        0xcc,0x0d,0xce,0x0d,0xd0,0x0d,0xd2,0x0d,0xd4,0x0d,
        0xd6,0x0d,0xd8,0x0d,0xda,0x0d,0xdc,0x0d,0xde,0x0d,
        0xe0,0x0d,0xe2,0x0d,0xe4,0x0d,0xe6,0x0d,0xe8,0x0d,
        0xea,0x0d,0xec,0x0d,0x0c,0x0e,0x26,0x0e,0x48,0x0e,
        0x64,0x0e,0x88,0x0e,0x92,0x0e,0xa6,0x0e,0xb4,0x0e,
        0xd0,0x0e,0xee,0x0e,0x02,0x0f,0x16,0x0f,0x26,0x0f,
        0x3c,0x0f,0x58,0x0f,0x6c,0x0f,0x6c,0x0f,0x6c,0x0f,
        0x6c,0x0f,0x6c,0x0f,0x6c,0x0f,0x6c,0x0f,0x6c,0x0f,
        0x6c,0x0f,0x6c,0x0f,0x6c,0x0f,0x6c,0x0f,0x6c,0x0f,
        0x6c,0x0f,0x6c,0x0f,0x6c,0x0f,0x6c,0x0f,0x10,0x80,
        0x05,0x95,0x00,0x72,0x00,0xfb,0xff,0x7f,0x01,0x7f,
        0x01,0x01,0xff,0x01,0x05,0xfe,0x05,0x95,0xff,0x7f,
        0x00,0x7a,0x01,0x86,0xff,0x7a,0x01,0x87,0x01,0x7f,
        0xfe,0x7a,0x0a,0x87,0xff,0x7f,0x00,0x7a,0x01,0x86,
        0xff,0x7a,0x01,0x87,0x01,0x7f,0xfe,0x7a,0x05,0xf2,
        0x0b,0x95,0xf9,0x64,0x0d,0x9c,0xf9,0x64,0xfa,0x91,
        0x0e,0x00,0xf1,0xfa,0x0e,0x00,0x04,0xfc,0x08,0x99,
        0x00,0x63,0x04,0x9d,0x00,0x63,0x04,0x96,0xff,0x7f,
        0x01,0x7f,0x01,0x01,0x00,0x01,0xfe,0x02,0xfd,0x01,
        0xfc,0x00,0xfd,0x7f,0xfe,0x7e,0x00,0x7e,0x01,0x7e,
        0x01,0x7f,0x02,0x7f,0x06,0x7e,0x02,0x7f,0x02,0x7e,
        0xf2,0x89,0x02,0x7e,0x02,0x7f,0x06,0x7e,0x02,0x7f,
        0x01,0x7f,0x01,0x7e,0x00,0x7c,0xfe,0x7e,0xfd,0x7f,
        0xfc,0x00,0xfd,0x01,0xfe,0x02,0x00,0x01,0x01,0x01,
        0x01,0x7f,0xff,0x7f,0x10,0xfd,0x15,0x95,0xee,0x6b,
        0x05,0x95,0x02,0x7e,0x00,0x7e,0xff,0x7e,0xfe,0x7f,
        0xfe,0x00,0xfe,0x02,0x00,0x02,0x01,0x02,0x02,0x01,
        0x02,0x00,0x02,0x7f,0x03,0x7f,0x03,0x00,0x03,0x01,
        0x02,0x01,0xfc,0xf2,0xfe,0x7f,0xff,0x7e,0x00,0x7e,
        0x02,0x7e,0x02,0x00,0x02,0x01,0x01,0x02,0x00,0x02,
        0xfe,0x02,0xfe,0x00,0x07,0xf9,0x15,0x8d,0xff,0x7f,
        0x01,0x7f,0x01,0x01,0x00,0x01,0xff,0x01,0xff,0x00,
        0xff,0x7f,0xff,0x7e,0xfe,0x7b,0xfe,0x7d,0xfe,0x7e,
        0xfe,0x7f,0xfd,0x00,0xfd,0x01,0xff,0x02,0x00,0x03,
        0x01,0x02,0x06,0x04,0x02,0x02,0x01,0x02,0x00,0x02,
        0xff,0x02,0xfe,0x01,0xfe,0x7f,0xff,0x7e,0x00,0x7e,
        0x01,0x7d,0x02,0x7d,0x05,0x79,0x02,0x7e,0x03,0x7f,
        0x01,0x00,0x01,0x01,0x00,0x01,0xf1,0xfe,0xfe,0x01,
        0xff,0x02,0x00,0x03,0x01,0x02,0x02,0x02,0x00,0x86,
        0x01,0x7e,0x08,0x75,0x02,0x7e,0x02,0x7f,0x05,0x80,
        0x05,0x93,0xff,0x01,0x01,0x01,0x01,0x7f,0x00,0x7e,
        0xff,0x7e,0xff,0x7f,0x06,0xf1,0x0b,0x99,0xfe,0x7e,
        0xfe,0x7d,0xfe,0x7c,0xff,0x7b,0x00,0x7c,0x01,0x7b,
        0x02,0x7c,0x02,0x7d,0x02,0x7e,0xfe,0x9e,0xfe,0x7c,
        0xff,0x7d,0xff,0x7b,0x00,0x7c,0x01,0x7b,0x01,0x7d,
        0x02,0x7c,0x05,0x85,0x03,0x99,0x02,0x7e,0x02,0x7d,
        0x02,0x7c,0x01,0x7b,0x00,0x7c,0xff,0x7b,0xfe,0x7c,
        0xfe,0x7d,0xfe,0x7e,0x02,0x9e,0x02,0x7c,0x01,0x7d,
        0x01,0x7b,0x00,0x7c,0xff,0x7b,0xff,0x7d,0xfe,0x7c,
        0x09,0x85,0x08,0x95,0x00,0x74,0xfb,0x89,0x0a,0x7a,
        0x00,0x86,0xf6,0x7a,0x0d,0xf4,0x0d,0x92,0x00,0x6e,
        0xf7,0x89,0x12,0x00,0x04,0xf7,0x06,0x81,0xff,0x7f,
        0xff,0x01,0x01,0x01,0x01,0x7f,0x00,0x7e,0xff,0x7e,
        0xff,0x7f,0x06,0x84,0x04,0x89,0x12,0x00,0x04,0xf7,
        0x05,0x82,0xff,0x7f,0x01,0x7f,0x01,0x01,0xff,0x01,
        0x05,0xfe,0x00,0xfd,0x0e,0x18,0x00,0xeb,0x09,0x95,
        0xfd,0x7f,0xfe,0x7d,0xff,0x7b,0x00,0x7d,0x01,0x7b,
        0x02,0x7d,0x03,0x7f,0x02,0x00,0x03,0x01,0x02,0x03,
        0x01,0x05,0x00,0x03,0xff,0x05,0xfe,0x03,0xfd,0x01,
        0xfe,0x00,0x0b,0xeb,0x06,0x91,0x02,0x01,0x03,0x03,
        0x00,0x6b,0x09,0x80,0x04,0x90,0x00,0x01,0x01,0x02,
        0x01,0x01,0x02,0x01,0x04,0x00,0x02,0x7f,0x01,0x7f,
        0x01,0x7e,0x00,0x7e,0xff,0x7e,0xfe,0x7d,0xf6,0x76,
        0x0e,0x00,0x03,0x80,0x05,0x95,0x0b,0x00,0xfa,0x78,
        0x03,0x00,0x02,0x7f,0x01,0x7f,0x01,0x7d,0x00,0x7e,
        0xff,0x7d,0xfe,0x7e,0xfd,0x7f,0xfd,0x00,0xfd,0x01,
        0xff,0x01,0xff,0x02,0x11,0xfc,0x0d,0x95,0xf6,0x72,
        0x0f,0x00,0xfb,0x8e,0x00,0x6b,0x07,0x80,0x0f,0x95,
        0xf6,0x00,0xff,0x77,0x01,0x01,0x03,0x01,0x03,0x00,
        0x03,0x7f,0x02,0x7e,0x01,0x7d,0x00,0x7e,0xff,0x7d,
        0xfe,0x7e,0xfd,0x7f,0xfd,0x00,0xfd,0x01,0xff,0x01,
        0xff,0x02,0x11,0xfc,0x10,0x92,0xff,0x02,0xfd,0x01,
        0xfe,0x00,0xfd,0x7f,0xfe,0x7d,0xff,0x7b,0x00,0x7b,
        0x01,0x7c,0x02,0x7e,0x03,0x7f,0x01,0x00,0x03,0x01,
        0x02,0x02,0x01,0x03,0x00,0x01,0xff,0x03,0xfe,0x02,
        0xfd,0x01,0xff,0x00,0xfd,0x7f,0xfe,0x7e,0xff,0x7d,
        0x10,0xf9,0x11,0x95,0xf6,0x6b,0xfc,0x95,0x0e,0x00,
        0x03,0xeb,0x08,0x95,0xfd,0x7f,0xff,0x7e,0x00,0x7e,
        0x01,0x7e,0x02,0x7f,0x04,0x7f,0x03,0x7f,0x02,0x7e,
        0x01,0x7e,0x00,0x7d,0xff,0x7e,0xff,0x7f,0xfd,0x7f,
        0xfc,0x00,0xfd,0x01,0xff,0x01,0xff,0x02,0x00,0x03,
        0x01,0x02,0x02,0x02,0x03,0x01,0x04,0x01,0x02,0x01,
        0x01,0x02,0x00,0x02,0xff,0x02,0xfd,0x01,0xfc,0x00,
        0x0c,0xeb,0x10,0x8e,0xff,0x7d,0xfe,0x7e,0xfd,0x7f,
        0xff,0x00,0xfd,0x01,0xfe,0x02,0xff,0x03,0x00,0x01,
        0x01,0x03,0x02,0x02,0x03,0x01,0x01,0x00,0x03,0x7f,
        0x02,0x7e,0x01,0x7c,0x00,0x7b,0xff,0x7b,0xfe,0x7d,
        0xfd,0x7f,0xfe,0x00,0xfd,0x01,0xff,0x02,0x10,0xfd,
        0x05,0x8e,0xff,0x7f,0x01,0x7f,0x01,0x01,0xff,0x01,
        0x00,0xf4,0xff,0x7f,0x01,0x7f,0x01,0x01,0xff,0x01,
        0x05,0xfe,0x05,0x8e,0xff,0x7f,0x01,0x7f,0x01,0x01,
        0xff,0x01,0x01,0xf3,0xff,0x7f,0xff,0x01,0x01,0x01,
        0x01,0x7f,0x00,0x7e,0xff,0x7e,0xff,0x7f,0x06,0x84,
        0x14,0x92,0xf0,0x77,0x10,0x77,0x04,0x80,0x04,0x8c,
        0x12,0x00,0xee,0xfa,0x12,0x00,0x04,0xfa,0x04,0x92,
        0x10,0x77,0xf0,0x77,0x14,0x80,0x03,0x90,0x00,0x01,
        0x01,0x02,0x01,0x01,0x02,0x01,0x04,0x00,0x02,0x7f,
        0x01,0x7f,0x01,0x7e,0x00,0x7e,0xff,0x7e,0xff,0x7f,
        0xfc,0x7e,0x00,0x7d,0x00,0xfb,0xff,0x7f,0x01,0x7f,
        0x01,0x01,0xff,0x01,0x09,0xfe,0x12,0x8d,0xff,0x02,
        0xfe,0x01,0xfd,0x00,0xfe,0x7f,0xff,0x7f,0xff,0x7d,
        0x00,0x7d,0x01,0x7e,0x02,0x7f,0x03,0x00,0x02,0x01,
        0x01,0x02,0xfb,0x88,0xfe,0x7e,0xff,0x7d,0x00,0x7d,
        0x01,0x7e,0x01,0x7f,0x07,0x8b,0xff,0x78,0x00,0x7e,
        0x02,0x7f,0x02,0x00,0x02,0x02,0x01,0x03,0x00,0x02,
        0xff,0x03,0xff,0x02,0xfe,0x02,0xfe,0x01,0xfd,0x01,
        0xfd,0x00,0xfd,0x7f,0xfe,0x7f,0xfe,0x7e,0xff,0x7e,
        0xff,0x7d,0x00,0x7d,0x01,0x7d,0x01,0x7e,0x02,0x7e,
        0x02,0x7f,0x03,0x7f,0x03,0x00,0x03,0x01,0x02,0x01,
        0x01,0x01,0xfe,0x8d,0xff,0x78,0x00,0x7e,0x01,0x7f,
        0x08,0xfb,0x09,0x95,0xf8,0x6b,0x08,0x95,0x08,0x6b,
        0xf3,0x87,0x0a,0x00,0x04,0xf9,0x04,0x95,0x00,0x6b,
        0x00,0x95,0x09,0x00,0x03,0x7f,0x01,0x7f,0x01,0x7e,
        0x00,0x7e,0xff,0x7e,0xff,0x7f,0xfd,0x7f,0xf7,0x80,
        0x09,0x00,0x03,0x7f,0x01,0x7f,0x01,0x7e,0x00,0x7d,
        0xff,0x7e,0xff,0x7f,0xfd,0x7f,0xf7,0x00,0x11,0x80,
        0x12,0x90,0xff,0x02,0xfe,0x02,0xfe,0x01,0xfc,0x00,
        0xfe,0x7f,0xfe,0x7e,0xff,0x7e,0xff,0x7d,0x00,0x7b,
        0x01,0x7d,0x01,0x7e,0x02,0x7e,0x02,0x7f,0x04,0x00,
        0x02,0x01,0x02,0x02,0x01,0x02,0x03,0xfb,0x04,0x95,
        0x00,0x6b,0x00,0x95,0x07,0x00,0x03,0x7f,0x02,0x7e,
        0x01,0x7e,0x01,0x7d,0x00,0x7b,0xff,0x7d,0xff,0x7e,
        0xfe,0x7e,0xfd,0x7f,0xf9,0x00,0x11,0x80,0x04,0x95,
        0x00,0x6b,0x00,0x95,0x0d,0x00,0xf3,0xf6,0x08,0x00,
        0xf8,0xf5,0x0d,0x00,0x02,0x80,0x04,0x95,0x00,0x6b,
        0x00,0x95,0x0d,0x00,0xf3,0xf6,0x08,0x00,0x06,0xf5,
        0x12,0x90,0xff,0x02,0xfe,0x02,0xfe,0x01,0xfc,0x00,
        0xfe,0x7f,0xfe,0x7e,0xff,0x7e,0xff,0x7d,0x00,0x7b,
        0x01,0x7d,0x01,0x7e,0x02,0x7e,0x02,0x7f,0x04,0x00,
        0x02,0x01,0x02,0x02,0x01,0x02,0x00,0x03,0xfb,0x80,
        0x05,0x00,0x03,0xf8,0x04,0x95,0x00,0x6b,0x0e,0x95,
        0x00,0x6b,0xf2,0x8b,0x0e,0x00,0x04,0xf5,0x04,0x95,
        0x00,0x6b,0x04,0x80,0x0c,0x95,0x00,0x70,0xff,0x7d,
        0xff,0x7f,0xfe,0x7f,0xfe,0x00,0xfe,0x01,0xff,0x01,
        0xff,0x03,0x00,0x02,0x0e,0xf9,0x04,0x95,0x00,0x6b,
        0x0e,0x95,0xf2,0x72,0x05,0x85,0x09,0x74,0x03,0x80,
        0x04,0x95,0x00,0x6b,0x00,0x80,0x0c,0x00,0x01,0x80,
        0x04,0x95,0x00,0x6b,0x00,0x95,0x08,0x6b,0x08,0x95,
        0xf8,0x6b,0x08,0x95,0x00,0x6b,0x04,0x80,0x04,0x95,
        0x00,0x6b,0x00,0x95,0x0e,0x6b,0x00,0x95,0x00,0x6b,
        0x04,0x80,0x09,0x95,0xfe,0x7f,0xfe,0x7e,0xff,0x7e,
        0xff,0x7d,0x00,0x7b,0x01,0x7d,0x01,0x7e,0x02,0x7e,
        0x02,0x7f,0x04,0x00,0x02,0x01,0x02,0x02,0x01,0x02,
        0x01,0x03,0x00,0x05,0xff,0x03,0xff,0x02,0xfe,0x02,
        0xfe,0x01,0xfc,0x00,0x0d,0xeb,0x04,0x95,0x00,0x6b,
        0x00,0x95,0x09,0x00,0x03,0x7f,0x01,0x7f,0x01,0x7e,
        0x00,0x7d,0xff,0x7e,0xff,0x7f,0xfd,0x7f,0xf7,0x00,
        0x11,0xf6,0x09,0x95,0xfe,0x7f,0xfe,0x7e,0xff,0x7e,
        0xff,0x7d,0x00,0x7b,0x01,0x7d,0x01,0x7e,0x02,0x7e,
        0x02,0x7f,0x04,0x00,0x02,0x01,0x02,0x02,0x01,0x02,
        0x01,0x03,0x00,0x05,0xff,0x03,0xff,0x02,0xfe,0x02,
        0xfe,0x01,0xfc,0x00,0x03,0xef,0x06,0x7a,0x04,0x82,
        0x04,0x95,0x00,0x6b,0x00,0x95,0x09,0x00,0x03,0x7f,
        0x01,0x7f,0x01,0x7e,0x00,0x7e,0xff,0x7e,0xff,0x7f,
        0xfd,0x7f,0xf7,0x00,0x07,0x80,0x07,0x75,0x03,0x80,
        0x11,0x92,0xfe,0x02,0xfd,0x01,0xfc,0x00,0xfd,0x7f,
        0xfe,0x7e,0x00,0x7e,0x01,0x7e,0x01,0x7f,0x02,0x7f,
        0x06,0x7e,0x02,0x7f,0x01,0x7f,0x01,0x7e,0x00,0x7d,
        0xfe,0x7e,0xfd,0x7f,0xfc,0x00,0xfd,0x01,0xfe,0x02,
        0x11,0xfd,0x08,0x95,0x00,0x6b,0xf9,0x95,0x0e,0x00,
        0x01,0xeb,0x04,0x95,0x00,0x71,0x01,0x7d,0x02,0x7e,
        0x03,0x7f,0x02,0x00,0x03,0x01,0x02,0x02,0x01,0x03,
        0x00,0x0f,0x04,0xeb,0x01,0x95,0x08,0x6b,0x08,0x95,
        0xf8,0x6b,0x09,0x80,0x02,0x95,0x05,0x6b,0x05,0x95,
        0xfb,0x6b,0x05,0x95,0x05,0x6b,0x05,0x95,0xfb,0x6b,
        0x07,0x80,0x03,0x95,0x0e,0x6b,0x00,0x95,0xf2,0x6b,
        0x11,0x80,0x01,0x95,0x08,0x76,0x00,0x75,0x08,0x95,
        0xf8,0x76,0x09,0xf5,0x11,0x95,0xf2,0x6b,0x00,0x95,
        0x0e,0x00,0xf2,0xeb,0x0e,0x00,0x03,0x80,0x03,0x93,
        0x00,0x6c,0x01,0x94,0x00,0x6c,0xff,0x94,0x05,0x00,
        0xfb,0xec,0x05,0x00,0x02,0x81,0x00,0x95,0x0e,0x68,
        0x00,0x83,0x06,0x93,0x00,0x6c,0x01,0x94,0x00,0x6c,
        0xfb,0x94,0x05,0x00,0xfb,0xec,0x05,0x00,0x03,0x81,
        0x03,0x87,0x08,0x05,0x08,0x7b,0xf0,0x80,0x08,0x04,
        0x08,0x7c,0x03,0xf9,0x01,0x80,0x10,0x00,0x01,0x80,
        0x06,0x95,0xff,0x7f,0xff,0x7e,0x00,0x7e,0x01,0x7f,
        0x01,0x01,0xff,0x01,0x05,0xef,0x0f,0x8e,0x00,0x72,
        0x00,0x8b,0xfe,0x02,0xfe,0x01,0xfd,0x00,0xfe,0x7f,
        0xfe,0x7e,0xff,0x7d,0x00,0x7e,0x01,0x7d,0x02,0x7e,
        0x02,0x7f,0x03,0x00,0x02,0x01,0x02,0x02,0x04,0xfd,
        0x04,0x95,0x00,0x6b,0x00,0x8b,0x02,0x02,0x02,0x01,
        0x03,0x00,0x02,0x7f,0x02,0x7e,0x01,0x7d,0x00,0x7e,
        0xff,0x7d,0xfe,0x7e,0xfe,0x7f,0xfd,0x00,0xfe,0x01,
        0xfe,0x02,0x0f,0xfd,0x0f,0x8b,0xfe,0x02,0xfe,0x01,
        0xfd,0x00,0xfe,0x7f,0xfe,0x7e,0xff,0x7d,0x00,0x7e,
        0x01,0x7d,0x02,0x7e,0x02,0x7f,0x03,0x00,0x02,0x01,
        0x02,0x02,0x03,0xfd,0x0f,0x95,0x00,0x6b,0x00,0x8b,
        0xfe,0x02,0xfe,0x01,0xfd,0x00,0xfe,0x7f,0xfe,0x7e,
        0xff,0x7d,0x00,0x7e,0x01,0x7d,0x02,0x7e,0x02,0x7f,
        0x03,0x00,0x02,0x01,0x02,0x02,0x04,0xfd,0x03,0x88,
        0x0c,0x00,0x00,0x02,0xff,0x02,0xff,0x01,0xfe,0x01,
        0xfd,0x00,0xfe,0x7f,0xfe,0x7e,0xff,0x7d,0x00,0x7e,
        0x01,0x7d,0x02,0x7e,0x02,0x7f,0x03,0x00,0x02,0x01,
        0x02,0x02,0x03,0xfd,0x0a,0x95,0xfe,0x00,0xfe,0x7f,
        0xff,0x7d,0x00,0x6f,0xfd,0x8e,0x07,0x00,0x03,0xf2,
        0x0f,0x8e,0x00,0x70,0xff,0x7d,0xff,0x7f,0xfe,0x7f,
        0xfd,0x00,0xfe,0x01,0x09,0x91,0xfe,0x02,0xfe,0x01,
        0xfd,0x00,0xfe,0x7f,0xfe,0x7e,0xff,0x7d,0x00,0x7e,
        0x01,0x7d,0x02,0x7e,0x02,0x7f,0x03,0x00,0x02,0x01,
        0x02,0x02,0x04,0xfd,0x04,0x95,0x00,0x6b,0x00,0x8a,
        0x03,0x03,0x02,0x01,0x03,0x00,0x02,0x7f,0x01,0x7d,
        0x00,0x76,0x04,0x80,0x03,0x95,0x01,0x7f,0x01,0x01,
        0xff,0x01,0xff,0x7f,0x01,0xf9,0x00,0x72,0x04,0x80,
        0x05,0x95,0x01,0x7f,0x01,0x01,0xff,0x01,0xff,0x7f,
        0x01,0xf9,0x00,0x6f,0xff,0x7d,0xfe,0x7f,0xfe,0x00,
        0x09,0x87,0x04,0x95,0x00,0x6b,0x0a,0x8e,0xf6,0x76,
        0x04,0x84,0x07,0x78,0x02,0x80,0x04,0x95,0x00,0x6b,
        0x04,0x80,0x04,0x8e,0x00,0x72,0x00,0x8a,0x03,0x03,
        0x02,0x01,0x03,0x00,0x02,0x7f,0x01,0x7d,0x00,0x76,
        0x00,0x8a,0x03,0x03,0x02,0x01,0x03,0x00,0x02,0x7f,
        0x01,0x7d,0x00,0x76,0x04,0x80,0x04,0x8e,0x00,0x72,
        0x00,0x8a,0x03,0x03,0x02,0x01,0x03,0x00,0x02,0x7f,
        0x01,0x7d,0x00,0x76,0x04,0x80,0x08,0x8e,0xfe,0x7f,
        0xfe,0x7e,0xff,0x7d,0x00,0x7e,0x01,0x7d,0x02,0x7e,
        0x02,0x7f,0x03,0x00,0x02,0x01,0x02,0x02,0x01,0x03,
        0x00,0x02,0xff,0x03,0xfe,0x02,0xfe,0x01,0xfd,0x00,
        0x0b,0xf2,0x04,0x8e,0x00,0x6b,0x00,0x92,0x02,0x02,
        0x02,0x01,0x03,0x00,0x02,0x7f,0x02,0x7e,0x01,0x7d,
        0x00,0x7e,0xff,0x7d,0xfe,0x7e,0xfe,0x7f,0xfd,0x00,
        0xfe,0x01,0xfe,0x02,0x0f,0xfd,0x0f,0x8e,0x00,0x6b,
        0x00,0x92,0xfe,0x02,0xfe,0x01,0xfd,0x00,0xfe,0x7f,
        0xfe,0x7e,0xff,0x7d,0x00,0x7e,0x01,0x7d,0x02,0x7e,
        0x02,0x7f,0x03,0x00,0x02,0x01,0x02,0x02,0x04,0xfd,
        0x04,0x8e,0x00,0x72,0x00,0x88,0x01,0x03,0x02,0x02,
        0x02,0x01,0x03,0x00,0x01,0xf2,0x0e,0x8b,0xff,0x02,
        0xfd,0x01,0xfd,0x00,0xfd,0x7f,0xff,0x7e,0x01,0x7e,
        0x02,0x7f,0x05,0x7f,0x02,0x7f,0x01,0x7e,0x00,0x7f,
        0xff,0x7e,0xfd,0x7f,0xfd,0x00,0xfd,0x01,0xff,0x02,
        0x0e,0xfd,0x05,0x95,0x00,0x6f,0x01,0x7d,0x02,0x7f,
        0x02,0x00,0xf8,0x8e,0x07,0x00,0x03,0xf2,0x04,0x8e,
        0x00,0x76,0x01,0x7d,0x02,0x7f,0x03,0x00,0x02,0x01,
        0x03,0x03,0x00,0x8a,0x00,0x72,0x04,0x80,0x02,0x8e,
        0x06,0x72,0x06,0x8e,0xfa,0x72,0x08,0x80,0x03,0x8e,
        0x04,0x72,0x04,0x8e,0xfc,0x72,0x04,0x8e,0x04,0x72,
        0x04,0x8e,0xfc,0x72,0x07,0x80,0x03,0x8e,0x0b,0x72,
        0x00,0x8e,0xf5,0x72,0x0e,0x80,0x02,0x8e,0x06,0x72,
        0x06,0x8e,0xfa,0x72,0xfe,0x7c,0xfe,0x7e,0xfe,0x7f,
        0xff,0x00,0x0f,0x87,0x0e,0x8e,0xf5,0x72,0x00,0x8e,
        0x0b,0x00,0xf5,0xf2,0x0b,0x00,0x03,0x80,0x09,0x99,
        0xfe,0x7f,0xff,0x7f,0xff,0x7e,0x00,0x7e,0x01,0x7e,
        0x01,0x7f,0x01,0x7e,0x00,0x7e,0xfe,0x7e,0x01,0x8e,
        0xff,0x7e,0x00,0x7e,0x01,0x7e,0x01,0x7f,0x01,0x7e,
        0x00,0x7e,0xff,0x7e,0xfc,0x7e,0x04,0x7e,0x01,0x7e,
        0x00,0x7e,0xff,0x7e,0xff,0x7f,0xff,0x7e,0x00,0x7e,
        0x01,0x7e,0xff,0x8e,0x02,0x7e,0x00,0x7e,0xff,0x7e,
        0xff,0x7f,0xff,0x7e,0x00,0x7e,0x01,0x7e,0x01,0x7f,
        0x02,0x7f,0x05,0x87,0x04,0x95,0x00,0x77,0x00,0xfd,
        0x00,0x77,0x04,0x80,0x05,0x99,0x02,0x7f,0x01,0x7f,
        0x01,0x7e,0x00,0x7e,0xff,0x7e,0xff,0x7f,0xff,0x7e,
        0x00,0x7e,0x02,0x7e,0xff,0x8e,0x01,0x7e,0x00,0x7e,
        0xff,0x7e,0xff,0x7f,0xff,0x7e,0x00,0x7e,0x01,0x7e,
        0x04,0x7e,0xfc,0x7e,0xff,0x7e,0x00,0x7e,0x01,0x7e,
        0x01,0x7f,0x01,0x7e,0x00,0x7e,0xff,0x7e,0x01,0x8e,
        0xfe,0x7e,0x00,0x7e,0x01,0x7e,0x01,0x7f,0x01,0x7e,
        0x00,0x7e,0xff,0x7e,0xff,0x7f,0xfe,0x7f,0x09,0x87,
        0x03,0x86,0x00,0x02,0x01,0x03,0x02,0x01,0x02,0x00,
        0x02,0x7f,0x04,0x7d,0x02,0x7f,0x02,0x00,0x02,0x01,
        0x01,0x02,0xee,0xfe,0x01,0x02,0x02,0x01,0x02,0x00,
        0x02,0x7f,0x04,0x7d,0x02,0x7f,0x02,0x00,0x02,0x01,
        0x01,0x03,0x00,0x02,0x03,0xf4,0x10,0x80,0x03,0x80,
        0x07,0x15,0x08,0x6b,0xfe,0x85,0xf5,0x00,0x10,0xfb,
        0x0d,0x95,0xf6,0x00,0x00,0x6b,0x0a,0x00,0x02,0x02,
        0x00,0x08,0xfe,0x02,0xf6,0x00,0x0e,0xf4,0x03,0x80,
        0x00,0x15,0x0a,0x00,0x02,0x7e,0x00,0x7e,0x00,0x7d,
        0x00,0x7e,0xfe,0x7f,0xf6,0x00,0x0a,0x80,0x02,0x7e,
        0x01,0x7e,0x00,0x7d,0xff,0x7d,0xfe,0x7f,0xf6,0x00,
        0x10,0x80,0x03,0x80,0x00,0x15,0x0c,0x00,0xff,0x7e,
        0x03,0xed,0x03,0xfd,0x00,0x03,0x02,0x00,0x00,0x12,
        0x02,0x03,0x0a,0x00,0x00,0x6b,0x02,0x00,0x00,0x7d,
        0xfe,0x83,0xf4,0x00,0x11,0x80,0x0f,0x80,0xf4,0x00,
        0x00,0x15,0x0c,0x00,0xff,0xf6,0xf5,0x00,0x0f,0xf5,
        0x04,0x95,0x07,0x76,0x00,0x0a,0x07,0x80,0xf9,0x76,
        0x00,0x75,0xf8,0x80,0x07,0x0c,0x09,0xf4,0xf9,0x0c,
        0x09,0xf4,0x03,0x92,0x02,0x03,0x07,0x00,0x03,0x7d,
        0x00,0x7b,0xfc,0x7e,0x04,0x7d,0x00,0x7a,0xfd,0x7e,
        0xf9,0x00,0xfe,0x02,0x06,0x89,0x02,0x00,0x06,0xf5,
        0x03,0x95,0x00,0x6b,0x0c,0x15,0x00,0x6b,0x02,0x80,
        0x03,0x95,0x00,0x6b,0x0c,0x15,0x00,0x6b,0xf8,0x96,
        0x03,0x00,0x07,0xea,0x03,0x80,0x00,0x15,0x0c,0x80,
        0xf7,0x76,0xfd,0x00,0x03,0x80,0x0a,0x75,0x03,0x80,
        0x03,0x80,0x07,0x13,0x02,0x02,0x03,0x00,0x00,0x6b,
        0x02,0x80,0x03,0x80,0x00,0x15,0x09,0x6b,0x09,0x15,
        0x00,0x6b,0x03,0x80,0x03,0x80,0x00,0x15,0x00,0xf6,
        0x0d,0x00,0x00,0x8a,0x00,0x6b,0x03,0x80,0x07,0x80,
        0xfd,0x00,0xff,0x03,0x00,0x04,0x00,0x07,0x00,0x04,
        0x01,0x02,0x03,0x01,0x06,0x00,0x03,0x7f,0x01,0x7e,
        0x01,0x7c,0x00,0x79,0xff,0x7c,0xff,0x7d,0xfd,0x00,
        0xfa,0x00,0x0e,0x80,0x03,0x80,0x00,0x15,0x0c,0x00,
        0x00,0x6b,0x02,0x80,0x03,0x80,0x00,0x15,0x0a,0x00,
        0x02,0x7f,0x01,0x7d,0x00,0x7b,0xff,0x7e,0xfe,0x7f,
        0xf6,0x00,0x10,0xf7,0x11,0x8f,0xff,0x03,0xff,0x02,
        0xfe,0x01,0xfa,0x00,0xfd,0x7f,0xff,0x7e,0x00,0x7c,
        0x00,0x79,0x00,0x7b,0x01,0x7e,0x03,0x00,0x06,0x00,
        0x02,0x00,0x01,0x03,0x01,0x02,0x03,0xfb,0x03,0x95,
        0x0c,0x00,0xfa,0x80,0x00,0x6b,0x09,0x80,0x03,0x95,
        0x00,0x77,0x06,0x7a,0x06,0x06,0x00,0x09,0xfa,0xf1,
        0xfa,0x7a,0x0e,0x80,0x03,0x87,0x00,0x0b,0x02,0x02,
        0x03,0x00,0x02,0x7e,0x01,0x02,0x04,0x00,0x02,0x7e,
        0x00,0x75,0xfe,0x7e,0xfc,0x00,0xff,0x01,0xfe,0x7f,
        0xfd,0x00,0xfe,0x02,0x07,0x8e,0x00,0x6b,0x09,0x80,
        0x03,0x80,0x0e,0x15,0xf2,0x80,0x0e,0x6b,0x03,0x80,
        0x03,0x95,0x00,0x6b,0x0e,0x00,0x00,0x7d,0xfe,0x98,
        0x00,0x6b,0x05,0x80,0x03,0x95,0x00,0x75,0x02,0x7d,
        0x0a,0x00,0x00,0x8e,0x00,0x6b,0x02,0x80,0x03,0x95,
        0x00,0x6b,0x10,0x00,0x00,0x15,0xf8,0x80,0x00,0x6b,
        0x0a,0x80,0x03,0x95,0x00,0x6b,0x10,0x00,0x00,0x15,
        0xf8,0x80,0x00,0x6b,0x0a,0x00,0x00,0x7d,0x02,0x83,
        0x10,0x80,0x03,0x95,0x00,0x6b,0x09,0x00,0x03,0x02,
        0x00,0x08,0xfd,0x02,0xf7,0x00,0x0e,0x89,0x00,0x6b,
        0x03,0x80,0x03,0x95,0x00,0x6b,0x09,0x00,0x03,0x02,
        0x00,0x08,0xfd,0x02,0xf7,0x00,0x0e,0xf4,0x03,0x92,
        0x02,0x03,0x07,0x00,0x03,0x7d,0x00,0x70,0xfd,0x7e,
        0xf9,0x00,0xfe,0x02,0x03,0x89,0x09,0x00,0x02,0xf5,
        0x03,0x80,0x00,0x15,0x00,0xf5,0x07,0x00,0x00,0x08,
        0x02,0x03,0x06,0x00,0x02,0x7d,0x00,0x70,0xfe,0x7e,
        0xfa,0x00,0xfe,0x02,0x00,0x08,0x0c,0xf6,0x0f,0x80,
        0x00,0x15,0xf6,0x00,0xfe,0x7d,0x00,0x79,0x02,0x7e,
        0x0a,0x00,0xf4,0xf7,0x07,0x09,0x07,0xf7,0x03,0x8c,
        0x01,0x02,0x01,0x01,0x05,0x00,0x02,0x7f,0x01,0x7e,
        0x00,0x74,0x00,0x86,0xff,0x01,0xfe,0x01,0xfb,0x00,
        0xff,0x7f,0xff,0x7f,0x00,0x7c,0x01,0x7e,0x01,0x00,
        0x05,0x00,0x02,0x00,0x01,0x02,0x03,0xfe,0x04,0x8e,
        0x02,0x01,0x04,0x00,0x02,0x7f,0x01,0x7e,0x00,0x77,
        0xff,0x7e,0xfe,0x7f,0xfc,0x00,0xfe,0x01,0xff,0x02,
        0x00,0x09,0x01,0x02,0x02,0x02,0x03,0x01,0x02,0x01,
        0x01,0x01,0x01,0x02,0x02,0xeb,0x03,0x80,0x00,0x15,
        0x03,0x00,0x02,0x7e,0x00,0x7b,0xfe,0x7e,0xfd,0x00,
        0x03,0x80,0x04,0x00,0x03,0x7e,0x00,0x78,0xfd,0x7e,
        0xf9,0x00,0x0c,0x80,0x03,0x8c,0x02,0x02,0x02,0x01,
        0x03,0x00,0x02,0x7f,0x01,0x7d,0xfe,0x7e,0xf9,0x7d,
        0xff,0x7e,0x00,0x7d,0x03,0x7f,0x02,0x00,0x03,0x01,
        0x02,0x01,0x02,0xfe,0x0d,0x8c,0xff,0x02,0xfe,0x01,
        0xfc,0x00,0xfe,0x7f,0xff,0x7e,0x00,0x77,0x01,0x7e,
        0x02,0x7f,0x04,0x00,0x02,0x01,0x01,0x02,0x00,0x0f,
        0xff,0x02,0xfe,0x01,0xf9,0x00,0x0c,0xeb,0x03,0x88,
        0x0a,0x00,0x00,0x02,0x00,0x03,0xfe,0x02,0xfa,0x00,
        0xff,0x7e,0xff,0x7d,0x00,0x7b,0x01,0x7c,0x01,0x7f,
        0x06,0x00,0x02,0x02,0x03,0xfe,0x03,0x8f,0x06,0x77,
        0x06,0x09,0xfa,0x80,0x00,0x71,0xff,0x87,0xfb,0x79,
        0x07,0x87,0x05,0x79,0x02,0x80,0x03,0x8d,0x02,0x02,
        0x06,0x00,0x02,0x7e,0x00,0x7d,0xfc,0x7d,0x04,0x7e,
        0x00,0x7d,0xfe,0x7e,0xfa,0x00,0xfe,0x02,0x04,0x85,
        0x02,0x00,0x06,0xf9,0x03,0x8f,0x00,0x73,0x01,0x7e,
        0x07,0x00,0x02,0x02,0x00,0x0d,0x00,0xf3,0x01,0x7e,
        0x03,0x80,0x03,0x8f,0x00,0x73,0x01,0x7e,0x07,0x00,
        0x02,0x02,0x00,0x0d,0x00,0xf3,0x01,0x7e,0xf8,0x90,
        0x03,0x00,0x08,0xf0,0x03,0x80,0x00,0x15,0x00,0xf3,
        0x02,0x00,0x06,0x07,0xfa,0xf9,0x07,0x78,0x03,0x80,
        0x03,0x80,0x04,0x0c,0x02,0x03,0x04,0x00,0x00,0x71,
        0x02,0x80,0x03,0x80,0x00,0x0f,0x06,0x77,0x06,0x09,
        0x00,0x71,0x02,0x80,0x03,0x80,0x00,0x0f,0x0a,0xf1,
        0x00,0x0f,0xf6,0xf8,0x0a,0x00,0x02,0xf9,0x05,0x80,
        0xff,0x01,0xff,0x04,0x00,0x05,0x01,0x03,0x01,0x02,
        0x06,0x00,0x02,0x7e,0x00,0x7d,0x00,0x7b,0x00,0x7c,
        0xfe,0x7f,0xfa,0x00,0x0b,0x80,0x03,0x80,0x00,0x0f,
        0x00,0xfb,0x01,0x03,0x01,0x02,0x05,0x00,0x02,0x7e,
        0x01,0x7d,0x00,0x76,0x03,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,0x10,0x80,
        0x10,0x80,0x0a,0x8f,0x02,0x7f,0x01,0x7e,0x00,0x76,
        0xff,0x7f,0xfe,0x7f,0xfb,0x00,0xff,0x01,0xff,0x01,
        0x00,0x0a,0x01,0x02,0x01,0x01,0x05,0x00,0xf9,0x80,
        0x00,0x6b,0x0c,0x86,0x0d,0x8a,0xff,0x03,0xfe,0x02,
        0xfb,0x00,0xff,0x7e,0xff,0x7d,0x00,0x7b,0x01,0x7c,
        0x01,0x7f,0x05,0x00,0x02,0x01,0x01,0x03,0x03,0xfc,
        0x03,0x80,0x00,0x0f,0x00,0xfb,0x01,0x03,0x01,0x02,
        0x04,0x00,0x01,0x7e,0x01,0x7d,0x00,0x76,0x00,0x8a,
        0x01,0x03,0x02,0x02,0x03,0x00,0x02,0x7e,0x01,0x7d,
        0x00,0x76,0x03,0x80,0x03,0x8f,0x00,0x74,0x01,0x7e,
        0x02,0x7f,0x04,0x00,0x02,0x01,0x01,0x01,0x00,0x8d,
        0x00,0x6e,0xff,0x7e,0xfe,0x7f,0xfb,0x00,0xfe,0x01,
        0x0c,0x85,0x03,0x8d,0x01,0x02,0x03,0x00,0x02,0x7e,
        0x01,0x02,0x03,0x00,0x02,0x7e,0x00,0x74,0xfe,0x7f,
        0xfd,0x00,0xff,0x01,0xfe,0x7f,0xfd,0x00,0xff,0x01,
        0x00,0x0c,0x06,0x82,0x00,0x6b,0x08,0x86,0x03,0x80,
        0x0a,0x0f,0xf6,0x80,0x0a,0x71,0x03,0x80,0x03,0x8f,
        0x00,0x73,0x01,0x7e,0x07,0x00,0x02,0x02,0x00,0x0d,
        0x00,0xf3,0x01,0x7e,0x00,0x7e,0x03,0x82,0x03,0x8f,
        0x00,0x79,0x02,0x7e,0x08,0x00,0x00,0x89,0x00,0x71,
        0x02,0x80,0x03,0x8f,0x00,0x73,0x01,0x7e,0x03,0x00,
        0x02,0x02,0x00,0x0d,0x00,0xf3,0x01,0x7e,0x03,0x00,
        0x02,0x02,0x00,0x0d,0x00,0xf3,0x01,0x7e,0x03,0x80,
        0x03,0x8f,0x00,0x73,0x01,0x7e,0x03,0x00,0x02,0x02,
        0x00,0x0d,0x00,0xf3,0x01,0x7e,0x03,0x00,0x02,0x02,
        0x00,0x0d,0x00,0xf3,0x01,0x7e,0x00,0x7e,0x03,0x82,
        0x03,0x8d,0x00,0x02,0x02,0x00,0x00,0x71,0x08,0x00,
        0x02,0x02,0x00,0x06,0xfe,0x02,0xf8,0x00,0x0c,0xf6,
        0x03,0x8f,0x00,0x71,0x07,0x00,0x02,0x02,0x00,0x06,
        0xfe,0x02,0xf9,0x00,0x0c,0x85,0x00,0x71,0x02,0x80,
        0x03,0x8f,0x00,0x71,0x07,0x00,0x03,0x02,0x00,0x06,
        0xfd,0x02,0xf9,0x00,0x0c,0xf6,0x03,0x8d,0x02,0x02,
        0x06,0x00,0x02,0x7e,0x00,0x75,0xfe,0x7e,0xfa,0x00,
        0xfe,0x02,0x04,0x85,0x06,0x00,0x02,0xf9,0x03,0x80,
        0x00,0x0f,0x00,0xf8,0x04,0x00,0x00,0x06,0x02,0x02,
        0x04,0x00,0x02,0x7e,0x00,0x75,0xfe,0x7e,0xfc,0x00,
        0xfe,0x02,0x00,0x05,0x0a,0xf9,0x0d,0x80,0x00,0x0f,
        0xf7,0x00,0xff,0x7e,0x00,0x7b,0x01,0x7e,0x09,0x00,
        0xf6,0xfa,0x04,0x06,0x08,0xfa
];
