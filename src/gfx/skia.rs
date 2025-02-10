use crate::gfx::sdl::Sdl;
use rand::Rng;
use skia_safe::gpu::direct_contexts::make_gl;
use skia_safe::gpu::gl::{FramebufferInfo, Interface};
use skia_safe::gpu::surfaces::wrap_backend_render_target;
use skia_safe::gpu::{ContextOptions, DirectContext};
use skia_safe::{gpu, Canvas, Color, Font, FontMgr, Paint, PaintStyle, Point, Surface};

static MAIN_FONT: &[u8] = include_bytes!("assets/NotoSans-Regular.ttf");

pub struct Skia {
    context: DirectContext,
    pub surface: Surface,
    pub font_main: Font,
    pub zoom: f32,
    pub target: Point,
}

pub const FONT_SIZE: f32 = 14.0;

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

        // Surface
        let surface =
            Skia::make_surface(&mut context, (sdl.width as f32 * sdl.dpi) as i32, (sdl.height as f32 * sdl.dpi) as i32);

        let mut skia = Skia {
            context,
            surface,
            font_main: Font::from_typeface(font_mgr.new_from_data(MAIN_FONT, None).unwrap(), FONT_SIZE),
            zoom: 3.0,
            target: Point::new(0.0, 0.0),
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
        self.get_canvas().clear(Color::TRANSPARENT);
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
}
