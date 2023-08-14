# Rusty-dealer
An attempt at a reimplementation of [redeal](https://github.com/anntzer/redeal) by Antony Lee in Rust. Based on the super useful [bridge-cards](https://github.com/droundy/bridge-cards) by David Roundy.
Now... the project grew a bit and I'm now trying to turn this into a full fledged crate that can back some
decent software.

### Goals&Ideas:
- [] Create safe wrappers for all the DDS library (in progress)
- [] Find a better way to store shapes for a hand:
    - Evaluating using a `u16` to represent full shapes and a `BTreeSet` to represent possible shapes
    - Use 3 `u8` to store two full shapes, the first 12 bits for the first shape and the second 12 for the second shape

I'll add ideas and goals while I come up with them.
