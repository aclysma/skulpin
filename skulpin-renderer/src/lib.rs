#[macro_use]
extern crate log;

use ash::vk;

mod alignment;
use alignment::Align;

pub mod util;

mod window_support;
pub use window_support::Window;

mod instance;
pub use instance::VkInstance;
pub use instance::VkCreateInstanceError;

mod device;
pub use device::VkDevice;
pub use device::VkQueueFamilyIndices;
pub use device::VkQueues;

mod swapchain;
pub use swapchain::VkSwapchain;
pub use swapchain::SwapchainInfo;
pub use swapchain::MAX_FRAMES_IN_FLIGHT;

mod skia_renderpass;
pub use skia_renderpass::VkSkiaRenderPass;

mod skia_support;
pub use skia_support::VkSkiaContext;

mod buffer;
pub use buffer::VkBuffer;

mod debug_reporter;
pub use debug_reporter::VkDebugReporter;

#[allow(clippy::module_inception)]
mod renderer;
pub use renderer::RendererBuilder;
pub use renderer::Renderer;
pub use renderer::CreateRendererError;

mod coordinates;
pub use coordinates::Size;
pub use coordinates::LogicalSize;
pub use coordinates::PhysicalSize;

/// Used to select which PresentMode is preferred. Some of this is hardware/platform dependent and
/// it's a good idea to read the Vulkan spec.
///
/// `Fifo` is always available on Vulkan devices that comply with the spec and is a good default for
/// many cases.
///
/// Values here match VkPresentModeKHR
#[derive(Copy, Clone, Debug)]
pub enum PresentMode {
    /// (`VK_PRESENT_MODE_IMMEDIATE_KHR`) - No internal buffering, and can result in screen
    /// tearin.
    Immediate = 0,

    /// (`VK_PRESENT_MODE_MAILBOX_KHR`) - This allows rendering as fast as the hardware will
    /// allow, but queues the rendered images in a way that avoids tearing. In other words, if the
    /// hardware renders 10 frames within a single vertical blanking period, the first 9 will be
    /// dropped. This is the best choice for lowest latency where power consumption is not a
    /// concern.
    Mailbox = 1,

    /// (`VK_PRESENT_MODE_FIFO_KHR`) - Default option, guaranteed to be available, and locks
    /// screen draw to vsync. This is a good default choice generally, and more power efficient
    /// than mailbox, but can have higher latency than mailbox.
    Fifo = 2,

    /// (`VK_PRESENT_MODE_FIFO_RELAXED_KHR`) - Similar to Fifo but if rendering is late,
    /// screen tearing can be observed.
    FifoRelaxed = 3,
}

impl PresentMode {
    /// Convert to `vk::PresentModeKHR`
    pub fn to_vk(self) -> vk::PresentModeKHR {
        match self {
            PresentMode::Immediate => vk::PresentModeKHR::IMMEDIATE,
            PresentMode::Mailbox => vk::PresentModeKHR::MAILBOX,
            PresentMode::Fifo => vk::PresentModeKHR::FIFO,
            PresentMode::FifoRelaxed => vk::PresentModeKHR::FIFO_RELAXED,
        }
    }
}

/// Used to specify which type of physical device is preferred. It's recommended to read the Vulkan
/// spec to understand precisely what these types mean
///
/// Values here match VkPhysicalDeviceType, DiscreteGpu is the recommended default
#[derive(Copy, Clone, Debug)]
pub enum PhysicalDeviceType {
    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_OTHER`
    Other = 0,

    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU`
    IntegratedGpu = 1,

    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU`
    DiscreteGpu = 2,

    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU`
    VirtualGpu = 3,

    /// Corresponds to `VK_PHYSICAL_DEVICE_TYPE_CPU`
    Cpu = 4,
}

