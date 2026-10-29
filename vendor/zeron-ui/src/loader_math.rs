//! Loader motion — the pure math behind zeron's loading indicators.
//!
//! Ported from zeron's `crates/proto/src/motion.rs` (MIT, @ 8b7afb3) because
//! `zeron-ui/src/motion.rs` re-exports these phases for its pulse/spinner
//! helpers. Upstream they live in the `zeron_proto` domain crate (protocol and
//! view models); mono-desk only needs the pure functions, so they live here
//! unchanged.

/// Zeron loader pulse period.
pub const ZERON_PULSE_MS: u64 = 2_400;
/// Gradient matrix spinner wave period.
pub const GRADIENT_SPIN_MS: u64 = 750;

/// Cells in the zeron wave loader.
pub const ZERON_CELLS: usize = 5;
/// Side length of the gradient spinner matrix.
pub const MATRIX_SIDE: usize = 3;

/// Zeron loader cells rest at this opacity between pulses.
pub const PULSE_MIN_OPACITY: f32 = 0.08;
/// …and at this scale.
pub const PULSE_MIN_SCALE: f32 = 0.9;
/// Per-cell stagger, as a fraction of the pulse period (0.15s of 2.4s).
pub const PULSE_STAGGER: f32 = 0.15 / 2.4;

/// Per-row tints of the gradient matrix spinner — zeron's "sunrise" gradient
/// sampled at each row: cool blue at the top, through amber, to pink.
pub const GSPIN_ROW_TINTS: [u32; MATRIX_SIDE] = [0xB6D3EF, 0xEDB185, 0xF888A0];
/// Opacity a gradient-spinner cell rests at between pulses.
pub const GSPIN_DIM: f32 = 0.1;

/// Clockwise ring position of each `(row, col)` cell of the 2×3 mini spinner,
/// top-left first: (0,0) → (0,1) → (1,1) → (2,1) → (2,0) → (1,0). Every cell of
/// a 2×3 grid is on the ring, so the brightness chases around it.
pub const MINI_RING: [[usize; 2]; 3] = [[0, 1], [5, 2], [4, 3]];
/// Cells in the mini spinner's ring.
pub const MINI_RING_LEN: f32 = 6.0;

/// Linear interpolation.
pub fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

/// A cell's phase, given the loader's raw phase and the cell's index.
pub fn staggered_phase(raw_delta: f32, index: usize, stagger: f32) -> f32 {
    (raw_delta - index as f32 * stagger).rem_euclid(1.0)
}

/// Cosine pulse: 0 at phase 0, 1 at phase 0.5, back to 0 at phase 1.
pub fn pulse_wave(phase: f32) -> f32 {
    0.5 - 0.5 * (phase * std::f32::consts::TAU).cos()
}

/// Zeron loader cell opacity for a phase: 0.08 → 1 → 0.08.
pub fn pulse_opacity(phase: f32) -> f32 {
    PULSE_MIN_OPACITY + (1.0 - PULSE_MIN_OPACITY) * pulse_wave(phase)
}

/// Zeron loader cell scale for a phase: 0.9 → 1 → 0.9.
pub fn pulse_scale(phase: f32) -> f32 {
    PULSE_MIN_SCALE + (1.0 - PULSE_MIN_SCALE) * pulse_wave(phase)
}

/// Gradient-spin cell opacity for a local phase `t` (0..1 of the period),
/// ported from zeron's `gradient-spin-pulse` keyframes: full at the cycle
/// start, easing down to `dim` by 45%, resting at `dim` until 92%, then rising
/// back to full — the per-cell phase offset sweeps this pulse across the grid.
pub fn gspin_opacity(t: f32, dim: f32) -> f32 {
    let t = t.rem_euclid(1.0);
    if t < 0.45 {
        lerp(1.0, dim, t / 0.45)
    } else if t < 0.92 {
        dim
    } else {
        lerp(dim, 1.0, (t - 0.92) / 0.08)
    }
}

/// The phase offset of a `(row, col)` cell in the 3×3 gradient spinner: the
/// pulse enters at the bottom edge and converges toward the top-centre cell, so
/// the wave reads as travelling upward.
pub fn gspin_cell_phase(row: usize, col: usize) -> f32 {
    let centre = (MATRIX_SIDE as f32 - 1.0) / 2.0;
    let max = MATRIX_SIDE as f32 - 1.0 + centre;
    let d = MATRIX_SIDE as f32 - 1.0 - row as f32 + (col as f32 - centre).abs();
    if max == 0.0 { 0.0 } else { d / (max + 1.0) }
}

/// The zeron mark's pixels — `[x, y]` of each 100×100 cell on the 820×940
/// canvas (zeron's `logo.tsx` CELLS), shared by the static mark and the
/// animated loader.
#[rustfmt::skip]
pub const MARK_CELLS: [(f32, f32); 34] = [
    (0., 600.), (0., 720.), (240., 840.), (240., 720.), (120., 840.), (120., 600.), (240., 600.),
    (0., 480.), (0., 360.), (480., 840.), (480., 720.), (120., 360.), (120., 240.), (240., 360.),
    (600., 720.), (480., 600.), (360., 360.), (240., 240.), (600., 600.), (720., 600.), (720., 480.),
    (240., 120.), (600., 380.), (720., 240.), (720., 0.), (480., 240.), (480., 0.), (120., 480.),
    (240., 480.), (360., 840.), (360., 720.), (360., 600.), (360., 480.), (120., 720.),
];

/// Fraction of the pulse cycle the mark's light sweep occupies.
pub const MARK_SPREAD: f32 = 0.55;

/// Per-cell stagger along the zeron's flight axis. The stagger *adds* phase
/// (the original uses a negative CSS delay, starting the cell mid-cycle), so a
/// larger value means the cell is further along and therefore **leads**: the
/// tail tip `(720, 0)` leads at `MARK_SPREAD`, the head `(0, 840)` trails at 0.
pub fn mark_cell_stagger(x: f32, y: f32) -> f32 {
    let t = (820.0 - x + y) / 1660.0;
    (1.0 - t) * MARK_SPREAD
}

/// A mark cell's phase at loader phase `delta`.
pub fn mark_phase(delta: f32, x: f32, y: f32) -> f32 {
    (delta + mark_cell_stagger(x, y)).rem_euclid(1.0)
}
