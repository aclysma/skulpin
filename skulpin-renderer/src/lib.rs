#[macro_use]
extern crate log;

pub use rafx;
pub use skia_safe;
pub use skia_bindings;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

// mod skia_renderpass;
// pub use skia_renderpass::VkSkiaRenderPass;
//
// mod skia_support;
// pub use skia_support::VkSkiaContext;

#[allow(clippy::module_inception)]
mod renderer;
pub use renderer::RendererBuilder;
pub use renderer::Renderer;
//pub use renderer::CreateRendererError;
pub use renderer::RendererPlugin;

mod coordinates;
pub use coordinates::Size;
pub use coordinates::LogicalSize;
pub use coordinates::PhysicalSize;
use rafx::api::RafxExtents2D;

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
#[derive(Clone)]
pub struct CoordinateSystemHelper {
    surface_extents: RafxExtents2D,
    window_logical_size: LogicalSize,
    window_physical_size: PhysicalSize,
    scale_factor: f64,
}

impl CoordinateSystemHelper {
    /// Create a CoordinateSystemHelper for a window of the given parameters
    pub fn new(
        surface_extents: RafxExtents2D,
        window_logical_size: LogicalSize,
        window_physical_size: PhysicalSize,
        scale_factor: f64,
    ) -> Self {
        CoordinateSystemHelper {
            surface_extents,
            window_logical_size,
            window_physical_size,
            scale_factor,
        }
    }

    /// Get the raw pixel size of the surface to which we are drawing
    pub fn surface_extents(&self) -> RafxExtents2D {
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

    /// Get the multiplier used for high-dpi displays. For example, a 4K display simulating a 1080p
    /// display will use a factor of 2.0
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
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
