# Structurray
A Rust Library designed to help create [`struct`](https://doc.rust-lang.org/1.58.1/std/keyword.struct.html)s that store many values of the same type. 
These structures, or psuedo-arrays, are useful for increasing storage efficiency in Google Firebase (realtime database)
or other environments that store data as a string-like object but do not support arrays. This is because Structurray uses a [Base62](https://en.wikipedia.org/wiki/Base62) 
algorithm, as outlined in the documentation of 
[`ascii_basing`](https://docs.rs/ascii_basing/latest/ascii_basing), to reduce
the length of identifiers compared to the base-10 naming algorithm usually used by default.

For more information about this library, check its documentation.