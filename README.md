<div align="center">
  <a href="https://github.com/alvarogaiotti/squeezer">
    <img src="./Squeezer logo.jpeg" alt="Logo" width="150" height="150">
  </a>
</div>

# Squeezer

An attempt at a Bridge software suite for deal generation, simulation and performance analysis.
This started as  a reimplementation of [redeal](https://github.com/anntzer/redeal) by Antony Lee in Rust. Based on the super useful [bridge-cards](https://github.com/droundy/bridge-cards) by David Roundy.
Now... the project grew a bit and I'm now trying to turn this into a full fledged crate that can back some
decent software.

### Goals & Ideas:
- [x] Create safe wrappers for all the DDS library
- [x] Find a better way to store shapes for a hand
- [x] Implement simulation of bridge situations
- [ ] Implement analysis of player performance

Now I'm thinking about creating some binaries to do a limited work, like a clone of Leadsolver by Andrew Lee or a performance analysis tool.
