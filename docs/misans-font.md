# MiSans UI Font

`frontend/assets/fonts/submora-misans-ui-vf.woff2` is a self-hosted UI subset generated from Xiaomi HyperOS `MiSansVF.ttf`.

Source download:
`https://hyperos.mi.com/font-download/MiSans.zip`

Regenerate the subset:

```bash
./scripts/build-misans-subset.sh
```

Requirements:
`curl`, `unzip`, `python3`, and `pyftsubset` from `fonttools` with WOFF2 support.

Subset scope:
printable ASCII, a few shared punctuation glyphs, and the characters currently present in `frontend/src/**/*.rs`.

License note:
the downloaded archive did not include a separate license file when checked on 2026-03-19. Verify Xiaomi's current distribution terms before redistributing this asset outside the project.
