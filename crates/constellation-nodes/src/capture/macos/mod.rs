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

// Import the new Screen Capture Kit implementation
mod macos_screen_capture_kit;
use macos_screen_capture_kit::{ScreenCaptureKitCapture, ScreenCaptureKitWindowCapture};

// Modern Screen Capture Kit implementation
pub type MacOSScreenCapture = ScreenCaptureKitCapture;

// Modern Screen Capture Kit window capture implementation
pub type MacOSWindowCapture = ScreenCaptureKitWindowCapture;

// Export helper functions from the Screen Capture Kit implementation
pub use macos_screen_capture_kit::{get_display_count, get_display_list, get_window_list};
