// use alloc::format;
// use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
// use embedded_graphics::{
//     geometry::Size,
//     image::Image,
//     mono_font::{ascii::FONT_7X13, MonoTextStyle},
//     pixelcolor::raw::RawU32,
//     prelude::{DrawTarget, OriginDimensions, PixelColor, Point, RgbColor},
//     text::Text,
//     Drawable, Pixel,
// };
// use tinybmp::Bmp;
//
// pub fn init(frame_buffer: &'static mut FrameBuffer) {
//     let info = frame_buffer.info();
//     let mut display = Display::new(frame_buffer.buffer_mut(), info);
//
//     display
//         .clear(Color {
//             value: 0,
//             format: PixelFormat::Rgb,
//         })
//         .unwrap();
//
//     Text::new(
//         "Hello, World!",
//         Point::new(10, 10),
//         MonoTextStyle::new(&FONT_7X13, Color {
//             value: 0xff_ff_ff_00,
//             format: PixelFormat::Rgb,
//         }),
//     )
//     .draw(&mut display)
//     .unwrap();
//
//     let bmp_data = include_bytes!("../ferris.bmp");
//     let bmp = Bmp::from_slice(bmp_data).unwrap();
//     Image::new(&bmp, Point::new(100, 100))
//         .draw(&mut display)
//         .unwrap();
//
//     Text::new(
//         format!("Image drawn: {:?}", bmp_data).as_str(),
//         Point::new(10, 200),
//         MonoTextStyle::new(&FONT_7X13, Color {
//             value: 0xff_ff_ff_00,
//             format: PixelFormat::Rgb,
//         }),
//     )
//     .draw(&mut display)
//     .unwrap();
// }
//
// pub struct Display {
//     frame_buffer: &'static mut [u8],
//     info: FrameBufferInfo,
// }
//
// impl Display {
//     pub fn new(frame_buffer: &'static mut [u8], info: FrameBufferInfo) ->
// Self {         Self { frame_buffer, info }
//     }
//
//     fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
//         let pixel_offset = y * self.info.stride + x;
//         let bytes_per_pixel = self.info.bytes_per_pixel;
//         let byte_offset = pixel_offset * bytes_per_pixel;
//
//         self.frame_buffer[byte_offset..(byte_offset + bytes_per_pixel)]
//             .copy_from_slice(&color.to_ne_bytes());
//     }
// }
//
// impl OriginDimensions for Display {
//     fn size(&self) -> Size { Size::new(self.info.width as u32,
// self.info.height as u32) } }
//
// impl DrawTarget for Display {
//     type Color = Color;
//     type Error = !;
//
//     fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
//     where
//         I: IntoIterator<Item = Pixel<Self::Color>>,
//     {
//         for Pixel(coord, color) in pixels.into_iter() {
//             if let PixelFormat::Rgb = color.format {
//                 self.draw_pixel(coord.x as usize, coord.y as usize,
// color.value);             }
//         }
//
//         Ok(())
//     }
// }
//
// #[derive(Copy, Clone, PartialEq)]
// pub struct Color {
//     value: u32,
//     format: PixelFormat,
// }
//
// impl PixelColor for Color {
//     type Raw = RawU32;
// }
//
// impl<C: RgbColor> From<C> for Color {
//     fn from(value: C) -> Self {
//         Self {
//             value: u32::from_ne_bytes([value.r(), value.g(), value.b(), 0]),
//             format: PixelFormat::Rgb,
//         }
//     }
// }
