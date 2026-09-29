#!/usr/bin/env python3
"""Generate `assets/fonts/XanhMono-SemiBoldItalic.ttf` from the shipped
`XanhMono-Italic.ttf` (weight 400).

Why: the app's wordmarks ask for `Xanh Mono` + `FontWeight::SEMIBOLD`, but the
vendored family only ships Regular/Italic at weight 400 and GPUI never
synthesizes bold, so the request silently resolved to the 400 italic and the
17px wordmark read thin and blurry on a 1.0-scale display. This script derives
a real SemiBold Italic face so the request resolves to a genuine 600-weight
outline.

How: every glyph outline is dilated by `--offset` font units per side
(skia-pathops stroke of the closed contour, unioned with the original fill),
which thickens stems without touching the italic angle or the monospace
advance widths. Cubic output from the stroker is converted back to TrueType
quadratics; per-glyph hinting is dropped because the outlines it was fitted to
no longer exist.

Licence: Xanh Mono is SIL OFL 1.1 (`assets/fonts/OFL.txt`). The upstream
copyright statement declares no Reserved Font Name, so OFL clause 3 does not
force a rename; the family name is kept as `Xanh Mono` (so the existing
`font_family("Xanh Mono")` request keeps working) and the face is distinguished
by its subfamily name `SemiBold Italic` and `OS/2.usWeightClass = 600`. The
copyright and licence name records are carried over verbatim.

Usage (throwaway venv, no system install):

    python3 -m venv /tmp/fontvenv
    /tmp/fontvenv/bin/pip install fonttools skia-pathops
    /tmp/fontvenv/bin/python scripts/embolden-xanh-mono.py

Re-run this whenever `assets/fonts/XanhMono-Italic.ttf` is upgraded.
"""

from __future__ import annotations

import argparse
import datetime
from pathlib import Path

import pathops
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.recordingPen import DecomposingRecordingPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont

REPO = Path(__file__).resolve().parent.parent
SOURCE = REPO / "assets/fonts/XanhMono-Italic.ttf"
TARGET = REPO / "assets/fonts/XanhMono-SemiBoldItalic.ttf"

# Font units per side at a 1000 upem. The 400 italic's stems measure ~68 units
# (6.8% of the em); +18 per side lands them at ~104 units (10.4%), the usual
# SemiBold band, and turns the 17px wordmark's 1.16px stem into ~1.77px.
DEFAULT_OFFSET = 18.0
REFERENCE_UPEM = 1000

FAMILY = "Xanh Mono"
SUBFAMILY = "SemiBold Italic"
FULL_NAME = f"{FAMILY} {SUBFAMILY}"
POSTSCRIPT_NAME = "XanhMono-SemiBoldItalic"
WEIGHT_CLASS = 600
DESCRIPTION = (
    "SemiBold Italic face derived from XanhMono-Italic.ttf by dilating the "
    "outlines (skia-pathops); see scripts/embolden-xanh-mono.py. "
    "Licensed under the SIL Open Font License, Version 1.1."
)


def embolden_glyphs(font: TTFont, offset: float) -> None:
    """Dilate every glyph outline by `offset` units per side, in place."""
    glyph_set = font.getGlyphSet()
    glyf = font["glyf"]
    width = offset * 2.0

    # Collect first: `glyph_set` reads through to `glyf`, so writing a glyph
    # back mid-loop would feed the already-emboldened outline into the
    # decomposition of every later composite that references it.
    rebuilt = {}

    for name in font.getGlyphOrder():
        glyph = glyph_set[name]

        # `PathPen` cannot decompose components, so flatten them first.
        recording = DecomposingRecordingPen(glyph_set)
        glyph.draw(recording)

        original = pathops.Path()
        recording.replay(original.getPen())
        # `union` adds winding numbers, so a glyph whose outermost contour runs
        # clockwise (as Xanh Mono's do) would cancel against the stroke ring
        # instead of merging with it. Normalising to the outermost-CCW
        # convention also merges the overlapping contours that decomposing a
        # composite leaves behind.
        original.simplify(
            fix_winding=True, keep_starting_points=False, clockwise=False
        )

        stroked = pathops.Path()
        recording.replay(stroked.getPen())
        stroked.stroke(
            width,
            pathops.LineCap.ROUND_CAP,
            pathops.LineJoin.ROUND_JOIN,
            4.0,
        )
        # Round joins/caps come back as Skia conics, which neither the union
        # operator nor TrueType's quadratic-only `glyf` can consume. The
        # tolerance is in font units: 1/1000 em, i.e. 0.017px at 17px.
        stroked.convertConicsToQuads(tolerance=1.0)
        # The inward offset self-intersects wherever a stroke is thinner than
        # the dilation, so resolve the ring before unioning it.
        stroked.simplify(
            fix_winding=True, keep_starting_points=False, clockwise=False
        )

        merged = pathops.Path()
        pathops.union([original, stroked], merged.getPen())
        merged.convertConicsToQuads(tolerance=1.0)

        pen = TTGlyphPen(None)
        merged.draw(Cu2QuPen(pen, max_err=1.0))
        rebuilt[name] = pen.glyph()

    for name, glyph in rebuilt.items():
        glyf[name] = glyph
        glyph.recalcBounds(glyf)
        # Dilation moves each outline's left edge, so the sidebearing has to
        # follow it: `hmtx.lsb` must equal `glyf.xMin`, and fontTools' glyph
        # set (as well as some rasterizers) shifts the outline when it does
        # not. Advance widths stay untouched — the family is monospaced and
        # the wordmark's metrics must not move.
        advance, _ = font["hmtx"][name]
        font["hmtx"][name] = (advance, glyph.xMin)


def retag(font: TTFont) -> None:
    """Rewrite the identity records for the derived SemiBold Italic face."""
    name = font["name"]
    for name_id, value in {
        1: FAMILY,
        2: SUBFAMILY,
        3: f"3.101;NONE;{POSTSCRIPT_NAME}",
        4: FULL_NAME,
        5: f"Version 3.101;{POSTSCRIPT_NAME}",
        6: POSTSCRIPT_NAME,
        10: DESCRIPTION,
        16: FAMILY,
        17: SUBFAMILY,
    }.items():
        name.setName(value, name_id, 3, 1, 0x409)

    os2 = font["OS/2"]
    os2.usWeightClass = WEIGHT_CLASS
    # Italic stays on (bit 0); the Bold bit (bit 5) is left clear — 600 is not
    # a RIBBI bold, and every matcher in the stack reads `usWeightClass`.
    os2.fsSelection = (os2.fsSelection & ~0b100000) | 0b1

    head = font["head"]
    head.macStyle = (head.macStyle & ~0b1) | 0b10  # italic, not bold
    head.modified = int(
        datetime.datetime.now(tz=datetime.timezone.utc).timestamp()
    ) + 2082844800  # Unix epoch -> 1904-01-01, the `head` epoch

    # The original signature no longer covers the modified tables.
    if "DSIG" in font:
        del font["DSIG"]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--offset",
        type=float,
        default=DEFAULT_OFFSET,
        help="outline dilation per side, in units at a 1000 upem",
    )
    parser.add_argument("--source", type=Path, default=SOURCE)
    parser.add_argument("--target", type=Path, default=TARGET)
    args = parser.parse_args()

    font = TTFont(args.source)
    scale = font["head"].unitsPerEm / REFERENCE_UPEM
    offset = args.offset * scale

    embolden_glyphs(font, offset)
    retag(font)
    font.save(args.target)
    print(f"wrote {args.target} (offset {offset:.2f} units/side)")


if __name__ == "__main__":
    main()