impl PhysicalDeviceType {
    /// Convert to `vk::PhysicalDeviceType`
    pub fn to_vk(self) -> vk::PhysicalDeviceType {
        match self {
            PhysicalDeviceType::Other => vk::PhysicalDeviceType::OTHER,
            PhysicalDeviceType::IntegratedGpu => vk::PhysicalDeviceType::INTEGRATED_GPU,
            PhysicalDeviceType::DiscreteGpu => vk::PhysicalDeviceType::DISCRETE_GPU,
            PhysicalDeviceType::VirtualGpu => vk::PhysicalDeviceType::VIRTUAL_GPU,
            PhysicalDeviceType::Cpu => vk::PhysicalDeviceType::CPU,
        }
    }
}

/// Default coordinate system to use
#[derive(Copy, Clone)]
pub enum CoordinateSystem {
    /// Logical coordinates will use (0,0) top-left and (+X,+Y) right-bottom where X/Y is the logical
    /// size of the window. Logical size applies a multiplier for hi-dpi displays. For example, many
    /// 4K displays would probably have a high-dpi factor of 2.0, simulating a 1080p display.
    Logical,

    /// Physical coordinates will use (0,0) top-left and (+X,+Y) right-bottom where X/Y is the raw
    /// number of pixels.
    Physical,

    /// Visible range allows specifying an arbitrary coordinate system. For example, if you want X to
    /// range (100, 300) and Y to range (-100, 400), you can do that. It's likely you'd want to
    /// determine either X or Y using the aspect ratio to avoid stretching.
    VisibleRange(skia_safe::Rect, skia_safe::matrix::ScaleToFit),

    /// FixedWidth will use the given center position and width, and calculate appropriate Y extents
    /// for the current aspect ratio
    FixedWidth(skia_safe::Point, f32),

    /// Do not modify the canvas matrix
    None,
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        CoordinateSystem::Logical
    }
}

/// Provides a convenient method to set the canvas coordinate system to commonly-desired defaults.
///
/// * Physical coordinates will use (0,0) top-left and (+X,+Y) right-bottom where X/Y is the raw
///   number of pixels.
/// * Logical coordinates will use (0,0) top-left and (+X,+Y) right-bottom where X/Y is the logical
///   size of the window. Logical size applies a multiplier for hi-dpi displays. For example, many
///   4K displays would probably have a high-dpi factor of 2.0, simulating a 1080p display.
/// * Visible range allows specifying an arbitrary coordinate system. For example, if you want X to
///   range (100, 300) and Y to range (-100, 400), you can do that. It's likely you'd want to
///   determine either X or Y using the aspect ratio to avoid stretching.
/// * FixedWidth will use the given center position and width, and calculate appropriate Y extents
///   for the current aspect ratio
/// * See `use_physical_coordinates`, `use_logical_coordinates`, or `use_visible_range` to choose
///   between these options.
///
/// For custom behavior, it's always possible to call `canvas.reset_matrix()` and set up the matrix
/// manually
pub struct CoordinateSystemHelper {
    surface_extents: vk::Extent2D,
    window_logical_size: LogicalSize,
    window_physical_size: PhysicalSize,
}

impl CoordinateSystemHelper {
    /// Create a CoordinateSystemHelper for a window of the given parameters
    pub fn new(
        surface_extents: vk::Extent2D,
        window_logical_size: LogicalSize,
        window_physical_size: PhysicalSize,
    ) -> Self {
        CoordinateSystemHelper {
            surface_extents,
            window_logical_size,
            window_physical_size,
        }
    }

    /// Get the raw pixel size of the surface to which we are drawing
    pub fn surface_extents(&self) -> vk::Extent2D {
        self.surface_extents
    }

    /// Get the logical inner size of the window
    pub fn window_logical_size(&self) -> LogicalSize {
        self.window_logical_size
    }

    /// Get the physical inner size of the window
    pub fn window_physical_size(&self) -> PhysicalSize {
        self.window_physical_size
    }

    /// Use raw pixels for the coordinate system. Top-left is (0, 0), bottom-right is (+X, +Y)
    pub fn use_physical_coordinates(
        &self,
        canvas: &mut skia_safe::Canvas,
    ) {
        // For raw physical pixels, no need to do anything
        canvas.reset_matrix();
    }

