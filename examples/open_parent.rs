#[cfg(target_os = "macos")]
use baseview::copy_to_clipboard;
use baseview::{
    Event, EventStatus, PhySize, Window, WindowEvent, WindowHandler, WindowScalePolicy,
};
use raw_window_handle_06::DisplayHandle;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;

fn main() {
    let window_open_options = baseview::WindowOpenOptions {
        title: "baseview".into(),
        size: baseview::Size::new(512.0, 512.0),
        scale: WindowScalePolicy::SystemScaleFactor,

        // TODO: Add an example that uses the OpenGL context
        #[cfg(feature = "opengl")]
        gl_config: None,
    };

    Window::open_blocking(window_open_options, MyWindowHandler::new);
}

struct MyWindowHandler<'a> {
    ctx: Context<DisplayHandle<'a>>,
    surface: Surface<DisplayHandle<'a>, raw_window_handle_06::WindowHandle<'a>>,
    current_size: PhySize,
}

impl<'a> MyWindowHandler<'a> {
    pub fn new(window: &mut Window<'a>) -> Self {
        let ctx = Context::new(window.display_handle()).unwrap();
        let surface = Surface::new(&ctx, window.window_handle()).unwrap();

        // TODO: no way to query physical size initially?
        Self { ctx, surface, current_size: PhySize::new(512, 512) }
    }
}

impl<'a> WindowHandler for MyWindowHandler<'a> {
    fn on_frame(&mut self, _window: &mut Window) {
        let mut buf = self.surface.buffer_mut().unwrap();
        buf.fill(0xFFAAAAAA);
        buf.present().unwrap();
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) -> EventStatus {
        match event {
            Event::Mouse(e) => {
                println!("Mouse event: {:?}", e);
            }
            Event::Keyboard(e) => println!("Keyboard event: {:?}", e),
            Event::Window(WindowEvent::Resized(info)) => {
                println!("Parent Resized: {:?}", info);
                let new_size = info.physical_size();
                self.current_size = new_size;

                if let (Some(width), Some(height)) =
                    (NonZeroU32::new(new_size.width), NonZeroU32::new(new_size.height))
                {
                    self.surface.resize(width, height).unwrap();
                }
            }
            Event::Window(e) => println!("Parent Window event: {:?}", e),
        }

        EventStatus::Captured
    }
}
