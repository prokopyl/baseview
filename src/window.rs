use std::marker::PhantomData;

use crate::event::{Event, EventStatus};
use crate::window_open_options::WindowOpenOptions;
use crate::{MouseCursor, Size};

#[cfg(target_os = "macos")]
use crate::macos as platform;
#[cfg(target_os = "windows")]
use crate::win as platform;
#[cfg(target_os = "linux")]
use crate::x11 as platform;

pub struct WindowHandle {
    window_handle: platform::WindowHandle,
    // so that WindowHandle is !Send on all platforms
    phantom: PhantomData<*mut ()>,
}

impl WindowHandle {
    fn new(window_handle: platform::WindowHandle) -> Self {
        Self { window_handle, phantom: PhantomData }
    }

    /// Close the window
    pub fn close(&mut self) {
        self.window_handle.close();
    }

    /// Returns `true` if the window is still open, and returns `false`
    /// if the window was closed/dropped.
    pub fn is_open(&self) -> bool {
        self.window_handle.is_open()
    }
}

pub trait WindowHandler {
    fn on_frame(&mut self, window: &mut Window);
    fn on_event(&mut self, window: &mut Window, event: Event) -> EventStatus;
}

pub struct Window<'a> {
    window: platform::Window<'a>,

    // so that Window is !Send on all platforms
    phantom: PhantomData<*mut ()>,
}

impl<'a> Window<'a> {
    #[cfg(target_os = "windows")]
    pub(crate) fn new(window: platform::Window<'a>) -> Window<'a> {
        Window { window, phantom: PhantomData }
    }

    #[cfg(not(target_os = "windows"))]
    pub(crate) fn new(window: platform::Window) -> Window {
        Window { window, phantom: PhantomData }
    }

    pub fn open_blocking<H, B>(options: WindowOpenOptions, build: B)
    where
        H: WindowHandler + 'static,
        B: FnOnce(&mut Window) -> H,
        B: Send + 'static,
    {
        platform::Window::open_blocking::<H, B>(options, build)
    }

    /// Close the window
    pub fn close(&mut self) {
        self.window.close();
    }

    /// Resize the window to the given size. The size is always in logical pixels. DPI scaling will
    /// automatically be accounted for.
    pub fn resize(&mut self, size: Size) {
        self.window.resize(size);
    }

    pub fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        self.window.set_mouse_cursor(cursor);
    }

    /// If provided, then an OpenGL context will be created for this window. You'll be able to
    /// access this context through [crate::Window::gl_context].
    #[cfg(feature = "opengl")]
    pub fn gl_context(&self) -> Option<&crate::gl::GlContext> {
        self.window.gl_context()
    }
}

#[cfg(feature = "raw-window-handle_05")]
const _: () = {
    use raw_window_handle_05::{
        HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
    };

    #[cfg(not(feature = "raw-window-handle_06"))]
    impl<'a> Window<'a> {
        pub fn open_parented<P, H, B>(
            parent: &P, options: WindowOpenOptions, build: B,
        ) -> WindowHandle
        where
            P: HasRawWindowHandle,
            H: WindowHandler + 'static,
            B: FnOnce(&mut Window) -> H,
            B: Send + 'static,
        {
            WindowHandle::new(platform::Window::open_parented::<H, B>(
                parent.raw_window_handle(),
                options,
                build,
            ))
        }
    }

    unsafe impl<'a> HasRawWindowHandle for Window<'a> {
        fn raw_window_handle(&self) -> RawWindowHandle {
            self.window.raw_window_handle()
        }
    }

    unsafe impl<'a> HasRawDisplayHandle for Window<'a> {
        fn raw_display_handle(&self) -> RawDisplayHandle {
            self.window.raw_display_handle()
        }
    }

    unsafe impl HasRawWindowHandle for WindowHandle {
        fn raw_window_handle(&self) -> RawWindowHandle {
            self.window_handle.raw_window_handle()
        }
    }
};

#[cfg(feature = "raw-window-handle_06")]
const _: () = {
    use raw_window_handle_06::{
        DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle,
        WindowHandle as RwhWindowHandle,
    };

    impl<'a> Window<'a> {
        pub fn open_parented<P, H, B>(
            parent: &P, options: WindowOpenOptions, build: B,
        ) -> WindowHandle
        where
            P: HasWindowHandle,
            H: WindowHandler + 'static,
            B: FnOnce(&mut Window) -> H,
            B: Send + 'static,
        {
            WindowHandle::new(platform::Window::open_parented::<H, B>(
                parent.window_handle().unwrap(), // TODO: unwrap / document panic
                options,
                build,
            ))
        }
    }

    impl<'a> HasWindowHandle for Window<'a> {
        fn window_handle(&self) -> Result<RwhWindowHandle, HandleError> {
            self.window.window_handle()
        }
    }

    impl<'a> HasDisplayHandle for Window<'a> {
        fn display_handle(&self) -> Result<DisplayHandle, HandleError> {
            self.window.display_handle()
        }
    }

    impl HasWindowHandle for WindowHandle {
        fn window_handle(&self) -> Result<RwhWindowHandle, HandleError> {
            self.window_handle.window_handle()
        }
    }
};
