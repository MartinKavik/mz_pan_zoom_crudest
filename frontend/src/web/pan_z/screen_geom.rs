use crate::svg::SvgPoint;
use num_traits::Zero;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Mul, Sub};
use wasm_bindgen::JsCast;
use web_sys::{Document, EventTarget, PointerEvent, SvgsvgElement, Window};
use zoon::events_extra::{
    PointerCancel, PointerDown, PointerLeave, PointerMove, PointerUp, WheelEvent,
};

/// A Matrix
///   
///    [a c e]
///    [b d f]
///    [0 0 1]
///
/// # See
/// * https://developer.mozilla.org/en-US/docs/Web/API/SVGMatrix
/// * https://en.wikipedia.org/wiki/Transformation_matrix#Affine_transformations
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AffineTransformMatrix {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl From<&SvgsvgElement> for AffineTransformMatrix {
    fn from(element: &SvgsvgElement) -> Self {
        let svg_matrix = element.get_screen_ctm().unwrap();
        Self {
            a: svg_matrix.a() as f64,
            b: svg_matrix.b() as f64,
            c: svg_matrix.c() as f64,
            d: svg_matrix.d() as f64,
            e: svg_matrix.e() as f64,
            f: svg_matrix.f() as f64,
        }
    }
}

impl AffineTransformMatrix {
    ///
    /// # See
    /// https://www.wolframalpha.com/input?i=inverse+%7B%7Ba%2C+c%2C+e%7D%2C+%7Bb%2C+d%2C+f%7D%2C+%7B0%2C0%2C1%7D%7D
    pub fn try_inverse(&self) -> Option<Self> {
        let det2 = self.a * self.d - self.c * self.b;
        if det2.abs() < f64::EPSILON {
            return None;
        }
        let i = Self {
            a: self.d / det2,
            b: -self.b / det2,
            c: -self.c / det2,
            d: self.a / det2,
            e: -(self.d * self.e - self.c * self.f) / det2,
            f: (self.b * self.e - self.a * self.f) / det2,
        };
        debug_assert!((self.a * i.a + self.c * i.b - 1.0).abs() < f64::EPSILON);
        debug_assert!((self.a * i.c + self.c * i.d).abs() < f64::EPSILON);
        debug_assert!((self.b * i.a + self.d * i.b).abs() < f64::EPSILON);
        debug_assert!((self.b * i.c + self.d * i.d - 1.0).abs() < f64::EPSILON);
        debug_assert!((self.a * i.e + self.c * i.f + self.e).abs() < 10000.0 * f64::EPSILON);
        debug_assert!(
            (self.b * i.e + self.d * i.f + self.f).abs() < 10000.0 * f64::EPSILON,
            "{} != 0",
            self.b * i.e + self.d * i.f + self.f
        );
        Some(i)
    }
}

/// A 2-dimensional position relative to the view port in screen coordinates --
/// i.e. the y-axis points downwards.
///
/// ViewPortPos are assumed to always be relative to the top-left corner of the
/// view port, i.e. the `ViewPortPos::origin` always refers to the top-left
/// corner of the view port.
///
/// ViewPortPos can be off screen and outside the view port, i.e. their
/// positions can be negative or beyond the displayable area.
///
/// # See
/// https://developer.mozilla.org/en-US/docs/Web/CSS/Viewport_concepts
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct ViewPortPos {
    x: f64,
    y: f64,
}

impl Display for ViewPortPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add<ScreenVec> for ViewPortPos {
    type Output = ViewPortPos;

    fn add(self, rhs: ScreenVec) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<ScreenVec> for ViewPortPos {
    fn add_assign(&mut self, rhs: ScreenVec) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for ViewPortPos {
    type Output = ScreenVec;

    fn sub(self, rhs: Self) -> Self::Output {
        ScreenVec {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ViewPortPos {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }
    pub fn as_vec(&self) -> ScreenVec {
        ScreenVec {
            x: self.x,
            y: self.y,
        }
    }
    pub fn to_svg_coords(
        &self,
        view_port_to_svg_transformation: AffineTransformMatrix,
    ) -> SvgPoint {
        let m = view_port_to_svg_transformation;
        SvgPoint::new(
            m.a * self.x + m.c * self.y + m.e,
            m.b * self.x + m.d * self.y + m.f,
        )
    }
    pub fn from_svg_coords(
        svg_point: SvgPoint,
        svg_to_view_port_transformation: AffineTransformMatrix,
    ) -> Self {
        let m = svg_to_view_port_transformation;
        Self::new(
            m.a * svg_point.x + m.c * svg_point.y + m.e,
            m.b * svg_point.x + m.d * svg_point.y + m.f,
        )
    }
}

/// A 2-dimensional vector in screen coordinates -- i.e. the y-axis points
/// downwards
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScreenVec {
    x: f64,
    y: f64,
}
impl Display for ScreenVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec({}, {})", self.x, self.y)
    }
}
impl Mul<f64> for ScreenVec {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Add for ScreenVec {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for ScreenVec {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Zero for ScreenVec {
    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

impl ScreenVec {
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

/// A rect relative to the view port in screen-coordinates -- i.e. the y-axis
/// points downwards.
///
/// # See
/// https://developer.mozilla.org/en-US/docs/Web/CSS/Viewport_concepts
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ViewPortRect {
    top_left: ViewPortPos,
    width: f64,
    height: f64,
}

impl Display for ViewPortRect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rect x:{{{}..{}}} ⨯ y:{{{}..{}}}",
            self.left(),
            self.right(),
            self.top(),
            self.bottom()
        )
    }
}

impl ViewPortRect {
    pub fn new(top_left: ViewPortPos, width: f64, height: f64) -> Self {
        debug_assert!(width >= 0.0);
        debug_assert!(height >= 0.0);
        Self {
            top_left,
            width,
            height,
        }
    }

    pub fn top(&self) -> f64 {
        self.top_left.y
    }
    pub fn left(&self) -> f64 {
        self.top_left.x
    }
    pub fn bottom(&self) -> f64 {
        self.top_left.y + self.height
    }
    pub fn right(&self) -> f64 {
        self.top_left.x + self.width
    }
    pub fn width(&self) -> f64 {
        self.width
    }
    pub fn height(&self) -> f64 {
        self.height
    }
    pub fn top_left(&self) -> ViewPortPos {
        self.top_left
    }
    pub fn bottom_right(&self) -> ViewPortPos {
        ViewPortPos::new(self.right(), self.bottom())
    }

    /// width / height
    pub fn aspect_ratio(&self) -> f64 {
        self.width() / self.height()
    }

    /// Returns the vector from the top-left corner of this rect to `point`.
    pub fn offset(&self, point: ViewPortPos) -> ScreenVec {
        let offset = point - self.top_left;
        warn!("offset for {} to top left of {} is {}", point, self, offset);
        offset
    }

    /// Returns a vector with each offset dimension divided by the rect-size in
    /// the respective dimension.
    ///
    /// `point`s inside the rect thus have values in each dimensions between
    /// `0.0` and `1.0`.
    pub fn rect_size_relative_offset(&self, point: ViewPortPos) -> ScreenVec {
        let absolute_offset = self.offset(point);
        ScreenVec {
            x: absolute_offset.x / self.width,
            y: absolute_offset.y / self.height,
        }
    }
}

pub trait Positioned {
    fn pos(&self) -> ViewPortPos;
}

impl Positioned for PointerEvent {
    fn pos(&self) -> ViewPortPos {
        ViewPortPos {
            x: self.client_x() as f64,
            y: self.client_y() as f64,
        }
    }
}

impl Positioned for PointerDown {
    fn pos(&self) -> ViewPortPos {
        ViewPortPos {
            x: self.x() as f64,
            y: self.y() as f64,
        }
    }
}
impl Positioned for PointerUp {
    fn pos(&self) -> ViewPortPos {
        ViewPortPos {
            x: self.x() as f64,
            y: self.y() as f64,
        }
    }
}
impl Positioned for PointerCancel {
    fn pos(&self) -> ViewPortPos {
        ViewPortPos {
            x: self.x() as f64,
            y: self.y() as f64,
        }
    }
}
impl Positioned for PointerLeave {
    fn pos(&self) -> ViewPortPos {
        ViewPortPos {
            x: self.x() as f64,
            y: self.y() as f64,
        }
    }
}
impl Positioned for PointerMove {
    fn pos(&self) -> ViewPortPos {
        ViewPortPos {
            x: self.x() as f64,
            y: self.y() as f64,
        }
    }
}

impl Positioned for WheelEvent {
    fn pos(&self) -> ViewPortPos {
        ViewPortPos {
            x: self.x() as f64,
            y: self.y() as f64,
        }
    }
}

/// A geometrically positioned and sized object, with a (possibly zero) extent
pub trait PositionedExtent {
    fn top_left(&self) -> ViewPortPos;
    fn bounding_rect(&self) -> ViewPortRect;
}

//FIXME: code is untested
impl PositionedExtent for web_sys::Window {
    fn top_left(&self) -> ViewPortPos {
        ViewPortPos::origin()
    }

    fn bounding_rect(&self) -> ViewPortRect {
        let width_js_value = self.inner_width().unwrap();
        let width = width_js_value.as_f64().unwrap_or_else(|| {
            panic!(
                "Window::inner_width value `{:#?}` is not a number",
                width_js_value
            )
        });
        let height_js_value = self.inner_height().unwrap();
        let height = height_js_value.as_f64().unwrap_or_else(|| {
            panic!(
                "Window::inner_height value `{:#?}` is not a number",
                height_js_value
            )
        });
        ViewPortRect::new(ViewPortPos::origin(), width, height)
    }
}

//FIXME: code is untested
impl PositionedExtent for web_sys::Document {
    fn top_left(&self) -> ViewPortPos {
        let body = self.body().unwrap();
        let bc_rect = body.get_bounding_client_rect();
        ViewPortPos::new(bc_rect.x(), bc_rect.y())
    }

    fn bounding_rect(&self) -> ViewPortRect {
        let body = self.body().unwrap();
        let bc_rect = body.get_bounding_client_rect();
        ViewPortRect::new(
            ViewPortPos::new(bc_rect.x(), bc_rect.y()),
            bc_rect.width(),
            bc_rect.height(),
        )
    }
}

impl PositionedExtent for web_sys::Element {
    fn top_left(&self) -> ViewPortPos {
        let bc_rect = self.get_bounding_client_rect();
        ViewPortPos::new(bc_rect.x(), bc_rect.y())
    }

    fn bounding_rect(&self) -> ViewPortRect {
        let bc_rect = self.get_bounding_client_rect();
        ViewPortRect::new(
            ViewPortPos::new(bc_rect.x(), bc_rect.y()),
            bc_rect.width(),
            bc_rect.height(),
        )
    }
}

// duplication for HTML and SVG, as Element is not a trait
impl PositionedExtent for web_sys::HtmlElement {
    fn top_left(&self) -> ViewPortPos {
        let bc_rect = self.get_bounding_client_rect();
        ViewPortPos::new(bc_rect.x(), bc_rect.y())
    }

    fn bounding_rect(&self) -> ViewPortRect {
        let bc_rect = self.get_bounding_client_rect();
        ViewPortRect::new(
            ViewPortPos::new(bc_rect.x(), bc_rect.y()),
            bc_rect.width(),
            bc_rect.height(),
        )
    }
}

impl PositionedExtent for web_sys::SvgElement {
    fn top_left(&self) -> ViewPortPos {
        let bc_rect = self.get_bounding_client_rect();
        ViewPortPos::new(bc_rect.x(), bc_rect.y())
    }

    fn bounding_rect(&self) -> ViewPortRect {
        let bc_rect = self.get_bounding_client_rect();
        ViewPortRect::new(
            ViewPortPos::new(bc_rect.x(), bc_rect.y()),
            bc_rect.width(),
            bc_rect.height(),
        )
    }
}

impl PositionedExtent for web_sys::SvgsvgElement {
    fn top_left(&self) -> ViewPortPos {
        let bc_rect = self.get_bounding_client_rect();
        ViewPortPos::new(bc_rect.x(), bc_rect.y())
    }

    fn bounding_rect(&self) -> ViewPortRect {
        let bc_rect = self.get_bounding_client_rect();
        ViewPortRect::new(
            ViewPortPos::new(bc_rect.x(), bc_rect.y()),
            bc_rect.width(),
            bc_rect.height(),
        )
    }
}

impl PositionedExtent for PositionedJsObject {
    fn top_left(&self) -> ViewPortPos {
        match self {
            PositionedJsObject::Window(window) => window.top_left(),
            PositionedJsObject::Document(document) => document.top_left(),
            PositionedJsObject::Element(element) => element.top_left(),
        }
    }

    fn bounding_rect(&self) -> ViewPortRect {
        match self {
            PositionedJsObject::Window(window) => {
                warn!("getting bounding rect of window");
                window.bounding_rect()
            }
            PositionedJsObject::Document(document) => {
                warn!("getting bounding rect of document");
                document.bounding_rect()
            }
            PositionedJsObject::Element(element) => {
                warn!("getting bounding rect of element");
                element.bounding_rect()
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum PositionedJsObject {
    Window(web_sys::Window),
    Document(web_sys::Document),
    Element(web_sys::Element),
}

impl Debug for PositionedJsObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionedJsObject::Window(_) => {
                write!(f, "Window")
            }
            PositionedJsObject::Document(_) => {
                write!(f, "Document")
            }
            PositionedJsObject::Element(element) => {
                let element: &web_sys::Element = element;
                write!(
                    f,
                    "{} id={}, class={}",
                    element.tag_name(),
                    element
                        .get_attribute("id")
                        .unwrap_or_else(|| "EMPTY".to_string()),
                    element
                        .get_attribute("class")
                        .unwrap_or_else(|| "EMPTY".to_string())
                )
            }
        }
    }
}

pub struct PositionedCastErr(EventTarget);

impl Display for PositionedCastErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unexpected event target type {}", self.0.to_string())
    }
}

impl Debug for PositionedCastErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unexpected event target type {}.\n\
         Not every event target type can be cast as Positioned.\n\
         Currently only Window, Document and Element are supported, \
         as the others do not seem to have a determinable geometric position.\n\
         The set of event target types seems to be extensible. \
         And there already are types beyond the mentioned:\n\
         All sub types of Node other than Element, like DocumentFragment, Attr and CharacterData\n\
         as well as non-Node types,like XMLHttpRequest, AudioNode, AudioContext, \
         MediaStream, Worker.",
            self.0.to_string()
        )
    }
}

impl Error for PositionedCastErr {}

impl TryFrom<EventTarget> for PositionedJsObject {
    type Error = PositionedCastErr;

    fn try_from(event_target: EventTarget) -> Result<Self, Self::Error> {
        if event_target.is_instance_of::<Window>() {
            //If you need to obtain the width of the window minus the scrollbar and
            // borders, use the root <html> element's clientWidth property instead.
            let window: Window = event_target.dyn_into().unwrap();
            Ok(Self::Window(window))
        } else if event_target.is_instance_of::<Document>() {
            let document: Document = event_target.dyn_into().unwrap();
            Ok(Self::Document(document))
        } else if event_target.is_instance_of::<web_sys::Element>() {
            let target_element = event_target.dyn_into::<web_sys::Element>().unwrap();
            Ok(Self::Element(target_element))
        } else {
            Err(PositionedCastErr(event_target))
        }
    }
}
