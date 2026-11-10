---
name: animation-math
description: Reference for Kinetic's animation maths - easing functions, cubic bezier easing, closed-form springs, settle time, interpolation of colours and paths. Use when writing or reviewing code in kinetic-core's easing, spring, lerp or timeline evaluation modules.
---

# Animation maths

All functions are pure and take `f64`. Normalised time `u` is in `[0, 1]`; outputs may leave `[0, 1]` (back, elastic, underdamped springs).

## Easing

- Every easing satisfies `f(0) == 0.0` and `f(1) == 1.0` exactly. Clamp `u` at the call site, then special-case the endpoints if the formula does not hit them exactly.
- `in_out` from an `in`: `u < 0.5 ? in(2u) / 2 : 1 - in(2 - 2u) / 2`. `out(u) = 1 - in(1 - u)`.
- Back: `c1 = 1.70158`, `c3 = c1 + 1`; `out_back(u) = 1 + c3 (u-1)^3 + c1 (u-1)^2`.
- Elastic and bounce: use the standard Penner constants; test against a table of known values.

## Cubic bezier easing

Control points `(0,0), (x1,y1), (x2,y2), (1,1)` with `x1, x2` in `[0, 1]` so x is monotonic.

1. Solve `bx(s) = u` for `s`: Newton from `s = u`, up to 8 iterations, stop when `|bx(s) - u| < 1e-7`.
2. If the derivative is under `1e-6` or Newton leaves `[0, 1]`, fall back to bisection (deterministic, around 30 iterations).
3. Return `by(s)`.

## Springs (closed form)

Unit spring from 0 to 1. Parameters: mass `m` (default 1), stiffness `k`, damping `c`, initial velocity `v0` (units of distance per second, where distance is the normalised 0 to 1 span).

```
w0 = sqrt(k / m)
z  = c / (2 sqrt(k m))
```

| Regime | x(t) |
|---|---|
| `z < 1` | `wd = w0 sqrt(1 - z^2)`; `1 - e^(-z w0 t) (cos(wd t) + ((z w0 - v0) / wd) sin(wd t))` |
| `z == 1` | `1 - e^(-w0 t) (1 + (w0 - v0) t)` |
| `z > 1` | `s = sqrt(z^2 - 1)`, `r1 = -w0 (z - s)`, `r2 = -w0 (z + s)`; `1 + ((r2 + v0) e^(r1 t) - (r1 + v0) e^(r2 t)) / (r1 - r2)` |

- Treat `|z - 1| < 1e-6` as critical to avoid dividing by a tiny `wd` or `r1 - r2`.
- All three satisfy `x(0) = 0` and `x'(0) = v0`. Test both, plus continuity across the regime boundary (values at `z = 1 - 1e-4` and `1 + 1e-4` agree to about `1e-3`).

### Settle time

The spring needs a finite duration. With tolerance `eps` (default `1e-3`):

- Underdamped and critical: amplitude envelope is about `A e^(-z w0 t)`, with `A = sqrt(1 + ((z w0 - v0) / wd)^2)` (use `A = 1 + |w0 - v0| t`-aware bound for critical; a few fixed-point iterations of `t = ln(A(t) / eps) / w0` is enough).
- Overdamped: dominated by the slow root `r1`; `t = ln(|coef1| / eps) / |r1|`.
- Round the result up to the next frame boundary only at export or preview time, not in core.
- At `t >= settle` return exactly `1.0`.

### Presets

`gentle (k 120, c 14)`, `wobbly (k 180, c 12)`, `stiff (k 210, c 20)`, `slow (k 280, c 60)`, `snappy (k 400, c 30)`.

### Chaining

When a spring segment starts while a previous one on the same track is still moving, carry velocity: differentiate the previous curve analytically at the hand-off time and convert to the new segment's normalised units (`v0 = v_abs / (to - from)`; if `to == from`, use `v0 = 0`).

## Interpolation

- Scalars and vectors: `a + (b - a) * u`. With `u` outside `[0, 1]` this extrapolates, which is what overshoot needs.
- Colours: lerp in linear premultiplied RGBA. Clamp channels to `[0, 1]` after overshoot, alpha included.
- Angles: stored in radians as plain scalars; `rotate_to` does not take the shortest path (so full turns are expressible). Provide `rotate_shortest_to` separately.
- Paths: point-wise lerp of prepared, equal-length point lists (see the vector crate). Overshoot extrapolates points; that is acceptable and looks right for springs.

## Tests to write

- Endpoints exact for every easing and for springs at settle time.
- Continuity: `|f(u + h) - f(u)| <= L h` on a fine grid, with `L` chosen per function.
- Springs: numerical derivative at 0 equals `v0`; compare against an RK4 integration of `m x'' + c x' + k (x - 1) = 0` at several times to `1e-6`.
- Bezier easing: `CubicBezier(0, 0, 1, 1)` is linear; CSS `ease` matches reference values.
