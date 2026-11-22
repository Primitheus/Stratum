# Stratum CPAL patch

CPAL 0.16.0 from the crates.io archive already pinned in Horizon's lockfile.
Archive checksum: cbd307f43cc2a697e2d1f8bc7a1d824b5269e052209e28883e5bc04d095aaa3f.
Upstream git revision from .cargo_vcs_info.json:
36b9fed079e3d3066acaa58b1ae935caf5c2bfb1.
License: Apache-2.0, preserved in LICENSE and the APK license directory.

Only src/host/aaudio/mod.rs changes behavior: output requests AAudio low-latency
performance mode, prefers supported stereo float 48 kHz, and requests a two-burst
buffer after opening. The requested mode and actual rate/burst/buffer are logged.
Input and non-Android backends keep upstream behavior. Android may clamp requests;
this does not promise a particular Bluetooth transport latency. Game audio remains
muted in automated runs by HORIZON_NO_AUDIO. Do not edit registry cache sources.
