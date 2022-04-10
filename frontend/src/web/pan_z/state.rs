use crate::web::pan_z;
use crate::{AffineTransformMatrix, ViewBox, ViewPortPos, ViewPortRect};
use approx::abs_diff_eq;
use std::fmt::Display;
use wasm_bindgen::JsCast;
use web_sys::SvgElement;

pub trait PanZoomState<A: Clone + pan_z::PositionedExtent + 'static>: Display {
    /// Returns the local coordinates of `points`. These coordinates are in the
    /// internal coordinate system. They may only be compared to another result
    /// of this function, and only for equality, as distances in the other
    /// coordinate system may be different from the Html View Port coordinate
    /// system.
    fn as_local_coordinates(&self, element: A, point: ViewPortPos) -> (f64, f64);
    /// Returns width and height in an internal coordinate system. They may only
    /// be compared to other results of this function, as distances in the other
    /// coordinate system may be different from the Html View Port
    /// coordinate system.
    fn unscaled_dimensions(&self, element: A) -> (f64, f64);

    fn scale(&self) -> f64;
    fn top_left(&self, element: A) -> ViewPortPos;
    fn bounding_rect(&self, element: A) -> ViewPortRect;

    /// Changes the scale to `new_scale`, translating such, that `fix_point`
    /// remains at the same position.
    fn set_scale(&mut self, element: A, fix_point: ViewPortPos, new_scale: f64);
}

impl PanZoomState<SvgElement> for ViewBox {
    fn as_local_coordinates(&self, element: SvgElement, point: ViewPortPos) -> (f64, f64) {
        todo!()
    }

    fn unscaled_dimensions(&self, _element: SvgElement) -> (f64, f64) {
        let scale = self.scale();
        (self.width() * scale, self.height() * scale)
    }

    fn scale(&self) -> f64 {
        self.scale()
    }

    fn top_left(&self, element: SvgElement) -> ViewPortPos {
        let svg_to_view_port_transformation =
            AffineTransformMatrix::from(&element.dyn_into().unwrap());
        let svg_top_left = self.content_box().top_left();
        let view_port_top_left =
            ViewPortPos::from_svg_coords(svg_top_left, svg_to_view_port_transformation);
        info!(
            "top left of view box is svg {}, view port {}",
            svg_top_left, view_port_top_left
        );
        view_port_top_left
    }

    /// must have preserveAspectRatio "xMidYMid meet"
    fn bounding_rect(&self, element: SvgElement) -> ViewPortRect {
        let svg_to_view_port_transformation =
            AffineTransformMatrix::from(&element.dyn_into().unwrap());

        let svg_content_top_left = self.content_box().top_left();
        let view_port_content_top_left =
            ViewPortPos::from_svg_coords(svg_content_top_left, svg_to_view_port_transformation);
        info!(
            "top left of view box is svg {}, view port {}",
            svg_content_top_left, view_port_content_top_left
        );

        let content_bottom_right = ViewPortPos::from_svg_coords(
            self.content_box().bottom_right(),
            svg_to_view_port_transformation,
        );
        let view_box_view_port = ViewPortRect::new(
            view_port_content_top_left,
            content_bottom_right.x() - view_port_content_top_left.x(),
            content_bottom_right.y() - view_port_content_top_left.y(),
        );

        let view_box = self.view_box();
        debug_assert!(
            abs_diff_eq!(
                view_box.aspect_radio(),
                view_box_view_port.aspect_ratio(),
                epsilon = 1e-12
            ),
            "Aspect ratio of view box in view port coordinate system {} \
            does not match that in SVG coordinate system {}",
            view_box_view_port.aspect_ratio(),
            view_box.aspect_radio()
        );
        view_box_view_port
    }

