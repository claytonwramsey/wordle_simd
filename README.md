# Clayton's hardware-accelerated Wordle solver

This is an implementation of a very, very fast Wordle grading algorithm, including benchmarks and comparisons.
Documentation will be sparse, as it is mostly a side project; however you are most likely interested in the SIMD implementation at [`src/squeeze.rs`](https://github.com/claytonwramsey/wordle_simd/blob/master/src/squeeze.rs).
For details, review [this blog post](https://claytonwramsey.com/blog/simd-wordle).

## Usage

To calculate the best unconditional two-word opener (in terms of remaining entropy), do the following:

```sh
cargo run --release --bin wordle answers.txt words.txt
```

This will require a nightly version of Cargo and rustc set up.

## License

This code is licensed to you under the GNU Affero General Public License, version 3.
For details, refer to `LICENSE.md`.
