# fmc brand

## The mark

A sidebar next to a prompt, on a dark tile. The lighter strip on the left is fmc's file list, with its selected row in indigo and two rows dimmed. The right side is a terminal prompt: a `>` and a block cursor in place of the underscore. The selected row lines up with the tip of the `>`: what you select is what opens.

The shapes are few and heavy so the mark reads at the size of a tab or sidebar icon (16 to 24 px). A dark tile nearly matches a dark interface, so an indigo rim draws its edge there. Below 32 px only the shapes that survive are kept: the strip, the selected row, the `>`, and the cursor.

The wordmark is lowercase `fmc`, drawn as round-capped strokes in the weight of the `>`, in one solid colour.

## Colours

All colours come from fmc's default palette (`src/theme/mod.rs`).

| Role | Hex | Palette token |
| --- | --- | --- |
| Tile, top | `#262b46` | `target-bg` (dark) |
| Tile, bottom | `#131519` | `fallback-background` (dark) |
| Sidebar | `#2c365e` | `selection-bg` (dark) |
| Rim | `#818cf8` at 45% | `accent` (dark) |
| Selected row | `#818cf8` | `accent` (dark) |
| Dimmed rows | `#c7d2fe` at 30% | `accent-soft` (dark) |
| `>` | `#c7d2fe` → `#a5b4fc` | `accent-soft`, first `wordmark` stop (dark) |
| Cursor | `#c7d2fe` | `accent-soft` (dark) |
| Icon shadow and lower rim | `#0f1117` | `accent-ink` (dark) |
| Wordmark on dark grounds | `#d4d8de` | `text` (dark) |
| Wordmark on light grounds | `#1f2328` | `text` (light) |

Contrast (WCAG ratio):

| Pair | Ratio |
| --- | --- |
| `>` and cursor on the tile, weakest point | 9.17 |
| Selected row on the sidebar | 3.92 |
| Tile on `#ffffff`, top and bottom | 13.85, 18.28 |
| Tile on `#0d1117`, top and bottom | 1.37, 1.04 |
| Rim on `#0d1117` (before its 45% opacity) | 6.34 |
| Dark wordmark on `#0d1117` | 13.23 |
| Light wordmark on `#ffffff` | 15.80 |

On a dark ground the tile itself does not separate (1.04 to 1.37); the rim and the lighter sidebar carry the edge, so keep both.

## Clear space and minimum size

- Clear space: a quarter of the mark's height on every side of the mark or the logo. Nothing else enters it.
- Mark: 16 px at the smallest. At 16 px use `mark-16.svg` (or the PNGs and favicons made from it), not the large mark scaled down.
- Logo: 120 px wide at the smallest. Below that, use the mark alone.
- Do not recolour, outline, rotate, or stretch the mark, and do not set the wordmark in a font.

## Files

| File | Use |
| --- | --- |
| `mark.svg` | The mark alone, edge to edge, for 24 px and up. Docs, slides, social images. |
| `mark-16.svg` | The mark drawn on a 16 px grid, every edge on a whole pixel. For 16 px only. |
| `icon.svg` | The macOS app icon source: the mark on the Big Sur grid (824 px tile in a 1024 px canvas), with its shadow and rim light. |
| `logo.svg` | Mark and wordmark. Follows `prefers-color-scheme`: the light wordmark by default, the dark one in dark mode. |
| `logo-light.svg`, `logo-dark.svg` | The logo for a light ground and for a dark ground. |
| `png/logo-light.png`, `png/logo-dark.png` | The logo at 588 × 192, and `@2x` at 1176 × 384, for a README header. |
| `png/icon-<size>.png` | The app icon at 16 to 1024 px. 16 px comes from `mark-16.svg`, the rest from `icon.svg`. |
| `fmc.icns` | The macOS app icon, built with `iconutil` (16 to 512, each with `@2x`; 16 from `mark-16.svg`). |
| `apple-touch-icon.png` | 180 × 180, opaque and square; iOS rounds the corners. |
| `favicon.svg` | Browser tab icon, the same drawing as `mark.svg`. |
| `favicon.ico` | 16 (from `mark-16.svg`), 32, and 48 px, for browsers that skip the SVG. |
| `favicon-32.png` | 32 px PNG favicon. |

On GitHub, `logo.svg` follows the reader's system setting, not GitHub's theme, so the repository README uses the two PNGs in a `<picture>` element.

## Sources

The SVGs are hand-written: one viewBox each, no editor metadata, no raster, no fonts. The PNGs, `fmc.icns`, and the favicons are rendered from them through Quick Look (`qlmanage`), once on black and once on white, with the alpha recovered from the pair in `magick`. `magick` alone cannot render them here: its SVG delegate, `rsvg-convert`, is not installed, and its built-in renderer drops gradients. Re-render after any change to an SVG.
