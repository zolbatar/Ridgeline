use crate::gfx::sdl::Sdl;
use rand::Rng;
use skia_safe::gpu::direct_contexts::make_gl;
use skia_safe::gpu::gl::{FramebufferInfo, Interface};
use skia_safe::gpu::surfaces::wrap_backend_render_target;
use skia_safe::gpu::{ContextOptions, DirectContext};
use skia_safe::image_filters::drop_shadow_only;
use skia_safe::{
    gpu, Canvas, Color, Color4f, Data, Font, FontMgr, ImageFilter, Paint, PaintStyle, Point, Rect, RuntimeEffect, Shader, Surface, Vector,
};

static MAIN_FONT: &[u8] = include_bytes!("assets/NotoSans-Regular.ttf");
const NOISE_SKSL: &str = include_str!("assets/noise.sksl");
const NOISE_MIX: f32 = 0.075;

pub const MIN_ZOOM: f32 = 0.03;
pub const MAX_ZOOM: f32 = 1.0;

pub struct Skia {
    context: DirectContext,
    pub surface: Surface,
    pub font_main: Font,
    pub font_label: Font,
    pub zoom: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub target: Point,
    pub panning: bool,
    pub noise_shader: RuntimeEffect,
    pub _drop_shadow: Option<ImageFilter>,
    pub _drop_shadow_white: Option<ImageFilter>,
}

pub const FONT_SIZE: f32 = 14.0;
pub const LABEL_SIZE: f32 = 10.0;

impl Skia {
    fn make_surface(context: &mut DirectContext, width: i32, height: i32) -> Surface {
        // Get window size and create a Skia surface from the OpenGL framebuffer
        let fb_info = FramebufferInfo {
            fboid: 0,
            format: gl::RGBA8,
            ..Default::default()
        };
        let backend_render_target = gpu::backend_render_targets::make_gl(
            (width, height),
            0, // Sample count
            8, // Stencil bits
            fb_info,
        );

        // Create the Skia surface for rendering
        wrap_backend_render_target(
            context,
            &backend_render_target,
            gpu::SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .expect("Could not create Skia surface")
    }

    pub fn new(sdl: &Sdl) -> Self {
        let interface = Interface::new_native().expect("Can't get GL interface");
        let options = ContextOptions::new();
        let mut context = make_gl(&interface, &options).expect("Can't create Skia context");

        // Fonts
        let font_mgr = FontMgr::new();

        // Shaders
        let noise_shader = RuntimeEffect::make_for_shader(NOISE_SKSL, None).expect("Failed to make runtime effect");

        // Filters
        let drop_shadow = drop_shadow_only(Vector::new(1.5, 1.5), (0.5, 0.5), Color::from_argb(64, 0, 0, 0), None, None, None);
        let drop_shadow_white = drop_shadow_only(Vector::new(1.5, 1.5), (2.0, 2.0), Color::from_argb(64, 255, 255, 255), None, None, None);

        // Surface
        let surface = Skia::make_surface(&mut context, (sdl.width as f32 * sdl.dpi) as i32, (sdl.height as f32 * sdl.dpi) as i32);

        let mut skia = Skia {
            context,
            surface,
            font_main: Font::from_typeface(font_mgr.new_from_data(MAIN_FONT, None).unwrap(), FONT_SIZE),
            font_label: Font::from_typeface(font_mgr.new_from_data(MAIN_FONT, None).unwrap(), LABEL_SIZE),
            zoom: 0.03,
            zoom_min: MIN_ZOOM,
            zoom_max: MAX_ZOOM,
            target: Point::new(0.0, -5000.0),
            panning: false,
            noise_shader,
            _drop_shadow: drop_shadow,
            _drop_shadow_white: drop_shadow_white,
        };

        unsafe {
            skia.flush();
        }

        skia
    }

    pub fn _test(&mut self, sdl: &Sdl) {
        let canvas = self.get_canvas();
        let mut rng = rand::rng();
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::WHITE);
        paint.set_style(PaintStyle::Stroke);
        for _ in 1..=10000 {
            canvas.draw_line(
                Point {
                    x: rng.random_range(0..=sdl.width) as f32,
                    y: rng.random_range(0..=sdl.height) as f32,
                },
                Point {
                    x: rng.random_range(0..=sdl.width) as f32,
                    y: rng.random_range(0..=sdl.height) as f32,
                },
                &paint,
            );
        }
    }

    pub fn get_canvas(&mut self) -> &Canvas {
        self.surface.canvas()
    }

    pub unsafe fn flush(&mut self) {
        self.surface.image_snapshot();
        self.context.flush_and_submit();

        // Clear
        let w = self.surface.width();
        let h = self.surface.height();
        self.get_canvas().clear(Color::TRANSPARENT);
        let mut paint_background = Paint::default();
        let bg = Color::from_rgb(0x08, 0x1A, 0x30); // Deep Trench Blue
//        let bg = Color::from_rgb(0xE0, 0xE0, 0xE0);
        //        let bg = Color::from_rgb(0x0, 0x0, 0x0);
        paint_background.set_style(PaintStyle::Fill);
        paint_background.set_shader(self.create_noise_shader(bg, NOISE_MIX));
        self.get_canvas().draw_rect(Rect::from_xywh(0.0, 0.0, w as f32, h as f32), &paint_background);
    }

    pub fn set_matrix(&mut self, gfx: &Sdl) {
        let canvas = self.get_canvas();
        canvas.reset_matrix();
        canvas.scale((gfx.dpi, gfx.dpi));
    }

    pub fn set_zoom_target(&mut self, gfx: &Sdl) {
        let zoom = self.zoom;
        let target = self.target;
        let canvas = self.get_canvas();
        canvas.translate(gfx.centre);
        canvas.scale((zoom, zoom));
        canvas.translate((-target.x, -target.y));
    }

    pub fn _clear_matrix(&mut self) {
        let canvas = self.get_canvas();
        canvas.restore();
    }

    pub fn _reset_context(&mut self) {
        self.context.reset(None);
    }

    pub fn create_noise_shader(&mut self, base_color: Color, mix: f32) -> Shader {
        let uniforms = {
            let mut data = vec![];

            // Mix
            data.extend_from_slice(&mix.to_ne_bytes());

            // Colour
            let d = Color4f::from(base_color).as_array().iter().map(|&f| f.to_ne_bytes()).flatten().collect::<Vec<_>>();
            data.extend_from_slice(&d);

            Data::new_copy(&data)
        };
        self.noise_shader.clone().make_shader(uniforms, &[], None).expect("Make shader failed")
    }
}
