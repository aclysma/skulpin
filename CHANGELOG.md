# Changelog

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