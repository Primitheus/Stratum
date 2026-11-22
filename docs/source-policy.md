# Stratum source policy

Stratum uses the pinned Horizon game under LGPL-3.0-or-later, with selected
MIT-licensed Starling Android integration. Covered Stratum changes retain the
Horizon license; imported components retain their original notices.

## Reviewed sources

Exact revisions live in [upstream-lock.json](../upstream-lock.json).

- Horizon `07296d9eff2663e6014420b35d5d2147d7d92d8b`: root `LICENSE` is
  LGPL version 3. `Cargo.toml` and `CONTRIBUTING.md` specify
  **LGPL-3.0-or-later**. Preserve `LICENSE`, `LICENSES/`, and
  `THIRD_PARTY_NOTICES.md`, including the MIT license for historical code.
- Starling `5432370d573642041585c69b40fb045b27133488`: root `LICENSE` is
  MIT, copyright (c) 2026 Ludovic Marie and the Horizon contributors.
  Preserve that full notice alongside any copied portions, plus file-specific
  attribution and applicable third-party notices. Review each imported file's
  notices; a root license is not a substitute for a dependency audit.

New Stratum changes to the covered game use LGPL-3.0-or-later. MIT source retains
its MIT notices; integrating it does not make the Horizon game MIT-licensed.
Do not replace upstream license texts or remove copyright lines. Record copied
paths, source revision, authors/notices, and adaptations in a provenance ledger
when each integration is made.

## Distribution requirements

Before distributing an APK, prepare the exact corresponding source and build
scripts, modifications and notices, with both GPLv3 and LGPLv3 license texts.
Provide recipients a practical way to rebuild/relink the covered code and install
their modified build. Include any required installation information; do not impose
terms that prohibit modification or reverse engineering for debugging those changes.
For statically linked combined works, source/application code and build materials
must support recombination with a modified covered library; merely linking to an
upstream repository is insufficient. Audit bundled dependencies and their notices
at the versions actually shipped. This is a release gate, not a claim that a
distribution package is already compliant.

Never distribute Mojang jars, extracted assets, decompiled code, credentials or
signing secrets in this repository. Player-provided resources remain separate.
No release signing key is selected or generated during bootstrap.

## History and integration

Base the source branch on the full pinned Horizon history. Preserve the bootstrap
documents as a subsequent Stratum change, retaining upstream versions of any
overlapping documentation for reference. Do not flatten Horizon into a source
snapshot or base Stratum on a renamed Starling checkout. Reference clones under
`.local/` are disposable independent clones, never Git alternates or worktrees.

## Primary references

- [Pinned Horizon license](https://github.com/horizon-rust/horizon/blob/07296d9eff2663e6014420b35d5d2147d7d92d8b/LICENSE)
- [Pinned Horizon contribution/license policy](https://github.com/horizon-rust/horizon/blob/07296d9eff2663e6014420b35d5d2147d7d92d8b/CONTRIBUTING.md)
- [Pinned Starling license](https://github.com/starling-client/starling/blob/5432370d573642041585c69b40fb045b27133488/LICENSE)
- [LGPLv3 terms](https://www.gnu.org/licenses/lgpl-3.0.html)
- [GPLv3 terms incorporated by LGPLv3](https://www.gnu.org/licenses/gpl-3.0.html)
