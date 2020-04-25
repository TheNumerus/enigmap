/// Helper struct for RGB images
#[derive(Debug, Clone)]
pub struct Image {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    color_mode: ColorMode
}

impl Image {
    pub fn new(width: u32, height: u32, color_mode: ColorMode) -> Image {
        let buffer = vec![0;(width * height * color_mode as u32) as usize];
        Image{width, height, buffer, color_mode}
    }

    pub fn from_buffer(width: u32, height: u32, buffer: Vec<u8>, color_mode: ColorMode) -> Image {
        Image{width, height, buffer, color_mode}
    }

    #[inline(always)]
    pub fn put_pixel(&mut self, x: u32, y: u32, color: [u8;3]) {
        let index = ((x + y * self.width) * 3) as usize;
        self.buffer[index] = color[0];
        self.buffer[index + 1] = color[1];
        self.buffer[index + 2] = color[2];
    }

    #[inline(always)]
    pub fn put_hor_line(&mut self, x: (u32, u32), y: u32, color: [u8;3]) {
        let start = ((x.0 + y * self.width) * 3) as usize;
        let len = (x.1 - x.0) as usize * 3;
        for pixel in self.buffer[start..(start + len)].chunks_exact_mut(3) {
            pixel[0] = color[0];
            pixel[1] = color[1];
            pixel[2] = color[2];
        }
    }

    #[inline(always)]
    pub fn put_pixel_rgba(&mut self, x: u32, y: u32, color: [u8;4]) {
        let index = ((x + y * self.width) * 4) as usize;
        self.buffer[index] = color[0];
        self.buffer[index + 1] = color[1];
        self.buffer[index + 2] = color[2];
        self.buffer[index + 3] = color[3];
    }

    pub fn from_fn<F>(width: u32, height: u32, function: F) -> Image 
        where F: Fn(u32, u32) -> [u8;3]
    {
        let buffer = vec![0;(width * height * 3) as usize];
        let mut img = Image{width, height, buffer, color_mode: ColorMode::Rgb};
        for x in 0..width {
            for y in 0..height {
                let color = function(x, y);
                img.put_pixel(x,y,color);
            }
        }
        img
    }

    pub fn from_fn_rgba<F>(width: u32, height: u32, function: F) -> Image 
        where F: Fn(u32, u32) -> [u8;4]
    {
        let buffer = vec![0;(width * height * 4) as usize];
        let mut img = Image{width, height, buffer, color_mode: ColorMode::Rgba};
        for x in 0..width {
            for y in 0..height {
                let color = function(x, y);
                img.put_pixel_rgba(x,y,color);
            }
        }
        img
    }

    #[inline(always)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline(always)]
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    #[inline(always)]
    pub fn color_mode(&self) -> &ColorMode {
        &self.color_mode
    }

    #[inline(always)]
    pub fn is_rgba(&self) -> bool {
        match self.color_mode {
            ColorMode::Rgb => false,
            ColorMode::Rgba => true
        }
    }

    #[inline(always)]
    pub fn get_pixel(&self, x: u32, y: u32) -> &[u8] {
        let index = ((x + y * self.width) * 3) as usize;
        &self.buffer[index..index + 3]
    }
}


#[derive(Copy, Clone, Debug)]
pub enum ColorMode {
    Rgb = 3,
    Rgba = 4
}
