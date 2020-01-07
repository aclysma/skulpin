//! OS-specific code required to get a surface for our swapchain

use ash::extensions::{ext::DebugReport, khr::Surface};
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;

//
// Code for creating surfaces
//
#[allow(clippy::missing_safety_doc)]
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    raw_window_handle: &raw_window_handle::RawWindowHandle,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use ash::extensions::khr::XlibSurface;
    use ash::extensions::khr::XcbSurface;
    use ash::extensions::khr::WaylandSurface;

    match raw_window_handle {
        raw_window_handle::RawWindowHandle::Xlib(window_handle) => {
            let x11_display = window_handle.display;
            let x11_window = window_handle.window;

            let x11_create_info = vk::XlibSurfaceCreateInfoKHR::builder()
                .window(x11_window)
                .dpy(x11_display as *mut vk::Display);

            let xlib_surface_loader = XlibSurface::new(entry, instance);
            xlib_surface_loader.create_xlib_surface(&x11_create_info, None)
        }
        raw_window_handle::RawWindowHandle::Xcb(window_handle) => {
            let window = window_handle.window;
            let connection = window_handle.connection;

            let xcb_create_info = vk::XcbSurfaceCreateInfoKHR::builder()
                .window(window)
                .connection(connection as *mut vk::xcb_connection_t);

            let xcb_surface_loader = XcbSurface::new(entry, instance);
            xcb_surface_loader.create_xcb_surface(&xcb_create_info, None)
        }
        raw_window_handle::RawWindowHandle::Wayland(window_handle) => {
            let surface = window_handle.surface;
            let display = window_handle.display;

            let wayland_create_info = vk::WaylandSurfaceCreateInfoKHR::builder()
                .surface(surface)
                .display(display);

            let wayland_surface_loader = WaylandSurface::new(entry, instance);
            wayland_surface_loader.create_wayland_surface(&wayland_create_info, None)
        }
        _ => unimplemented!(),
    }
}

#[allow(clippy::missing_safety_doc)]
#[cfg(target_os = "macos")]
pub unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    raw_window_handle: &raw_window_handle::RawWindowHandle,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use ash::extensions::mvk::MacOSSurface;
    use cocoa::appkit::{NSView, NSWindow};
    use cocoa::base::id as cocoa_id;
    use metal::CoreAnimationLayer;
    use objc::runtime::YES;

    match raw_window_handle {
        raw_window_handle::RawWindowHandle::MacOS(window_handle) => {
            use std::ptr;

            let wnd: cocoa_id = window_handle.ns_window as *mut objc::runtime::Object;

            let layer = CoreAnimationLayer::new();

            layer.set_edge_antialiasing_mask(0);
            layer.set_presents_with_transaction(false);
            layer.remove_all_animations();

            let view = wnd.contentView();

            layer.set_contents_scale(view.backingScaleFactor());
            view.setLayer(
                layer.as_ref() as *const metal::CoreAnimationLayerRef as *mut objc::runtime::Object
            );
            view.setWantsLayer(YES);

            let create_info = vk::MacOSSurfaceCreateInfoMVK {
                s_type: vk::StructureType::MACOS_SURFACE_CREATE_INFO_M,
                p_next: ptr::null(),
                flags: Default::default(),
                p_view: window_handle.ns_view as *const std::os::raw::c_void,
            };

            let macos_surface_loader = MacOSSurface::new(entry, instance);
            macos_surface_loader.create_mac_os_surface_mvk(&create_info, None)
        }
        _ => unimplemented!(),
    }
}

#[allow(clippy::missing_safety_doc)]
#[cfg(target_os = "windows")]
pub unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    raw_window_handle: &raw_window_handle::RawWindowHandle,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use ash::extensions::khr::Win32Surface;

    match raw_window_handle {
        raw_window_handle::RawWindowHandle::Windows(window_handle) => {
            let hwnd = window_handle.hwnd;
            let hinstance = window_handle.hinstance;

            let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
                s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
                p_next: std::ptr::null(),
                flags: Default::default(),
                hinstance,
                hwnd: hwnd as *const std::os::raw::c_void,
            };

            let win32_surface_loader = Win32Surface::new(entry, instance);
            win32_surface_loader.create_win32_surface(&win32_create_info, None)
        }
        _ => unimplemented!(),
    }
}

//
// Extensions we want to use for each platform
//

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn extension_names(raw_window_handle: &raw_window_handle::RawWindowHandle) -> Vec<*const i8> {
    use ash::extensions::khr::XlibSurface;
    use ash::extensions::khr::XcbSurface;
    use ash::extensions::khr::WaylandSurface;

    match raw_window_handle {
        raw_window_handle::RawWindowHandle::Xlib(_) => vec![
            Surface::name().as_ptr(),
            XlibSurface::name().as_ptr(),
            DebugReport::name().as_ptr(),
        ],
        raw_window_handle::RawWindowHandle::Xcb(_) => vec![
            Surface::name().as_ptr(),
            XcbSurface::name().as_ptr(),
            DebugReport::name().as_ptr(),
        ],
        raw_window_handle::RawWindowHandle::Wayland(_) => vec![
            Surface::name().as_ptr(),
            WaylandSurface::name().as_ptr(),
            DebugReport::name().as_ptr(),
        ],
        _ => unimplemented!(),
    }
}

#[cfg(target_os = "macos")]
pub fn extension_names(raw_window_handle: &raw_window_handle::RawWindowHandle) -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(all(windows))]
pub fn extension_names(raw_window_handle: &raw_window_handle::RawWindowHandle) -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}
