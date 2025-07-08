use super::{ScreenCaptureBackend, WindowCaptureBackend, WindowInfo};
#[cfg(target_os = "linux")]
use anyhow::Result;
use constellation_core::{VideoFormat, VideoFrame};

use std::ffi::CString;
use std::ptr;

// X11 bindings
use x11::xlib::{
    Display, Window, XCloseDisplay, XDefaultRootWindow, XDefaultScreen, XDestroyImage,
    XDisplayHeight, XDisplayWidth, XFetchName, XFree, XFreeStringList, XGetImage,
    XGetWindowAttributes, XGetWindowProperty, XImage, XOpenDisplay, XQueryTree, XTextProperty,
    XWindowAttributes, Xutf8TextPropertyToTextList, ZPixmap,
};

pub struct LinuxScreenCapture {
    display_id: u32,
    capture_cursor: bool,
    width: u32,
    height: u32,
    display: *mut Display,
    root_window: Window,
}

impl ScreenCaptureBackend for LinuxScreenCapture {
    fn new(display_id: u32, capture_cursor: bool) -> Result<Self> {
        unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                return Err(anyhow::anyhow!("Failed to open X11 display"));
            }

            let screen = XDefaultScreen(display);
            let root_window = XDefaultRootWindow(display);
            let width = XDisplayWidth(display, screen) as u32;
            let height = XDisplayHeight(display, screen) as u32;

            Ok(Self {
                display_id,
                capture_cursor,
                width,
                height,
                display,
                root_window,
            })
        }
    }

    fn capture_frame(&mut self) -> Result<VideoFrame> {
        unsafe {
            let image = XGetImage(
                self.display,
                self.root_window,
                0,
                0,
                self.width,
                self.height,
                AllPlanes,
                ZPixmap,
            );

            if image.is_null() {
                return Err(anyhow::anyhow!("Failed to capture X11 screen image"));
            }

            let frame_data = self.convert_ximage_to_frame_data(image)?;

            XDestroyImage(image);

            Ok(VideoFrame {
                width: self.width,
                height: self.height,
                format: VideoFormat::Bgra8,
                data: frame_data,
            })
        }
    }

    fn get_display_count() -> Result<u32> {
        // For simplicity, return 1 for now
        // Real implementation would check DISPLAY environment variable
        // and enumerate available X11 displays
        Ok(1)
    }

    fn get_display_bounds(&self, _display_id: u32) -> Result<(u32, u32, u32, u32)> {
        Ok((0, 0, self.width, self.height))
    }
}

impl LinuxScreenCapture {
    fn convert_ximage_to_frame_data(&self, image: *mut XImage) -> Result<Vec<u8>> {
        unsafe {
            let image_ref = &*image;
            let width = image_ref.width as u32;
            let height = image_ref.height as u32;
            let bytes_per_pixel = (image_ref.bits_per_pixel / 8) as usize;

            if bytes_per_pixel != 4 {
                return Err(anyhow::anyhow!(
                    "Unsupported pixel format: {} bits per pixel",
                    image_ref.bits_per_pixel
                ));
            }

            let data_size = (width * height * 4) as usize;
            let mut frame_data = Vec::with_capacity(data_size);

            let src_data = std::slice::from_raw_parts(
                image_ref.data as *const u8,
                (height as usize) * (image_ref.bytes_per_line as usize),
            );

            // Copy and convert pixel format if needed
            for y in 0..height {
                let src_row_offset = (y as usize) * (image_ref.bytes_per_line as usize);
                for x in 0..width {
                    let src_pixel_offset = src_row_offset + (x as usize) * bytes_per_pixel;

                    if src_pixel_offset + 3 < src_data.len() {
                        // X11 typically uses BGRA format
                        let b = src_data[src_pixel_offset];
                        let g = src_data[src_pixel_offset + 1];
                        let r = src_data[src_pixel_offset + 2];
                        let a = src_data[src_pixel_offset + 3];

                        // Convert to BGRA format expected by our system
                        frame_data.push(b);
                        frame_data.push(g);
                        frame_data.push(r);
                        frame_data.push(a);
                    }
                }
            }

            Ok(frame_data)
        }
    }
}

