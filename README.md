[![CI](https://github.com/jedisct1/rust-slabigator/actions/workflows/ci.yml/badge.svg)](https://github.com/jedisct1/rust-slabigator/actions/workflows/ci.yml)

# Slabigator

A linked list that doesn't do dynamic memory allocations.

Things it was designed to do:

- Add to the head of the list in O(1) - What you get back is a stable slot number
- Pop from the tail of the list in O(1)
- Delete an element given its slot number in O(1)
- And nothing else.

Dumb, small, maintainable, zero dependencies.

Cargo features:

- `unsafe`: assume that `remove()` will always be called with a valid index. This saves some memory, but has to be used with extreme caution. That feature is not set by default.
