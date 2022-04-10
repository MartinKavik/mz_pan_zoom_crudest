# MoonZoon Pan Zoom Test

A first attempt at zooming with MoonZoon, to see, the API needs for the use case. 

Zooming is implemented using Ctrl+ Mouse Wheel. Panning is not implemented yet. Neither are any other Ui-devices.

It contains a couple of stripped down abstractions from my private code for dealing with SVG and HTML geometry. One trait ***requires a nightly feature***.

It creates an SVG of four circles and shows the current view box.

The API in this code is abysmal:
1. Elements are cloned very often, as MoonZoon only knows that an element is an HTML or an SVG element. We need to take ownership and cast elements from general `SVGElement` to `SvgsvgElement` (or more generally `SvgGraphicsElement`).
2. Methods require the state -- actually a reference to the Mutable holding the state -- and the element created from it. And I have not found a way to ensure on the type level, that they have any bearing on each other. The only I can do is implement an assertion.
3. Changes are update to SVG only in the next animation frame. Making an assertion about the effect of the change can thus happen only two animation frames after the update, requiring calls to `request_animation_frame`.

Also, there is a bug in the assertion. When zooming too small or too big, the assertion fails. But that isn't an API problem. That's just a plain bug. 


## Other notes
A couple of PointerEvents are missing
* pointerover
* pointerenter
* pointerout
* gotpointercapture
* lostpointercapture

I don't know if they are necessary, but just having half an API implemented is wierd. See https://developer.mozilla.org/en-US/docs/Web/API/Pointer_event

 