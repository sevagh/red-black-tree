Two red-black trees:

1. Allocating nodes from a slab
2. Unsafe mut pointers

The trees take keys representing satellite data of type `T: PartialOrd`. It should be trivial to add values to use as a K/V store.

The pointer implementation is completely unsafe - a real one should use Box or Rc. It was just shoved in for comparisons' sake. The real showcase is the slab implementation which is a neat pattern inspired by https://gist.github.com/stjepang/07fbf88afa824e11796e51ea2f68bd5a and https://www.reddit.com/r/rust/comments/7zsy72/writing_a_doubly_linked_list_in_rust_is_easy/
