# Changelog

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