# nbez-rs [![Build Status](https://travis-ci.org/Osspial/nbez-rs.svg?branch=master)](https://travis-ci.org/Osspial/nbez-rs) [![Version](https://img.shields.io/crates/v/nbez.svg)](https://crates.io/crates/nbez)
A repository that provides various bezier curve types of varying order and dimensionality.

## [Documentation](http://osspial.github.io/nbez-rs/nbez/index.html)

## Usage
To use this crate, simply add the following your `cargo.toml`:

```
[dependencies]
nbez = "0.1"
```

From there, import any of the types you wish into your module, as well as the [`BezCurve`](http://osspial.github.io/nbez-rs/nbez/trait.BezCurve.html)
trait. That trait exposes most of the curve functions, so you won't really be able to do much
without it.