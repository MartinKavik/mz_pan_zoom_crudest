pub mod pan_z;
use crate::svg::ToSvgString;
use crate::ViewBox;
use zoon::{Broadcaster, RawEl, RawSvgEl, Signal};

pub trait AsElement {
    type DomType: RawEl;
    type Param;
    fn as_element<'a>(
        &self,
        param: Self::Param,
        id: impl Into<Option<&'a str>>,
        class: impl Into<Option<&'a str>>,
    ) -> Self::DomType;
}

//TODO: The generic <T: AsElement> must match the item type of the signal.
// It is a workaround for https://github.com/rust-lang/rust/issues/20400.
// Remove when  that issue is fixed.
pub trait IntoElementWithAttributeSignal<T: AsElement>: Signal<Item: AsElement> {
    fn into_element_with_attribute_signal<'a>(
        self,
        param: <Self::Item as AsElement>::Param,
        id: impl Into<Option<&'a str>>,
        class: impl Into<Option<&'a str>>,
    ) -> <Self::Item as AsElement>::DomType;
}

/// SVG-tags define a canvas. They are the only ones that can have a `ViewBox`.
/// Thus the dom for a `ViewBox` is an svg-tag, that fills the whole parent.
impl AsElement for ViewBox {
    type DomType = RawSvgEl;
    type Param = ();

    fn as_element<'a>(
        &self,
        _param: Self::Param,
        id: impl Into<Option<&'a str>>,
        class: impl Into<Option<&'a str>>,
    ) -> Self::DomType {
        // We use the default preserveAspectRatio="xMidYMid meet".
        // It shrinks the viewBox to fit the view port and centers it.
        let mut el = RawSvgEl::new("svg")
            .attr("version", "1.1")
            .attr("xmlns", "http://www.w3.org/2000/svg")
            .attr("width", "100%")
            .attr("height", "100%")
            .style("display", "block")
            .attr("viewBox", &self.to_svg_string());

        let view_box = RawSvgEl::new("rect")
            .attr("x", &self.view_box().left().to_string())
            .attr("y", &self.view_box().top().to_string())
            .attr("width", &self.view_box().width().to_string())
            .attr("height", &self.view_box().height().to_string());
        el = el.child(view_box);

        if let Some(id) = id.into() {
            el = el.attr("id", id)
        }
        if let Some(class) = class.into() {
            el = el.class(class)
        }
        el
    }
}

impl<S: Signal<Item = ViewBox> + Unpin + 'static> IntoElementWithAttributeSignal<ViewBox> for S {
    fn into_element_with_attribute_signal<'a>(
        self,
        _param: Self::Param,
        id: impl Into<Option<&'a str>>,
        class: impl Into<Option<&'a str>>,
    ) -> Self::DomType {
        // We use the default preserveAspectRatio="xMidYMid meet".
        // It shrinks the viewBox to fit the view port and centers it.

        let signal_broadcaster = Broadcaster::new(self);

        let mut el = RawSvgEl::new("svg")
            .attr("version", "1.1")
            .attr("xmlns", "http://www.w3.org/2000/svg")
            .attr("width", "100%")
            .attr("height", "100%")
            .style("display", "block")
            .attr_signal(
                "viewBox",
                signal_broadcaster.signal_ref(ViewBox::to_svg_string),
            )
            .child_signal(signal_broadcaster.signal_ref(|vb| {
                RawSvgEl::new("rect")
                    .attr("x", &vb.view_box().left().to_string())
                    .attr("y", &vb.view_box().top().to_string())
                    .attr("width", &vb.view_box().width().to_string())
                    .attr("height", &vb.view_box().height().to_string())
                    .style("fill", "none")
                    .style("stroke", "crimson")
            }));

        if let Some(id) = id.into() {
            el = el.attr("id", id)
        }
        if let Some(class) = class.into() {
            el = el.class(class)
        }
        el
    }
}
