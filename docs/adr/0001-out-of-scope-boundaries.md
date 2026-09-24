# The crate is an enabler library, not a product

Squeezer models the bridge domain and provides computations — deal generation,
double-dummy solving, simulation, performance analysis — for others to build
products on. It therefore does not own persistence, player identity or
sessions, a UI toolkit, language bindings, or a WASM build; those are
application-layer concerns. Core types are plain, serde-able data so
applications can store and serialize them however they wish.