impl Drop for LinuxScreenCapture {
    fn drop(&mut self) {
        unsafe {
            if !self.display.is_null() {
                XCloseDisplay(self.display);
            }
        }
    }
}

pub struct LinuxWindowCapture {
    window_id: Window,
    width: u32,
    height: u32,
    display: *mut Display,
}

impl WindowCaptureBackend for LinuxWindowCapture {
    fn new(window_id: u64) -> Result<Self> {
        unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                return Err(anyhow::anyhow!(
                    "Failed to open X11 display for window capture"
                ));
            }

            let window = window_id as Window;
            let (width, height) = get_window_dimensions(display, window)?;

            Ok(Self {
                window_id: window,
                width,
                height,
                display,
            })
        }
    }

    fn new_by_title(title: &str) -> Result<Self> {
        let window_id = find_window_by_title(title)?;
        Self::new(window_id)
    }

    fn capture_frame(&mut self) -> Result<VideoFrame> {
        unsafe {
            let image = XGetImage(
                self.display,
                self.window_id,
                0,
                0,
                self.width,
                self.height,
                AllPlanes,
                ZPixmap,
            );

            if image.is_null() {
                return Err(anyhow::anyhow!("Failed to capture X11 window image"));
            }

            let frame_data = self.convert_ximage_to_frame_data(image)?;

            XDestroyImage(image);

            Ok(VideoFrame {
                width: self.width,
                height: self.height,
                format: VideoFormat::Bgra8,
                data: frame_data,
            })
        }
    }

    fn get_window_list() -> Result<Vec<WindowInfo>> {
        get_window_list()
    }

    fn get_window_bounds(&self) -> Result<(u32, u32, u32, u32)> {
        get_window_bounds(self.display, self.window_id)
    }
}

impl LinuxWindowCapture {
    fn convert_ximage_to_frame_data(&self, image: *mut XImage) -> Result<Vec<u8>> {
        unsafe {
            let image_ref = &*image;
            let width = image_ref.width as u32;
            let height = image_ref.height as u32;
            let bytes_per_pixel = (image_ref.bits_per_pixel / 8) as usize;

            if bytes_per_pixel != 4 {
                return Err(anyhow::anyhow!(
                    "Unsupported window pixel format: {} bits per pixel",
                    image_ref.bits_per_pixel
                ));
            }

            let data_size = (width * height * 4) as usize;
            let mut frame_data = Vec::with_capacity(data_size);

            let src_data = std::slice::from_raw_parts(
                image_ref.data as *const u8,
                (height as usize) * (image_ref.bytes_per_line as usize),
            );

            for y in 0..height {
                let src_row_offset = (y as usize) * (image_ref.bytes_per_line as usize);
                for x in 0..width {
                    let src_pixel_offset = src_row_offset + (x as usize) * bytes_per_pixel;

                    if src_pixel_offset + 3 < src_data.len() {
                        let b = src_data[src_pixel_offset];
                        let g = src_data[src_pixel_offset + 1];
                        let r = src_data[src_pixel_offset + 2];
                        let a = src_data[src_pixel_offset + 3];

                        frame_data.push(b);
                        frame_data.push(g);
                        frame_data.push(r);
                        frame_data.push(a);
                    }
                }
            }

            Ok(frame_data)
        }
    }
}

impl Drop for LinuxWindowCapture {
    fn drop(&mut self) {
        unsafe {
            if !self.display.is_null() {
                XCloseDisplay(self.display);
            }
        }
    }
}

