# Slabigator

A linked list that doesn't do dynamic memory allocations.

Things it was designed to do:

- Add to the head of the list in O(1) - What you get back is a stable slot number
- Pop from the tail of the list in O(1)
- Delete an element given its slot number in O(1)

Simple, small, readable.