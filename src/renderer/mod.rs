use ash::vk;

mod alignment;
use alignment::Align;

pub mod util;

mod window_support;

mod instance;
pub use instance::VkInstance;
pub use instance::CreateInstanceError; // TODO: Should this be re-exported like this? Name is very general.

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

// Values here match VkPresentModeKHR, FIFO is the recommended default
#[derive(Copy, Clone, Debug)]
pub enum PresentMode {
    Immediate = 0,
    Mailbox = 1,
    Fifo = 2,
    FifoRelaxed = 3,
}

impl PresentMode {
    pub fn to_vk(self) -> vk::PresentModeKHR {
        match self {
            PresentMode::Immediate => vk::PresentModeKHR::IMMEDIATE,
            PresentMode::Mailbox => vk::PresentModeKHR::MAILBOX,
            PresentMode::Fifo => vk::PresentModeKHR::FIFO,
            PresentMode::FifoRelaxed => vk::PresentModeKHR::FIFO_RELAXED,
        }
    }
}

// Values here match VkPhysicalDeviceType, DISCRETE_GPU is the recommended default
#[derive(Copy, Clone, Debug)]
pub enum PhysicalDeviceType {
    Other = 0,
    IntegratedGpu = 1,
    DiscreteGpu = 2,
    VirtualGpu = 3,
    Cpu = 4,
}

impl PhysicalDeviceType {
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
