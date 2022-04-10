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

fn root() -> RawHtmlEl {
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
    let root_element = root();
    //root_element.after_insert(||);
    start_app("main", || root_element);
}