// Helper functions
fn get_window_dimensions(display: *mut Display, window: Window) -> Result<(u32, u32)> {
    unsafe {
        let mut attrs = XWindowAttributes {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            border_width: 0,
            depth: 0,
            visual: ptr::null_mut(),
            root: 0,
            class: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: 0,
            colormap: 0,
            map_installed: 0,
            map_state: 0,
            all_event_masks: 0,
            your_event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: 0,
            screen: ptr::null_mut(),
        };

        let result = XGetWindowAttributes(display, window, &mut attrs);
        if result == 0 {
            return Err(anyhow::anyhow!("Failed to get window attributes"));
        }

        Ok((attrs.width as u32, attrs.height as u32))
    }
}

fn get_window_bounds(display: *mut Display, window: Window) -> Result<(u32, u32, u32, u32)> {
    unsafe {
        let mut attrs = XWindowAttributes {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            border_width: 0,
            depth: 0,
            visual: ptr::null_mut(),
            root: 0,
            class: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: 0,
            colormap: 0,
            map_installed: 0,
            map_state: 0,
            all_event_masks: 0,
            your_event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: 0,
            screen: ptr::null_mut(),
        };

        let result = XGetWindowAttributes(display, window, &mut attrs);
        if result == 0 {
            return Err(anyhow::anyhow!("Failed to get window bounds"));
        }

        Ok((
            attrs.x as u32,
            attrs.y as u32,
            attrs.width as u32,
            attrs.height as u32,
        ))
    }
}

fn find_window_by_title(title: &str) -> Result<u64> {
    let window_list = get_window_list_impl()?;

    for window in window_list {
        if window.title == title {
            return Ok(window.id);
        }
    }

    Err(anyhow::anyhow!("Window not found: {}", title))
}

fn get_window_list_impl() -> Result<Vec<WindowInfo>> {
    unsafe {
        let display = XOpenDisplay(ptr::null());
        if display.is_null() {
            return Err(anyhow::anyhow!(
                "Failed to open X11 display for window enumeration"
            ));
        }

        let root = XDefaultRootWindow(display);
        let mut windows = Vec::new();

        enumerate_windows(display, root, &mut windows)?;

        XCloseDisplay(display);
        Ok(windows)
    }
}

fn enumerate_windows(
    display: *mut Display,
    window: Window,
    windows: &mut Vec<WindowInfo>,
) -> Result<()> {
    unsafe {
        let mut root_return = 0;
        let mut parent_return = 0;
        let mut children_return = ptr::null_mut();
        let mut nchildren_return = 0;

        let result = XQueryTree(
            display,
            window,
            &mut root_return,
            &mut parent_return,
            &mut children_return,
            &mut nchildren_return,
        );

        if result == 0 {
            return Ok(());
        }

        if !children_return.is_null() && nchildren_return > 0 {
            let children = std::slice::from_raw_parts(children_return, nchildren_return as usize);

            for &child in children {
                // Get window title
                let mut name_ptr = ptr::null_mut();
                let title = if XFetchName(display, child, &mut name_ptr) != 0 && !name_ptr.is_null()
                {
                    let c_str = std::ffi::CStr::from_ptr(name_ptr);
                    let title = c_str.to_string_lossy().into_owned();
                    XFree(name_ptr as *mut _);
                    title
                } else {
                    String::new()
                };

                // Only include windows with titles
                if !title.is_empty() {
                    let bounds = get_window_bounds(display, child).unwrap_or((0, 0, 0, 0));

                    windows.push(WindowInfo {
                        id: child as u64,
                        title,
                        process_name: "Unknown".to_string(), // TODO: Get actual process name
                        bounds,
                    });
                }

                // Recursively enumerate child windows
                let _ = enumerate_windows(display, child, windows);
            }

            XFree(children_return as *mut _);
        }

        Ok(())
    }
}

pub fn get_display_count() -> Result<u32> {
    // For X11, typically there's one display per DISPLAY environment variable
    // Real implementation would parse DISPLAY and check for multiple screens
    Ok(1)
}

pub fn get_window_list() -> Result<Vec<WindowInfo>> {
    get_window_list_impl()
}
