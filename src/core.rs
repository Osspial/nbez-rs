n_bezier!{BezCubePoly {
    start: 1,
    ctrl0: 3,
    ctrl1: 3,
    end:   1
} derived {
    ctrl0 - start: 1,
    ctrl1 - ctrl0: 2,
    end   - ctrl1: 1
}}