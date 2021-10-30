# Changelog

## 0.14.1

 * Update to rafx 0.0.14. This fixes an compile error caused by a non-semver change upstream
 * Add winit-25 config flag

## 0.14.0

 * Reuse the same canvas every frame instead of rotating between one per swapchain
 * Update to rafx 0.0.13 to fix a crash when resizing the window on linux

## 0.13.1

 * Fix a crash that occurs when minimizing

## 0.13.0

 * Add vsync config option

## 0.12.0

 * Complete rewrite! Almost all the rendering code has been removed. `skulpin` now uses
   `rafx`. (The vulkan backend in `rafx` was originally from `skulpin` but is much
   more general and user-friendly).
 * The plugin system and included imgui support has been dropped. The intended way to do this
   now is to build your own renderer using `rafx` and use `VkSkiaContext`, `VkSkiaSurface`, and
   coordinate system types directly.
 * Feature names are now consistently kabob-case (i.e. "winit-app")

## 0.11.3

* Limit open-ended version of ash to fix build issue from changes in upstream crates

## 0.11.2

* Add winit-24 feature flag
* Limit some open-ended versions to fix build issues from changes in upstream crates

## 0.11.1

* Bump imgui due to fix build issue from changes in upstream crates

## 0.11.0

 * Refactor the API integrations to use ash_window, simplifying the code and removing a lot of platform-specific code
   from the winit integration.
 * The imgui plugin has been updated to use open-ended versioning. This is published as skulpin-plugin-imgui 0.6.0 but
   does not affect the skulpin release itself as this is a dev-dependency.
 * Added feature flags winit-21, winit-22, winit-23, and winit-latest for easier control of what winit version to pull
   in. This also allows us to have version-specific code (for example to handle upstream changes in winit API)

## 0.10.1

 * Limit some open-ended versions to fix build issues

## 0.10.0

 * Improvements to validation layer handling
     * Search for VK_LAYER_KHRONOS_validation but fall back to VK_LAYER_LUNARG_standard_validation if it's not present
     * Return a failure with a better error message if neither exist
 * Validation is now off by default in the examples
 * Rust minimum version is now 1.43.0
 * skia-safe minimum version is now 0.30.1
 * Remove incorrect gamma correction from imgui examples

## 0.9.5

 * Bump macos dependencies (metal/cocoa)
 * Update to skulpin-plugin-imgui, 0.4.0 bumps imgui support from 0.3 to 0.4
     * This is a dev-dependency only so should not have any downstream effects for users not explicitly using it

## 0.9.4

 * Fix a panic on windows that can occur when using remote desktop or unplugging monitors (Reported in #47)
 
## 0.9.3

 * This was 0.9.2 accidentally republished without intended changes
 
## 0.9.2

 * Fixed cases where the vulkan validation layer not loading when it should have, and loading when it
   shouldn't have.

## 0.9.1

 * Windows-specific fix to determine scaling factor for logical vs. physical coordinates

## 0.9.0

 * Use ash >=0.30 and skia_safe >=0.27. While this release likely builds with 0.26, it has a known issue that can
   cause a crash on MacOS (https://github.com/rust-skia/rust-skia/issues/299)

## 0.8.2
 
 * Build fixes for upstream changes:
     * Require ash 0.29 rather than >=0.29
     * Pin imgui-winit-support to 0.3.0 exactly

## 0.8.1

 * Allow any version of winit >= 0.21

## 0.8.0
 * Add support for injecting command buffers into the renderer. (See `RendererPlugin`)
 * Add an example of using the `RendererPlugin` interface to inject imgui
 * The update and draw functions on skulpin apps now take a single struct containing all parameters rather than each
   parameter individually. This allows adding new parameters without breaking the API, and simplifies it a bit.
 * Mouse wheel support
 * App builder now accepts a custom window title

## 0.7.0
 * Add an abstraction layer so that multiple windowing backends can be supported. Built-in support is provided for
   `sdl2` and `winit`
 * Refactored into multiple crates:
     * Existing rendering code was moved to `skulpin-renderer`
     * Existing `winit` support was moved to `skulpin-renderer-winit`
     * New `sdl2` support was implemented in `skulpin-renderer-sdl2`
     * Existing winit-based app layer was moved to `skulpin-app-winit`
     * `skulpin` crate now pulls in these crates and supports turning them on/off via feature flags

## 0.6.1
 * Fixed an array index out of bound issue that could occur if khr::Swapchain created more images than the specified
   minimum

## 0.6.0
 * Update to winit 0.21

## 0.5.2
 * Implement support for wayland and xcb

## 0.5.1
 * This release pins winit to exactly 0.20.0-alpha6 as winit 0.20 has breaking changes

## 0.5.0
 * Update to 0.20.0-alpha6 (breaking change)

## 0.4.1
 * Limit winit to 0.20.0-alpha4 and 0.20.0-alpha5 due to breaking changes in 0.20.0-alpha6
 * Allow skia_safe to be any version >= 0.21, allowing downstream users to use newer versions of skia bindings without
   requiring the crate to be republished

## 0.4.0
 * Update to skia_safe 0.21 and enable all upstream features by default

## 0.3.0
 * Added support for selecting from among several coordinate systems, and mechanism for turning this off.
 * Error handling is now done via a new callback on AppHandler `fatal_error()`. The app will no longer return, which
   mirrors `winit` functionality
 * Simplification to `TimeState`
 * Removed dependency on several crates (`num-traits`, `strum`)
 * Some internal types used in rendering code were renamed to have a Vk prefix

## 0.2.3
 * Allow configuring PresentMode, PhysicalDeviceType, and vulkan debug layer
 * Swapchain is explicitly rebuilt when the window is resized
 * Improve error handling (more errors are passed up the chain rather than unwrapped)

## 0.2.2
 * Initialize Vulkan to be the highest version that's available. This avoids triggering some validation code in Skia
   (#13, #14, #21)
 * Fixes for queue family handling (#5, #12, #14, #21)
 * On MacOS, switch from the deprecated `metal-rs` crate to the replacement `metal` crate.
 * Add support for choosing between integrated or discrete GPU (or other device types)
 * Add support for choosing between FIFO, MAILBOX, or other presentation modes

## 0.2.1
 * Minimum supported Rust version is 1.36
 * Fix red/blue components being reversed on Windows
 * Fix crash in Windows that occurs when minimizing
 * An image barrier in the Vulkan code had a hardcoded queue family index. This is now properly
   using QUEUE_FAMILY_IGNORED
 * When swapchain is rebuilt, the old swapchain is now passed into vkCreateSwapchainKHR
 * Adjusted some log levels

## 0.2.0
 * Changes prior to 0.2.0 were not tracked.
