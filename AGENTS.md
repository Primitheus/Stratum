# Stratum: project instructions

Read [docs/handoff.md](docs/handoff.md) before continuing. This workspace now
contains the pinned Horizon source and Android integration for touchscreens and
physical controllers. Upstream guidance is retained in
[docs/upstream/horizon-AGENTS.md](docs/upstream/horizon-AGENTS.md); these Stratum
instructions take precedence, including the prohibition on unauthorized publishing.

## Direction

- Project name: Stratum.
- Base the game on Horizon; reuse suitable Starling Android integration with
  attribution. Do not silently replace this direction with a renamed Starling fork.
- Support Android gameplay, on-screen controls and physical controllers. Mod
  compatibility is an architectural concern, not a promise of universal desktop-mod
  compatibility.
- Preserve external reference checkouts and their uncommitted work.
- Do not publish repositories, releases, issues, or announcements without authorization.

## Portable workspace

- Never hardcode this user's home directory, drive letter, or checkout path.
- Resolve project files from the repository root or the script's own location.
- Configure external tools through environment variables or ignored local settings.
- Keep reproducible state and handoff notes in the repository; don't depend on a chat.
- Do not create linked worktrees or local-path Git alternates for this project.
- Keep downloads, generated assets, logs and caches under ignored `.local/` or `target/`.

## Sources and attribution

- Check licenses at the exact revisions selected before importing code. Current
  Horizon was observed to publish LGPL-3.0, while the reference Starling checkout
  is MIT. The previous checkout's permissive-only contribution policy cannot be
  carried forward unchanged if this project adopts current Horizon.
- Resolve and document that policy and distribution requirements before importing
  current Horizon source. Keep upstream copyright, license and NOTICE files intact.
- Preserve upstream history when bringing in the game; bootstrap files are not a
  reason to flatten it into a new initial source commit.
- Never commit Mojang jars, assets, decompiled code, accounts, signing keys or tokens.

## Verification

- Test games only with disposable player data. Configure fresh HORIZON_HOME,
  HORIZON_LAUNCHER_HOME, XDG_DATA_HOME and XDG_CONFIG_HOME, and set
  HORIZON_LAUNCHER_NO_SELF_UPDATE=1 and HORIZON_NO_AUDIO=1 for automated runs.
- Use separate Android application identity and disposable data for Stratum tests.
- Record the actual device, ABI, game revision and tests performed. A compiler
  check is not an APK link, and an APK link is not a successful device test.
- Do not claim baseline completion until touch and physical-controller operation
  have been verified on Android, or clearly report the remaining device-test blocker.

## Public documentation and privacy

- Public docs describe the product, architecture, usage and reproducible build steps.
- Never publish chat transcripts, personal device details, network addresses,
  session narratives or private test logs. Keep them under ignored `.local/`.
- Local continuation notes, when available, are `.local/private-notes/handoff.md`.
  Read them for ongoing work, but never stage or quote them in public artifacts.
