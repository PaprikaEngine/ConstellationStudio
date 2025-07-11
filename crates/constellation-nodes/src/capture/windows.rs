/*
 * Constellation Studio - Professional Real-time Video Processing
 * Copyright (c) 2025 MACHIKO LAB
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use super::{ScreenCaptureBackend, WindowCaptureBackend, WindowInfo};
#[cfg(target_os = "windows")]
use anyhow::Result;
use constellation_core::{VideoFormat, VideoFrame};

use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{Direct3D11::*, Dxgi::Common::*, Dxgi::*, Gdi::*},
        System::Com::*,
        UI::WindowsAndMessaging::*,
    },
};

pub struct WindowsScreenCapture {
    display_id: u32,
    capture_cursor: bool,
    width: u32,
    height: u32,
    hdc: Option<HDC>,
    mem_dc: Option<HDC>,
    bitmap: Option<HBITMAP>,
}

impl ScreenCaptureBackend for WindowsScreenCapture {
    fn new(display_id: u32, capture_cursor: bool) -> Result<Self> {
        let (width, height) = get_display_dimensions(display_id)?;

        Ok(Self {
            display_id,
            capture_cursor,
            width,
            height,
            hdc: None,
            mem_dc: None,
            bitmap: None,
        })
    }

    fn capture_frame(&mut self) -> Result<VideoFrame> {
        self.initialize_capture_context()?;

        // Use BitBlt for now (legacy but reliable method)
        let frame_data = self.capture_with_bitblt()?;

        Ok(VideoFrame {
            width: self.width,
            height: self.height,
            format: VideoFormat::Bgra8,
            data: frame_data,
        })
    }

    fn get_display_count() -> Result<u32> {
        get_display_count()
    }

    fn get_display_bounds(&self, display_id: u32) -> Result<(u32, u32, u32, u32)> {
        // Get display bounds for specified display
        let mut display_index = 0;
        let mut bounds = (0, 0, 0, 0);

        unsafe {
            let mut enum_context = DisplayEnumContext {
                target_index: display_id,
                current_index: 0,
                found_bounds: None,
            };

            EnumDisplayMonitors(
                None,
                None,
                Some(enum_display_proc),
                LPARAM(&mut enum_context as *mut _ as isize),
            );

            if let Some(found_bounds) = enum_context.found_bounds {
                bounds = found_bounds;
            }
        }

        Ok(bounds)
    }
}

impl WindowsScreenCapture {
    fn initialize_capture_context(&mut self) -> Result<()> {
        if self.hdc.is_some() {
            return Ok(());
        }

        unsafe {
            let hdc = GetDC(None);
            let mem_dc = CreateCompatibleDC(hdc);
            let bitmap = CreateCompatibleBitmap(hdc, self.width as i32, self.height as i32);

            if hdc.is_invalid() || mem_dc.is_invalid() || bitmap.is_invalid() {
                return Err(anyhow::anyhow!("Failed to create capture context"));
            }

            SelectObject(mem_dc, bitmap);

            self.hdc = Some(hdc);
            self.mem_dc = Some(mem_dc);
            self.bitmap = Some(bitmap);
        }

        Ok(())
    }

    fn capture_with_bitblt(&mut self) -> Result<Vec<u8>> {
        let hdc = self
            .hdc
            .ok_or_else(|| anyhow::anyhow!("HDC not initialized"))?;
        let mem_dc = self
            .mem_dc
            .ok_or_else(|| anyhow::anyhow!("Memory DC not initialized"))?;

        unsafe {
            // Copy screen to memory DC
            if !BitBlt(
                mem_dc,
                0,
                0,
                self.width as i32,
                self.height as i32,
                hdc,
                0,
                0,
                SRCCOPY,
            )
            .as_bool()
            {
                return Err(anyhow::anyhow!("BitBlt failed"));
            }

            // Get bitmap data
            let bitmap = self
                .bitmap
                .ok_or_else(|| anyhow::anyhow!("Bitmap not initialized"))?;
            let mut bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: self.width as i32,
                    biHeight: -(self.height as i32), // Top-down DIB
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default(); 1],
            };

            let mut buffer = vec![0u8; (self.width * self.height * 4) as usize];

            let lines = GetDIBits(
                hdc,
                bitmap,
                0,
                self.height,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            );

            if lines == 0 {
                return Err(anyhow::anyhow!("GetDIBits failed"));
            }

            Ok(buffer)
        }
    }
}

impl Drop for WindowsScreenCapture {
    fn drop(&mut self) {
        unsafe {
            if let Some(bitmap) = self.bitmap {
                DeleteObject(bitmap);
            }
            if let Some(mem_dc) = self.mem_dc {
                DeleteDC(mem_dc);
            }
            if let Some(hdc) = self.hdc {
                ReleaseDC(None, hdc);
            }
        }
    }
}

pub struct WindowsWindowCapture {
    window_handle: HWND,
    width: u32,
    height: u32,
    hdc: Option<HDC>,
    mem_dc: Option<HDC>,
    bitmap: Option<HBITMAP>,
}

impl WindowCaptureBackend for WindowsWindowCapture {
    fn new(window_id: u64) -> Result<Self> {
        let window_handle = HWND(window_id as isize);
        let (width, height) = get_window_dimensions(window_handle)?;

        Ok(Self {
            window_handle,
            width,
            height,
            hdc: None,
            mem_dc: None,
            bitmap: None,
        })
    }

    fn new_by_title(title: &str) -> Result<Self> {
        let window_handle = find_window_by_title(title)?;
        let (width, height) = get_window_dimensions(window_handle)?;

        Ok(Self {
            window_handle,
            width,
            height,
            hdc: None,
            mem_dc: None,
            bitmap: None,
        })
    }

    fn capture_frame(&mut self) -> Result<VideoFrame> {
        self.initialize_capture_context()?;
        let frame_data = self.capture_window()?;

        Ok(VideoFrame {
            width: self.width,
            height: self.height,
            format: VideoFormat::Bgra8,
            data: frame_data,
        })
    }

    fn get_window_list() -> Result<Vec<WindowInfo>> {
        get_window_list()
    }

    fn get_window_bounds(&self) -> Result<(u32, u32, u32, u32)> {
        get_window_bounds(self.window_handle)
    }
}

impl WindowsWindowCapture {
    fn initialize_capture_context(&mut self) -> Result<()> {
        if self.hdc.is_some() {
            return Ok(());
        }

        unsafe {
            let window_dc = GetWindowDC(self.window_handle);
            let mem_dc = CreateCompatibleDC(window_dc);
            let bitmap = CreateCompatibleBitmap(window_dc, self.width as i32, self.height as i32);

            if window_dc.is_invalid() || mem_dc.is_invalid() || bitmap.is_invalid() {
                return Err(anyhow::anyhow!("Failed to create window capture context"));
            }

            SelectObject(mem_dc, bitmap);

            self.hdc = Some(window_dc);
            self.mem_dc = Some(mem_dc);
            self.bitmap = Some(bitmap);
        }

        Ok(())
    }

    fn capture_window(&mut self) -> Result<Vec<u8>> {
        let window_dc = self
            .hdc
            .ok_or_else(|| anyhow::anyhow!("Window DC not initialized"))?;
        let mem_dc = self
            .mem_dc
            .ok_or_else(|| anyhow::anyhow!("Memory DC not initialized"))?;

        unsafe {
            // Use PrintWindow for better compatibility with modern applications
            let bitmap = self
                .bitmap
                .ok_or_else(|| anyhow::anyhow!("Bitmap not initialized"))?;

            if !PrintWindow(self.window_handle, mem_dc, PRINT_WINDOW_FLAGS(0)).as_bool() {
                // Fallback to BitBlt if PrintWindow fails
                if !BitBlt(
                    mem_dc,
                    0,
                    0,
                    self.width as i32,
                    self.height as i32,
                    window_dc,
                    0,
                    0,
                    SRCCOPY,
                )
                .as_bool()
                {
                    return Err(anyhow::anyhow!("Both PrintWindow and BitBlt failed"));
                }
            }

            // Extract bitmap data
            let mut bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: self.width as i32,
                    biHeight: -(self.height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default(); 1],
            };

            let mut buffer = vec![0u8; (self.width * self.height * 4) as usize];

            let lines = GetDIBits(
                window_dc,
                bitmap,
                0,
                self.height,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bitmap_info,
                DIB_RGB_COLORS,
            );

            if lines == 0 {
                return Err(anyhow::anyhow!("GetDIBits failed for window capture"));
            }

            Ok(buffer)
        }
    }
}

impl Drop for WindowsWindowCapture {
    fn drop(&mut self) {
        unsafe {
            if let Some(bitmap) = self.bitmap {
                DeleteObject(bitmap);
            }
            if let Some(mem_dc) = self.mem_dc {
                DeleteDC(mem_dc);
            }
            if let Some(hdc) = self.hdc {
                ReleaseDC(self.window_handle, hdc);
            }
        }
    }
}

// Helper functions
fn get_display_count() -> Result<u32> {
    unsafe {
        let count = GetSystemMetrics(SM_CMONITORS);
        if count > 0 {
            Ok(count as u32)
        } else {
            Ok(1) // Fallback to single display
        }
    }
}

fn get_display_dimensions(display_id: u32) -> Result<(u32, u32)> {
    unsafe {
        if display_id == 0 {
            // Primary display
            let width = GetSystemMetrics(SM_CXSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYSCREEN) as u32;
            Ok((width, height))
        } else {
            // Secondary displays - simplified for now
            Ok((1920, 1080)) // TODO: Implement proper multi-monitor support
        }
    }
}

fn get_window_dimensions(hwnd: HWND) -> Result<(u32, u32)> {
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).as_bool() {
            let width = (rect.right - rect.left) as u32;
            let height = (rect.bottom - rect.top) as u32;
            Ok((width, height))
        } else {
            Err(anyhow::anyhow!("Failed to get window dimensions"))
        }
    }
}

fn get_window_bounds(hwnd: HWND) -> Result<(u32, u32, u32, u32)> {
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).as_bool() {
            Ok((
                rect.left as u32,
                rect.top as u32,
                (rect.right - rect.left) as u32,
                (rect.bottom - rect.top) as u32,
            ))
        } else {
            Err(anyhow::anyhow!("Failed to get window bounds"))
        }
    }
}

fn find_window_by_title(title: &str) -> Result<HWND> {
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let hwnd = FindWindowW(None, PCWSTR(title_wide.as_ptr()));
        if !hwnd.is_invalid() {
            Ok(hwnd)
        } else {
            Err(anyhow::anyhow!("Window not found: {}", title))
        }
    }
}

struct DisplayEnumContext {
    target_index: u32,
    current_index: u32,
    found_bounds: Option<(u32, u32, u32, u32)>,
}

unsafe extern "system" fn enum_display_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let context = &mut *(lparam.0 as *mut DisplayEnumContext);

    if context.current_index == context.target_index {
        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
            let rect = monitor_info.rcMonitor;
            context.found_bounds = Some((
                rect.left as u32,
                rect.top as u32,
                (rect.right - rect.left) as u32,
                (rect.bottom - rect.top) as u32,
            ));
        }

        return FALSE; // Stop enumeration
    }

    context.current_index += 1;
    TRUE
}

fn get_window_list() -> Result<Vec<WindowInfo>> {
    let mut windows = Vec::new();

    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut windows as *mut _ as isize),
        )?;
    }

    Ok(windows)
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

    // Skip invisible windows
    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }

    // Get window title
    let mut title_buffer = [0u16; 256];
    let title_len = GetWindowTextW(hwnd, &mut title_buffer);

    if title_len > 0 {
        let title = String::from_utf16_lossy(&title_buffer[..title_len as usize]);

        // Skip windows without meaningful titles
        if !title.is_empty() && title != "Program Manager" {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).as_bool() {
                let window_info = WindowInfo {
                    id: hwnd.0 as u64,
                    title,
                    process_name: "Unknown".to_string(), // TODO: Get actual process name
                    bounds: (
                        rect.left as u32,
                        rect.top as u32,
                        (rect.right - rect.left) as u32,
                        (rect.bottom - rect.top) as u32,
                    ),
                };

                windows.push(window_info);
            }
        }
    }

    TRUE
}
