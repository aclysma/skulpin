# Changelog

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