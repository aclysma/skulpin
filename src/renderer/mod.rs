mod alignment;
use alignment::Align;

pub mod util;

mod window_support;

mod instance;
pub use instance::VkInstance;

mod device;
pub use device::VkDevice;
pub use device::QueueFamilyIndices; // TODO: Should this be re-exported like this? Name is very general.
pub use device::Queues; // TODO: Should this be re-exported like this? Name is very general.

mod swapchain;
pub use swapchain::VkSwapchain;
pub use swapchain::SwapchainInfo;
pub use swapchain::MAX_FRAMES_IN_FLIGHT;

mod skia_pipeline;
pub use skia_pipeline::VkPipeline;

mod skia_support;
pub use skia_support::VkSkiaContext;

mod buffer;
pub use buffer::VkBuffer;

mod debug_reporter;
pub use debug_reporter::VkDebugReporter;

mod renderer;
pub use renderer::RendererBuilder;
pub use renderer::Renderer;