    /// Use logical coordinates for the coordinate system. Top-left is (0, 0), bottom-right is
    /// (+X, +Y). Logical size applies a multiplier for hi-dpi displays. For example, many
    ///   4K displays would probably have a high-dpi factor of 2.0, simulating a 1080p display.
    pub fn use_logical_coordinates(
        &self,
        canvas: &mut skia_safe::Canvas,
    ) {
        // To handle hi-dpi displays, we need to compare the logical size of the window with the
        // actual canvas size. Critically, the canvas size won't necessarily be the size of the
        // window in physical pixels.
        let scale = (
            (f64::from(self.surface_extents.width) / self.window_logical_size.width as f64) as f32,
            (f64::from(self.surface_extents.height) / self.window_logical_size.height as f64)
                as f32,
        );

        canvas.reset_matrix();
        canvas.scale(scale);
    }

    /// Maps the given visible range to the render surface. For example, if you want a coordinate
    /// system where (0, 0) is the center of the screen, the X bounds are (-640, 640) and Y bounds
    /// are (-360, 360) you can specify that here.
    ///
    /// The scale_to_fit parameter will choose how to handle an inconsistent aspect ratio between
    /// visible_range and the surface. Common choices would be `skia_safe::matrix::ScaleToFit::Fill`
    /// to allow stretching or `skia_safe::matrix::ScaleToFit::Center` to scale such that the full
    /// visible_range is included (even if it means there is extra showing)
    ///
    /// Skia assumes that left is less than right and that top is less than bottom. If you provide
    /// a visible range that violates this, this function will apply a scaling factor to try to
    /// provide intuitive behavior. However, this can have side effects like upside-down text.
    ///
    /// See https://skia.org/user/api/SkMatrix_Reference#SkMatrix_setRectToRect
    /// See https://skia.org/user/api/SkMatrix_Reference#SkMatrix_ScaleToFit
    pub fn use_visible_range(
        &self,
        canvas: &mut skia_safe::Canvas,
        mut visible_range: skia_safe::Rect,
        scale_to_fit: skia_safe::matrix::ScaleToFit,
    ) -> Result<(), ()> {
        let x_scale = if visible_range.left <= visible_range.right {
            1.0
        } else {
            visible_range.left *= -1.0;
            visible_range.right *= -1.0;
            -1.0
        };

        let y_scale = if visible_range.top <= visible_range.bottom {
            1.0
        } else {
            visible_range.top *= -1.0;
            visible_range.bottom *= -1.0;
            -1.0
        };

        let dst = skia_safe::Rect {
            left: 0.0,
            top: 0.0,
            right: self.surface_extents.width as f32,
            bottom: self.surface_extents.height as f32,
        };

        let m = skia_safe::Matrix::from_rect_to_rect(visible_range, dst, scale_to_fit);
        match m {
            Some(m) => {
                canvas.set_matrix(&m);
                canvas.scale((x_scale, y_scale));
                Ok(())
            }
            None => Err(()),
        }
    }

    /// Given a center position and half-extents for X, calculate an appropriate Y half-extents that
    /// is consistent with the aspect ratio.
    pub fn use_fixed_width(
        &self,
        canvas: &mut skia_safe::Canvas,
        center: skia_safe::Point,
        x_half_extents: f32,
    ) -> Result<(), ()> {
        let left = center.x - x_half_extents;
        let right = center.x + x_half_extents;
        let y_half_extents = x_half_extents as f32
            / (self.surface_extents.width as f32 / self.surface_extents.height as f32);
        let top = center.y - y_half_extents;
        let bottom = center.y + y_half_extents;

        let rect = skia_safe::Rect {
            left,
            top,
            right,
            bottom,
        };

        self.use_visible_range(canvas, rect, skia_safe::matrix::ScaleToFit::Fill)
    }
}
