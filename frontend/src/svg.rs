use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Sub};

use approx::abs_diff_eq;
use num_traits::Zero;

pub trait ToSvgString {
    fn to_svg_string(&self) -> String;
}

/// A point in the SVG coordinate system
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SvgPoint {
    pub x: f64,
    pub y: f64,
}

impl Display for SvgPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SvgPoint ({}, {})", self.x, self.y)
    }
}
impl SvgPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Add<SvgVec> for SvgPoint {
    type Output = SvgPoint;

    fn add(self, rhs: SvgVec) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for SvgPoint {
    type Output = SvgVec;

    fn sub(self, rhs: Self) -> Self::Output {
        SvgVec::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Sub<SvgVec> for SvgPoint {
    type Output = SvgPoint;

    fn sub(self, rhs: SvgVec) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for SvgPoint {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

/// An element of the vector space of the SVG coordinate system
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SvgVec {
    x: f64,
    y: f64,
}

impl Add for SvgVec {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for SvgVec {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for SvgVec {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f64> for SvgVec {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Zero for SvgVec {
    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }
}
impl SvgVec {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

// unlike the mathematical `BoundingRect`, SvgRect's top<bottom
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SvgRect {
    pub top_left: SvgPoint,
    pub dimensions: SvgVec,
}

impl Display for SvgRect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SvgRect x:{{{}..{}}} ⨯ y:{{{}..{}}}",
            self.left(),
            self.right(),
            self.top(),
            self.bottom()
        )
    }
}

impl SvgRect {
    pub fn new(top_left: SvgPoint, dimensions: SvgVec) -> Self {
        debug_assert!(dimensions.x >= 0.0);
        debug_assert!(dimensions.y >= 0.0);
        Self {
            top_left,
            dimensions,
        }
    }
    pub fn top(&self) -> f64 {
        self.top_left.y
    }
    pub fn bottom(&self) -> f64 {
        self.top() + self.height()
    }
    pub fn left(&self) -> f64 {
        self.top_left.x
    }
    pub fn right(&self) -> f64 {
        self.left() + self.width()
    }
    pub fn width(&self) -> f64 {
        self.dimensions.x
    }
    pub fn height(&self) -> f64 {
        self.dimensions.y
    }
    pub fn top_left(&self) -> SvgPoint {
        self.top_left
    }
    pub fn bottom_right(&self) -> SvgPoint {
        SvgPoint::new(self.right(), self.bottom())
    }

    pub fn dimensions(&self) -> SvgVec {
        self.dimensions
    }

    /// width / height
    pub fn aspect_radio(&self) -> f64 {
        self.width() / self.height()
    }
}

/// The visible part of the infinite SVG canvas.
///
/// In a larger view port -- the part of the screen, that displays the `ViewBox`
/// -- the content of same `ViewBox` will appear larger than in a smaller view
/// port.
///
/// `ViewBox` is not aware of padding. To create padding just create a larger
/// view box.
#[derive(Debug, Clone, Copy)]
pub struct ViewBox {
    /// The visible part of the SVG. May contain parts or all of the content or
    /// a much wider area than the content.
    ///
    /// Its size and position are controlled by the user; its shape (i.e. aspect
    /// ratio) is controlled by by changes to the view-port.
    // FIXME: Its aspect ratio must must match the view-port's (not the content's ) to avoid empty
    // borders, when zooming in -> There must be a setter for the aspect ratio
    view_box: SvgRect,
    /// The bounding box of content in the SVG canvas. Changes when the content
    /// changes.
    // Used as a measure for determining the scale of the ViewBox.
    content_box: SvgRect,
}

impl Default for ViewBox {
    fn default() -> Self {
        let rect = SvgRect {
            top_left: SvgPoint::new(-100.0, -100.0),
            dimensions: SvgVec::new(200.0, 200.0),
        };
        Self {
            view_box: rect,

            content_box: rect,
        }
    }
}

impl ViewBox {
    pub fn new(view_box: SvgRect, content_box: SvgRect) -> Self {
        Self {
            view_box,
            content_box,
        }
    }

    pub fn min_x(&self) -> f64 {
        self.view_box.left()
    }
    pub fn min_y(&self) -> f64 {
        self.view_box.top()
    }

    pub fn width(&self) -> f64 {
        self.view_box.width()
    }
    pub fn height(&self) -> f64 {
        self.view_box.height()
    }
    pub fn scale(&self) -> f64 {
        if self.content_box.dimensions().is_zero() {
            1.0
        } else if self.content_box.width() == 0.0 {
            self.content_box.height() / self.height()
        } else if self.content_box.height() == 0.0 {
            self.content_box.height() / self.width()
        } else {
            f64::max(
                self.content_box.width() / self.width(),
                self.content_box.height() / self.height(),
            )
        }
    }
    pub fn set_scale(&mut self, new_scale: f64) {
        info!(
            "Changing scale of view box from {} to {}",
            self.scale(),
            new_scale
        );
        debug_assert!(new_scale >= 0.0);
        self.view_box.dimensions = self.content_box.dimensions / new_scale;

        debug_assert!(
            abs_diff_eq!(self.scale(), new_scale, epsilon = 1e-12),
            "Computed scale {} does not match set scale {}",
            self.scale(),
            new_scale
        );
    }
    pub fn top_left(&self) -> SvgPoint {
        self.view_box.top_left
    }

    pub fn set_top_left(&mut self, pos: SvgPoint) {
        info!(
            "Changing top left of view box from {} to {}",
            self.view_box.top_left, pos
        );
        self.view_box.top_left = pos;
    }
    pub fn content_box(&self) -> SvgRect {
        self.content_box
    }
    pub fn set_content_box(&mut self, rect: SvgRect) {
        self.content_box = rect
    }
    pub fn view_box(&self) -> SvgRect {
        self.view_box
    }
}
impl Display for ViewBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ViewBox {{{}}}", self.to_svg_string())
    }
}
impl ToSvgString for ViewBox {
    fn to_svg_string(&self) -> String {
        format!(
            "{} {} {} {}",
            self.min_x(),
            self.min_y(),
            self.width(),
            self.height()
        )
    }
}
