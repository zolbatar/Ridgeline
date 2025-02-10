use crate::gfx::skia::Skia;
use sdl2::video::{GLContext, GLProfile, Window};
use sdl2::{EventPump, VideoSubsystem};
use skia_safe::utils::text_utils::Align;
use skia_safe::{Paint, PaintStyle, Point, Vector};
use std::time::{Duration, Instant};

const WINDOW_WIDTH: u32 = 1400;
const WINDOW_HEIGHT: u32 = 800;

pub struct Sdl {
    window: Window,
    _video: VideoSubsystem,
    _gl_context: GLContext,
    pub width: u32,
    pub height: u32,
    pub centre: Vector,
    pub dpi: f32,
    pub fps: f64,
    pub event_loop: EventPump,
    pub frame_count: u64,
    last_fps_check: Instant,
    check_interval: Duration,
}

impl Sdl {
    pub fn new() -> Sdl {
        // Initialize SDL2
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        // Set OpenGL attributes
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3); // OpenGL 3.3

        // Create an SDL2 window
        let window = video
            .window("Ridgeline", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .opengl()
            .allow_highdpi()
            .build()
            .expect("Unable to create SDL window");

        // Create an OpenGL context
        let gl_context = window.gl_create_context().expect("Failed to create OpenGL context");
        window.gl_make_current(&gl_context).expect("Failed to set OpenGL context");

        // Load OpenGL functions
        gl::load_with(|s| video.gl_get_proc_address(s) as *const _);

        // Get display index (typically 0 is the default display)
        let display_index = 0;

        // Get actual High-DPI scaling (drawable size)
        let (drawable_width, drawable_height) = window.drawable_size();
        let dpi = drawable_width as f32 / WINDOW_WIDTH as f32;

        Self {
            dpi,
            window,
            _video: video,
            _gl_context: gl_context,
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            centre: Vector::new(WINDOW_WIDTH as f32 / 2.0f32, WINDOW_HEIGHT as f32 / 2.0f32),
            fps: 0.0,
            event_loop: sdl.event_pump().unwrap(),
            frame_count: 0,
            last_fps_check: Instant::now(),
            check_interval: Duration::from_secs(1), // Check FPS every second
        }
    }

    pub fn frame_start(&mut self) {
        // Measure the time it took to render the previous frame
        let current_time = Instant::now();

        // Increment the frame count
        self.frame_count += 1;

        // Calculate FPS every second
        if current_time - self.last_fps_check >= self.check_interval {
            self.fps = self.frame_count as f64 / (self.check_interval.as_secs_f64());

            // Reset frame count and last FPS check time
            self.frame_count = 0;
            self.last_fps_check = current_time;
        }
    }

    pub fn show_fps(&self, skia: &mut Skia) {
        let fps = format!("FPS: {:.0} Zoom: {}, Position: {},{}", self.fps, skia.zoom, skia.target.x, skia.target.y);
        let mut paint = Paint::default();
        paint.set_style(PaintStyle::StrokeAndFill);
        paint.set_color(skia_safe::Color::YELLOW);
        let canvas = skia.surface.canvas();
        canvas.draw_text_align(fps, Point::new(10.0, 30.0), &skia.font_main, &paint, Align::Left);
    }

    pub fn frame_end(&self) {
        self.window.gl_swap_window();
    }
}
