# MoonZoon Pan Zoom Test

A first attempt at zooming with MoonZoon, to see, the API needs for the use case.

Zooming is implemented using Ctrl+ Mouse Wheel. Panning is not implemented yet. Neither are any other Ui-devices.

It contains a couple of stripped down abstractions from my private code for dealing with SVG and HTML geometry. One
trait ***requires a nightly feature***.

It creates an SVG of four circles and shows the current view box.

The API in this code is abysmal:

1. Elements are cloned very often, as MoonZoon only knows that an element is an HTML or an SVG element. We need to take
   ownership and cast elements from general `SVGElement` to `SvgsvgElement` (or more generally `SvgGraphicsElement`).
2. Methods require the state -- actually a reference to the `Mutable` holding the state -- and the `RawEl` created from
   it. And I have not found a way to ensure (on the type level) that they have any bearing on each other. The only way I
   can do is implement an assertion. Instead, there should be a way for a `Mutable` (or the state inside it) to know
   which `RawEl` is created from it and access it. This could be done by saving an element id in the state and ask the
   browser for the element, but it should be possible without crossing the Rust-JS barrier twice.
3. Changes are update to SVG only in the next animation frame. Making an assertion about the effect of the change can
   thus happen only two animation frames after the update, requiring calls to `request_animation_frame`.

Also, there is a bug in the assertion. When zooming too small or too big, the assertion fails. But that isn't an API
problem. That's just a plain bug.

## Other notes

A couple of PointerEvents are missing

* pointerover
* pointerenter
* pointerout
* gotpointercapture
* lostpointercapture

Some of these will be necessary for notification of entering and leaving dragable areas. Anyway, just having half an API
implemented is wierd. See https://developer.mozilla.org/en-US/docs/Web/API/Pointer_event

## Install & Run

1
```
cargo install mzoon --git https://github.com/MoonZoon/MoonZoon --rev d80b1faec1b54fc702149f821825ddb419b51c27 --root cargo_install_root --locked
```

2
```
mv cargo_install_root/bin/mzoon mzoon
# or
move cargo_install_root/bin/mzoon mzoon
```

3
```
./mzoon start -o
# or
mzoon start -o
```

 