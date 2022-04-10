use approx::relative_eq;

use wasm_bindgen::JsCast;
use web_sys::SvgElement;
use zoon::dominator::{with_node, EventOptions};
use zoon::events_extra::WheelEvent;
use zoon::RawEl;
use zoon::*;

use crate::ViewBox;
pub use screen_geom::*;
pub use state::view_state::view_state;
use state::PanZoomState;

mod screen_geom;
mod state;
const ZOOM_SPEED_FACTOR: f64 = 0.05;

/// Safety: `el` must be created from a signal of `view_box` and have the
/// default value for the `preserveAspectRatio` attribute, which is `"xMidYMid
/// meet"`
pub unsafe fn enable_zooming_svg_view_box(
    el: RawSvgEl,
    view_box: &'static Mutable<ViewBox>,
) -> RawSvgEl {
    el.update_dom_builder(|builder| {
        let builder: DomBuilder<SvgElement> = builder;
        set_zoom_event_listener(builder, view_box)
    })
}

pub fn enable_zooming_svg_element<PZ: PanZoomState<SvgElement>>(
    _el: RawSvgEl,
    _state: &Mutable<SvgElement>,
) -> RawSvgEl {
    todo!("implement either like the html version or the view box version")
}

pub fn enable_zooming_html_element<PZ: PanZoomState<web_sys::HtmlElement>>(
    el: RawHtmlEl,
    state: &'static Mutable<PZ>,
) -> RawHtmlEl {
    el.update_dom_builder(|builder| {
        let element = builder.__internal_element();
        let builder = builder.style("transform-origin", "0 0").style_signal(
            "transform",
            state.signal_ref(move |view_state| {
                warn!("view state: {}", view_state);
                let top_left_pos = view_state.top_left(element.clone());
                format!(
                    "translate({}px, {}px) scale({})",
                    top_left_pos.x(),
                    top_left_pos.y(),
                    view_state.scale()
                )
            }),
        );
        set_zoom_event_listener(builder, state)
    })
}

fn set_zoom_event_listener<
    A: Clone + screen_geom::PositionedExtent + 'static,
    PZ: PanZoomState<A>,
>(
    builder: DomBuilder<A>,
    // state must have a 'static lifetime, as the event listener might live for the rest of
    // eternity
    state: &'static Mutable<PZ>,
) -> DomBuilder<A> {
    with_node!(builder,zoom_element => {
    .global_event_with_options(&EventOptions::preventable(),move |e: WheelEvent| {
            e.prevent_default();
            if e.ctrl_key(){
            // zooming by delta_y of mouse wheel
                  let zoom_element_bounds = state.lock_ref().bounding_rect(zoom_element.clone());

                let fix_point = e.pos();
                let zoom_amount = -e.delta_y() * ZOOM_SPEED_FACTOR;
    warn!("Zooming by {}% with fixpoint {}", zoom_amount, fix_point);
    let rect_relative_fix_point_offset: ScreenVec =
        zoom_element_bounds.rect_size_relative_offset(fix_point);

    let unscaled_dimensions: (f64, f64) =
        state.lock_ref().unscaled_dimensions(zoom_element.clone());
    if zoom_amount != 0.0 {
        //TODO: use the mouse wheel to determine a force of scroll (instead of
        // amount). Instead a velocity tracker to control the mass of the "object" and
        // the friction of the movement.
        let new_scale = f64::max(state.lock_ref().scale() * (1.0 + zoom_amount / 100.0), 0.0);
        state
            .lock_mut()
            .set_scale(zoom_element.clone(), fix_point, new_scale)
    }

    if cfg!(debug_assertions) {
        let zoom_element = zoom_element.clone();


        let func = move || {
            warn!("Locking zoom state in animation frame");
            let this = state.lock_ref();
            let new_zoon_element_bounds = this.bounding_rect(zoom_element.clone());
            let new_unscaled_dimensions: (f64, f64) =
                this.unscaled_dimensions(zoom_element.clone());
            info!("Lock freed");
            assert_ne!(new_zoon_element_bounds, zoom_element_bounds);

            assert!(is_pair_approx_eq_f32_epsilon(new_unscaled_dimensions, unscaled_dimensions),
                "Unscaled dimensions new {:?} != old {:?}",
                new_unscaled_dimensions,
                unscaled_dimensions
            );
            let new_rect_relative_fix_point_offset =
                new_zoon_element_bounds.rect_size_relative_offset(fix_point);
            let relative_pointer_offset_diff =
                new_rect_relative_fix_point_offset - rect_relative_fix_point_offset;

            assert!(is_screen_vec_approx_eq_f32_epsilon(new_rect_relative_fix_point_offset,rect_relative_fix_point_offset),
                "\nrelative pointer offset must be a fix point, i.e new == old.\n\
                order of magnitude: ({}, {})\n\
                absolute unzoomed diff: ({}, {})",
                f64::max(new_rect_relative_fix_point_offset.x(), rect_relative_fix_point_offset.x()),
                f64::max(new_rect_relative_fix_point_offset.y(), rect_relative_fix_point_offset.y()),
                relative_pointer_offset_diff.x() * new_unscaled_dimensions.0,
                relative_pointer_offset_diff.y() * new_unscaled_dimensions.1
            );
        };
        after_redraw(func)
    }
            } else {
            // panning by delta_x or delta_y of mouse wheel
                todo!()
            }
        })
    })
}

fn is_pair_approx_eq_f32_epsilon(a: (f64, f64), b: (f64, f64)) -> bool {
    relative_eq!(
        a.0 as f32,
        b.0 as f32,
        epsilon = f32::EPSILON,
        max_relative = f32::EPSILON
    ) && relative_eq!(
        a.1 as f32,
        b.1 as f32,
        epsilon = f32::EPSILON,
        max_relative = f32::EPSILON
    )
}
fn is_screen_vec_approx_eq_f32_epsilon(a: ScreenVec, b: ScreenVec) -> bool {
    relative_eq!(
        a.x() as f32,
        b.x() as f32,
        epsilon = f32::EPSILON,
        max_relative = f32::EPSILON
    ) && relative_eq!(
        a.y() as f32,
        b.y() as f32,
        epsilon = f32::EPSILON,
        max_relative = f32::EPSILON
    )
}

fn after_redraw<F: FnMut() + 'static>(mut func: F) {
    let before_next_redraw: Closure<dyn FnMut(f64)> = Closure::once(move |time| {
        warn!("Before redraw, {}s after document creation", time / 1000.0);
        let after_next_redraw: Closure<dyn FnMut(f64)> = Closure::once(move |time| {
            warn!("After redraw, {}s after document creation", time / 1000.0);
            (func)()
        });
        let after_next_redraw = Box::leak(Box::new(after_next_redraw));
        let _second_request_id: i32 = window()
            .request_animation_frame(after_next_redraw.as_ref().unchecked_ref())
            .unwrap();
    });
    let before_next_redraw = Box::leak(Box::new(before_next_redraw));
    let _request_id: i32 = window()
        .request_animation_frame(before_next_redraw.as_ref().unchecked_ref())
        .unwrap();
}