    fn set_scale(&mut self, element: SvgElement, fix_point: ViewPortPos, new_scale: f64) {
        let old_scale: f64 = self.scale();
        warn!(
            "Changing scale from {} to {} with fix point {}",
            old_scale, new_scale, fix_point
        );
        let svg_to_view_port_transformation =
            AffineTransformMatrix::from(&element.dyn_into().unwrap());
        let view_port_to_svg_transformation =
            svg_to_view_port_transformation.try_inverse().unwrap();
        let fix_point_svg = fix_point.to_svg_coords(view_port_to_svg_transformation);
        let old_scale_top_left_svg = self.top_left();
        let old_scale_fixpoint_offset_from_top_left = fix_point_svg - old_scale_top_left_svg;
        let new_scale_fixpoint_offset_from_new_scale_top_left =
            old_scale_fixpoint_offset_from_top_left * (old_scale / new_scale);
        let new_scale_top_left_svg =
            fix_point_svg - new_scale_fixpoint_offset_from_new_scale_top_left;

        self.set_top_left(new_scale_top_left_svg);
        self.set_scale(new_scale);
        debug_assert_eq!(self.top_left(), new_scale_top_left_svg);
    }
}

pub mod view_state {
    use approx::relative_eq;
    use std::fmt::{Display, Formatter};

    use num_traits::Zero;
    use zoon::Mutable;
    use zoon::*;

    use crate::web::pan_z::screen_geom;
    use crate::web::pan_z::state::PanZoomState;
    use crate::{ScreenVec, ViewPortPos, ViewPortRect};

    #[derive(Debug, Copy, Clone)]
    pub struct ViewState {
        top_left: ViewPortPos,
        scale: f64,
    }

    impl<A: Clone + screen_geom::PositionedExtent + 'static> PanZoomState<A> for ViewState {
        fn as_local_coordinates(&self, element: A, point: ViewPortPos) -> (f64, f64) {
            todo!()
        }
        fn unscaled_dimensions(&self, element: A) -> (f64, f64) {
            let scaled_width = element.bounding_rect().width();
            let scaled_height = element.bounding_rect().height();
            (scaled_width / self.scale, scaled_height / self.scale)
        }

        fn scale(&self) -> f64 {
            self.scale
        }

        fn top_left(&self, _element: A) -> ViewPortPos {
            self.top_left
        }

        fn bounding_rect(&self, element: A) -> ViewPortRect {
            element.bounding_rect()
        }

        fn set_scale(&mut self, element: A, fix_point: ViewPortPos, new_scale: f64) {
            let old_scale: f64 = <Self as PanZoomState<A>>::scale(self);
            // distance from top left of zoom element in view port units
            let fixpoint_offset_from_top_left: ScreenVec =
                self.bounding_rect(element.clone()).offset(fix_point);
            warn!(
                "Computing translation for scale change from {} to {}, \n\
                with top-left relative fix point {}",
                old_scale, new_scale, fixpoint_offset_from_top_left
            );

            let scale_ratio = new_scale / old_scale;
            info!(
                "scale ratio {} = new scale {} / old scale {}",
                scale_ratio, new_scale, old_scale
            );

            let fix_point_stabilizing_translation = if relative_eq!(
                new_scale,
                old_scale,
                epsilon = f32::EPSILON as f64,
                max_relative = f32::EPSILON as f64
            ) {
                ScreenVec::zero()
            } else {
                let fix_point_stabilizing_translation =
                    fixpoint_offset_from_top_left * (1.0 - scale_ratio);
                info!(
                    "pointer_focus_stable_scaled_top_left: {} \
                    = fixpoint_offset_from_top_left {} * (1.0 - actual_scale_change {})",
                    fix_point_stabilizing_translation, fixpoint_offset_from_top_left, scale_ratio
                );
                fix_point_stabilizing_translation
            };
            let new_top_left = self.top_left(element) + fix_point_stabilizing_translation;

            self.scale = new_scale;
            self.top_left = new_top_left;
        }
    }

    impl Display for ViewState {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "ViewState{{top left: {}, scale: {}}}",
                self.top_left, self.scale
            )
        }
    }

    impl ViewState {
        fn new() -> Self {
            Self {
                top_left: Default::default(),
                scale: 1.0,
            }
        }
    }

    #[static_ref]
    pub fn view_state() -> &'static Mutable<ViewState> {
        Mutable::new(ViewState::new())
    }
}
