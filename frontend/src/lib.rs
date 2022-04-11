#![feature(associated_type_bounds)]

#[macro_use]
extern crate approx;
#[macro_use]
extern crate log;
#[macro_use]
extern crate zoon;
use svg::ViewBox;
use web::pan_z::*;
use web::IntoElementWithAttributeSignal;
use zoon::*;
mod svg;
mod web;
// ------ ------
//    States
// ------ ------

#[static_ref]
pub fn view_box() -> &'static Mutable<ViewBox> {
    Mutable::new(ViewBox::default())
}

// ------ ------
//   Commands
// ------ ------

// ------ ------
//     View
// ------ ------

fn root() -> impl Element {
    El::new()
        .s(Borders::all(Border::new()))
        .s(Width::new(320))
        .s(Height::new(320))
        .s(Align::center())
        .child(artboard())
}

fn artboard() -> impl Element {
    #[derive(Clone, Copy)]
    struct ViewBox {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }

    let (view_box, view_box_signal) = Mutable::new_and_signal(ViewBox {
        x: -100.,
        y: -100.,
        width: 200.,
        height: 200.,
    });

    let svg = RawSvgEl::new("svg");
    let svg_dom_element = svg.dom_element().unchecked_into::<web_sys::SvgsvgElement>();

    svg
        .attr("width", "100%")
        .attr("height", "100%")
        .attr_signal("viewBox", view_box_signal.map(|view_box| {
            let ViewBox { x, y, width, height } = view_box;
            format!("{x} {y} {width} {height}")
        }))
        .event_handler_with_options(EventOptions::new().preventable(), move |event: events_extra::WheelEvent| {
            event.prevent_default();
            let current_view_box = view_box.get();

            let (width, height) = { 
                // Note: It could be replaced with `.on_resize` + `Rc<Cell<width, height>>` 
                // once ResizeObserver can reliable observer SVG elements (is there a workaround?)
                let dom_rect = svg_dom_element.get_bounding_client_rect();
                (dom_rect.width(), dom_rect.height())
            };
            
            let origin_x = f64::from(event.offset_x());
            let origin_y = f64::from(event.offset_y());
            let zoom = event.delta_y().signum() * 0.2;
            let delta_view_box_width = current_view_box.width * zoom;
            let delta_view_box_height = current_view_box.height * zoom;

            view_box.set(ViewBox {
                x: current_view_box.x - (delta_view_box_width / width * origin_x),
                y: current_view_box.y - (delta_view_box_height / height * origin_y),
                width: current_view_box.width + delta_view_box_width,
                height: current_view_box.height + delta_view_box_height,
            });
        })
        .children(circles())
}

fn circles() -> impl IntoIterator<Item = impl Element> {
    [
        RawSvgEl::new("circle")
            .attr("cx", "-30")
            .attr("cy", "-30")
            .attr("r", "10")
            .attr("fill", "cadetblue"),
        RawSvgEl::new("circle")
            .attr("cx", "30")
            .attr("cy", "30")
            .attr("r", "10")
            .attr("fill", "steelblue"),
        RawSvgEl::new("circle")
            .attr("cx", "30")
            .attr("cy", "-30")
            .attr("r", "10")
            .attr("fill", "lightblue"),
        RawSvgEl::new("circle")
            .attr("cx", "-30")
            .attr("cy", "30")
            .attr("r", "10")
            .attr("fill", "cornflowerblue"),
    ]
}





fn _root() -> RawHtmlEl {
    RawHtmlEl::new("div").children([
        //enable_zooming_html_element(RawHtmlEl::new("article").child(Text::new("bla bla bla")),view_state())
        unsafe { enable_zooming_svg_view_box(four_circles(), view_box()) },
    ])
}

fn four_circles() -> RawSvgEl {
    view_box()
        .signal()
        .into_element_with_attribute_signal((), "my_svg_element", None)
        .children([
            RawSvgEl::new("circle")
                .attr("cx", "-30")
                .attr("cy", "-30")
                .attr("r", "10")
                .attr("fill", "cadetblue"),
            RawSvgEl::new("circle")
                .attr("cx", "30")
                .attr("cy", "30")
                .attr("r", "10")
                .attr("fill", "steelblue"),
            RawSvgEl::new("circle")
                .attr("cx", "30")
                .attr("cy", "-30")
                .attr("r", "10")
                .attr("fill", "lightblue"),
            RawSvgEl::new("circle")
                .attr("cx", "-30")
                .attr("cy", "30")
                .attr("r", "10")
                .attr("fill", "cornflowerblue"),
        ])
}

pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(fern::Output::call(console_log::log))
        .apply()?;
    //  log_panics::init();
    Ok(())
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    setup_logger().unwrap();
    start_app("app", root);
}
