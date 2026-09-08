# DejaVu Sans 2.37 fixture

`DejaVuSans.ttf` is an unmodified font from the upstream DejaVu 2.37 binary
release. `LICENSE` is copied verbatim from that release archive and must travel
with distributed copies of the font.

- Release: https://github.com/dejavu-fonts/dejavu-fonts/releases/tag/version_2_37
- Archive: https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2
- Archive SHA-256: `fa9ca4d13871dd122f61258a80d01751d603b4d3ee14095d65453b4e846e17d7`
- Font path in archive: `dejavu-fonts-ttf-2.37/ttf/DejaVuSans.ttf`
- Font size: 757076 bytes
- Font SHA-256: `7da195a74c55bef988d0d48f9508bd5d849425c1770dba5d7bfc6ce9ed848954`
- License SHA-256: `7a083b136e64d064794c3419751e5c7dd10d2f64c108fe5ba161eae5e5958a93`

The font covers the Latin, combining-mark, Greek, Hebrew, and Arabic fixtures in
`probes/text-candidate-screen/corpus`. The deliberately unsupported CJK and
family-emoji fixture records missing glyphs; this file is not a universal
fallback font. Both screen adapters disable system font discovery and load only
these bytes, so measurements do not depend on installed fonts.
