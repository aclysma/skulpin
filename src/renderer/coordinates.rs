
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LogicalSize {
    pub width: u32,
    pub height: u32
}

impl LogicalSize {
    pub fn new(width: u32, height: u32) -> Self {
        LogicalSize {
            width,
            height,
        }
    }

    pub fn to_physical(&self, scale_factor: f64) -> LogicalSize {
        LogicalSize {
            width: (self.width as f64 / scale_factor).round() as u32,
            height: (self.height as f64 / scale_factor).round() as u32,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32
}

impl PhysicalSize {
    pub fn new(width: u32, height: u32) -> Self {
        PhysicalSize {
            width,
            height,
        }
    }

    pub fn to_logical(&self, scale_factor: f64) -> LogicalSize {
        LogicalSize {
            width: (self.width as f64 * scale_factor).round() as u32,
            height: (self.height as f64 * scale_factor).round() as u32,
        }
    }
}