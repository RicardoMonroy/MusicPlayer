# Real-time spectrum spike

2026-10-18. Research only; no playback/UI changes. Read `CLAUDE.md` and
`docs/PLAN.md` first. Recommendation: use a labelled mpv lavfi filter bank,
initially **19 stereo power bands**, shared by desktop and TUI. It works through
our existing libmpv2 API without capturing the system mix. Treat synchronization
with audible output as a remaining acceptance test, not a solved property of
`af-metadata`. Do not promise sample-accurate visualization from this property.

## Environment and evidence

Actual local tests: mpv/libmpv 0.41.0, libmpv2 6.0.0, FFmpeg n9.0.1
(libavfilter 12.1.101), Linux x86_64, i7-14700. mpv's version alone does not
specify its available FFmpeg filters. Windows/macOS packages need the same
filter capability check. No network was used; cross-platform capture notes
below are design guidance, not platform validation.

All playback probes used `ao=null`, `vo=null`, `video=no`. Sources were
`av://lavfi:anoisesrc=sample_rate=48000` and
`av://lavfi:sine=frequency=1000:sample_rate=48000`, including stereo and
opposite-phase stereo variants. No probe sent sound to speakers.

Prototypes and logs are in `/tmp/mp-visualizer-spike` (temporary, not repository
artifacts):

- `probe.py`: ctypes calls to the installed **libmpv**, not an IPC approximation;
  reads `MPV_FORMAT_NODE` maps, frees nodes, polls every 10 ms.
- `batch.py`, `extra.py`, `rate.py`, `threads.py`, `stereo.py`, `final_probe.py`:
  graphs, rate/CPU/PTS experiments. `results.log`, `extra.log` hold raw results.
- `validate.py`, `validate.log`: PCM comparison and FFT processing prototype.
- `rust/`: an independently built offline Cargo binary using libmpv2 **6.0.0**;
  successfully read `af-metadata/viz/by-key/lavfi.astats.1.RMS_level` as `String`,
  parsed `-21.101784` dBFS. Build output: `rust.log`.

CPU figures are process CPU time / wall time, expressed as percent of **one
logical CPU**, including Python polling and mpv threads. Short 1–3 s samples,
0.5 s warmup, variable scheduling and CPU frequency; indicative, not a benchmark
of Rust production code. Update rates count changed metadata snapshots, not
all internally processed frames. Constant tones can produce identical snapshots.

## Results at a glance

| Approach | Works in tested libmpv? | Observed metadata rate | CPU, one logical CPU | Main limitation |
| --- | --- | --- | --- | --- |
| No analysis | Yes | N/A | ~0.6% | Baseline including property polling |
| `astats`, RMS only | Yes | ~20–21/s, default null buffering | ~1.6–1.9% | Broadband level, no spectrum |
| `ebur128=metadata=1` | Yes | ~10/s | ~0.9–1.2% | Loudness windows, no bands |
| `aspectralstats` | Yes | ~20–21/s | ~1.3–2.4% | Spectral descriptors, no bin array |
| 16 mono bandpass channels + dry channel | Yes | ~20/s default; 25/s in 40 ms test | ~7–9% auto threads; ~3.8% one filter thread | Downmix can cancel opposite-phase stereo |
| 32 mono bands + dry channel | Yes | ~19–20/s default | ~10–13% auto threads; ~6.8% one filter thread | Same downmix issue; more work |
| 16/19 stereo bands + dry stereo | Yes | ~20–21/s default; ~25/s framed small-buffer test | ~4.9% / ~7.8% one thread on tone | Latest metadata runs ahead of sound |
| 32 stereo bands + dry stereo | **No**, naive merged graph | N/A | N/A | 66 channels exceeds `amerge`'s 64-channel maximum |
| `afftfilt=real=re:imag=im` | Audio passes | No metadata keys | ~1.6% | FFT coefficients stay inside expression evaluator |
| `showfreqs` in `af` | **No** | N/A | Not measured | Audio-to-video, rejected with `mismatching media type` |
| External capture + FFT | DSP tested; live capture blocked | 25/s DSP feasible | ~52 µs/FFT frame, ~0.13% at 25/s, DSP only | OS backend, routing, permissions and mixed audio |

## (a) Audio-frame metadata

A minimal working filter:

```sh
mpv --no-config --ao=null --vo=null --video=no \
  '--af=@viz:lavfi=[astats=metadata=1:reset=1:measure_perchannel=RMS_level:measure_overall=none]:o=[threads=1]' \
  'av://lavfi:anoisesrc=sample_rate=48000'
```

Read `af-metadata/viz` as a native map or individual string properties under
`af-metadata/viz/by-key/`. `viz` is the **mpv filter label**, not an internal
FFmpeg branch label. Metadata values are strings, including `-inf` for silence.
`reset=1` gives frame-local RMS; the default accumulates statistics and is
unsuitable for moving bars. `length=0.04` alone does not set a 25 Hz frame rate.
Selecting only RMS avoids expensive unrelated statistics.

`ebur128=metadata=1` returned `lavfi.r128.M`, `.S`, `.I`, `.LRA` and range
metadata, in 100 ms frames. Momentary loudness integrates roughly 400 ms;
short-term integrates 3 s and integrated loudness spans the program. Useful for
meters, not a 25 Hz spectrum. It also reframes audio into 100 ms blocks.

### A working filter bank that preserves playback

Independent `bandpass,astats` branches do not automatically combine metadata
into one dictionary: each `astats` reuses the same keys and joins are not a
metadata reduction API. Running bandpasses in sequence also changes the music.
The tested solution merges **analysis samples into temporary channels**, runs one
`astats`, then selects only the dry channels for output. Metadata survives that
final `pan`. Only samples on the dry path reach the output.

This generator describes the tested stereo topology (19 bands = 40 temporary
channels). It belongs in a future implementation, not this spike's product code:

```python
def graph(n=19):
    edges = [40 * (16000 / 40) ** (i / n) for i in range(n + 1)]
    parts = [f'aformat=channel_layouts=stereo,asplit={n + 1}[dry]'
             + ''.join(f'[b{i}]' for i in range(n))]
    for i, (lo, hi) in enumerate(zip(edges, edges[1:])):
        parts.append(f'[b{i}]bandpass=f={(lo * hi) ** .5}:t=h:w={hi-lo}[f{i}]')
    parts.append('[dry]' + ''.join(f'[f{i}]' for i in range(n))
                 + f'amerge=inputs={n+1},'
                 'astats=metadata=1:reset=1:measure_perchannel=RMS_level:'
                 'measure_overall=none,pan=stereo|c0=c0|c1=c1')
    return ';'.join(parts)
# mpv af value: '@viz:lavfi=[' + graph() + ']:o=[threads=1]'
```

Because every branch has overlapping stereo layouts, the tested `amerge`
concatenates channels in input order. Metadata channels 1/2 are dry L/R;
band index `i` uses **3+2*i and 4+2*i**. Combine channel powers, not signed
samples: `power = (10^(left_db/10) + 10^(right_db/10))/2`, then
`band_db = 10*log10(power)`. This avoids opposite-phase cancellation.

Verified: a 1 kHz tone peaks in the expected mono band 8 of 16 (965 Hz centre),
about -21.26 dBFS. Stereo analysis also retained opposite-phase tone energy.
A separate FFmpeg comparison of dry versus analyzed s16 stereo output produced
192,000 bytes each, **byte-identical**, even with opposite-phase channels.
This validates sample routing for that format, not every future conversion.
The first mono bank prototype is not suitable as the general stereo solution.

These are overlapping biquad filter energies, not rectangular FFT bins.
Low bands have longer settling times; white noise rises toward wider high bands.
Keep that energy interpretation or explicitly calibrate bandwidth before claiming
spectral density. Above-Nyquist bands must be omitted/zeroed or analysis resampled
to a fixed supported rate. Do not silently force all playback to 48 kHz just for
visualization. Stereo coercion also changes multichannel input: restrict the
first implementation to mono/stereo, or preserve the dry layout explicitly.

For 32 stereo bands, avoid the failed 66-channel graph: two labelled 16-band
stages can preserve dry stereo and expose separate maps (not prototyped), or use
a different reduction topology. Initially 19 bands already match Classic's 19
bars and meet the requested 16–32 range.

### Rate and audible synchronization

Default timed `ao=null` delivered approximately 20 bursts/s, even when FFmpeg
could generate more frames. Setting only `audio-buffer=0.04` did not fix that.
A successful 25/s test used `asetnsamples=n=1920:p=0` before the bank at 48 kHz,
plus these **test** options:

```text
ao=null
ao-null-buffer=0.04
ao-null-outburst=480
audio-buffer=0.04
```

The final 19-band stereo noise run with 1920-sample framing measured 25.3
changed snapshots/s and ~7.8% of one logical CPU over 3 s.
Without `asetnsamples`, a stereo bank reached ~47 changed snapshots/s with the
smaller simulated buffer. Publish to the UI at 25 Hz regardless; audio frame
size and output scheduling govern actual independent analysis samples. The
`asetnsamples` window introduces up to 40 ms framing wait at this sample rate.
Do not hardcode 1920 for 44.1 kHz. One graph thread (`:o=[threads=1]`, including
the brackets) reduced overhead noticeably on this many-core host.

Crucially, these properties expose the **latest filtered frame**, not the frame
currently audible. `ametadata=mode=print:direct=1:file=...` exposes frame PTS for
measurement: at the end of a default-null run, the newest frame PTS was
**~392 ms ahead of `time-pos`**. This is one measured snapshot, not a universal
latency constant. More output buffering/Bluetooth can enlarge the difference.
Filter group delay, framing, the 40 ms UI tick and display smoothing add other
terms. Small null-buffer results do not validate a real device's latency.

The metadata map itself has no guaranteed frame PTS. `audio-pts` accounts for
driver delay but cannot retrospectively timestamp the latest map. For a casual
visualizer, measure and accept bounded lead, potentially delaying snapshots in
a bounded queue after device-specific measurement. A fixed 392 ms compensation
is not portable. Accurate synchronization needs timestamped frame transport
(e.g. a custom filter/PCM tap); logging metadata to a pipe is a possible research
instrument, not the recommended production transport. Never shrink audio buffers
just to make bars smoother: mpv documents `audio-buffer` as a testing option.

Pause/stop/seek/track changes can leave stale data. Clear display history on
those transitions, decay paused bars, and treat missing metadata as unavailable.
Volume, mute, ReplayGain, speed filters and OS gain require explicit semantics:
this is energy at the analysis filter position, not guaranteed post-device energy.
If desired, scale display for player mute/volume; do not claim it measures the
physical loudspeaker signal.

## (b) Filters with frequency information

`aspectralstats` performs spectral analysis and successfully returns channel
mean/variance, centroid, spread, skewness, kurtosis, entropy, flatness, crest,
flux, slope, decrease and rolloff. There is **no per-frequency bin array** in
its metadata. Those summaries cannot reconstruct a spectrum. Its default 2048
sample window is ~42.7 ms at 48 kHz, default 50% overlap; visible updates still
arrived in output-driven bursts. Cross-platform risk is mostly filter availability
in the packaged FFmpeg; latency has the same pre-output issue as `astats`.

`afftfilt` exposes `re`, `im`, bin index and bin count to internal expressions,
then transforms back to audio. Identity expressions worked but returned an empty
metadata map. Expression logging is not a stable 32-value frame API. It also
adds FFT buffering/windowing; do not insert it expecting an exported FFT.

`showfreqs`, `showspectrum`, `showcqt` and `showcwt` produce spectrum **video**.
Direct `showfreqs` in `af` failed as expected. A standalone FFmpeg pipeline or
mpv `lavfi-complex` can route an audio split to video while retaining audio,
but that changes the audio-only configuration (`video=no` currently), requires
video frame extraction/rendering and still yields pixels rather than band
values. `vo=null` discards rendered output. No shared TUI numeric API follows
for free. CPU/latency for that alternative were not measured; FFT windows,
video scheduling, rendering/readback and payload size make it a worse first
choice. All these use bundled FFmpeg, so OS portability is better than capture
but build portability still needs testing.

The installed public mpv headers provide video rendering and stream input APIs,
not a general decoded/output PCM callback. libmpv2 cannot invent such a callback.
A custom libmpv filter/tap would be a separate integration and packaging project.

## (c) Capture outside mpv, then FFT

Processing prototype: FFmpeg-generated 48 kHz f32 PCM, 2048-sample Hann window,
real FFT, sum power into 16 logarithmic 40 Hz–16 kHz bands, 1920-sample hop.
The 1 kHz source peaked in band 8. NumPy processing took ~52 microseconds/frame
for 1000 iterations (~0.13% of one CPU at 25/s). That measures **DSP only**,
not capture, copying, resampling, Rust performance or IPC. A 2048-sample window
covers 42.7 ms; lowest bands have poor FFT resolution (~23.4 Hz/bin). A larger
window improves bass discrimination at the cost of latency.

Production shape: capture callback writes interleaved PCM to a bounded ring;
worker combines per-channel FFT **powers**, computes bands every 40 ms, publishes
only the latest frame. Never allocate, lock the UI, run FFTs or perform IPC in
the realtime callback. Handle device format changes, silence and ring overruns.

| Platform/backend | Capture strategy and scope | Risk / latency |
| --- | --- | --- |
| Linux PipeWire | Capture sink monitor, or target mpv's playback node monitor to isolate this player. Pulse compatibility exposes sink `.monitor` sources. | Node/sink changes, channel layout, permissions, PipeWire/Pulse availability; sink monitor includes other apps. Clock/graph quantum and FFT window govern delay. |
| Linux CPAL | Ordinary ALSA input is not automatically a playback monitor. Needs an exposed loopback/monitor device or another backend. | CPAL alone is not a portable system-output capture solution. No cached CPAL crate was present for a local backend test. |
| Windows WASAPI | Shared-mode render-endpoint loopback with `AUDCLNT_STREAMFLAGS_LOOPBACK`; process loopback is a separate, newer Windows capability to investigate. | Endpoint capture generally mixes apps; exclusive modes/device changes complicate it. Verify the chosen CPAL version's loopback implementation rather than assuming its common input API supports it. No Windows run here. |
| macOS | ScreenCaptureKit system audio, or Core Audio process taps on supported newer macOS; virtual audio device for older setups. | Native integration, OS deployment floor, capture permissions and routing; generic CPAL CoreAudio input does not automatically supply speaker audio. No macOS run here. |

Live Linux capture was attempted: `pactl info` was refused and `pw-record`
failed with `pw_context_connect() failed: Operation not permitted`. Thus **live
monitor capture, its CPU and end-to-end latency are unverified** in this sandbox.
With `ao=null`, mpv creates no audible playback stream to capture anyway.
A valid later silent integration test should route mpv to a dedicated virtual
sink and capture its monitor without linking that sink to physical speakers.
It requires a reachable audio server; it was not performed here.

Example monitor-reader prototype command (replace the target after enumeration):

```sh
pw-record --target '<sink-node-name>' \
  -P '{ stream.capture.sink = true }' \
  --rate 48000 --channels 2 --format f32 --latency 20ms --raw -
```

This supplies raw bytes to the DSP above; it is not a proven capture command in
this sandbox. A render monitor is closer to output timing than decoder metadata,
but is not necessarily post-hardware-volume and does not include physical device
or Bluetooth latency. Expect a capture quantum + analysis window + up to 40 ms
publication wait, plus backend-dependent timing. No measured millisecond figure
is justified. An endpoint monitor also visualizes other applications, which is a
functional mismatch unless isolation is configured. Overall cross-platform risk
is substantially higher than the lavfi bank.

## Recommendation and proposed API

Implement 19 stereo bands with a labelled lavfi bank first, enabled only while
a visualizer is visible. Preserve dry audio, use one filter thread, fail open
when filters cannot initialize, and expose availability separately from silence.
Run a real-device timing acceptance test before shipping; if synchronized output
cannot be made acceptable, revisit a timestamped tap rather than making OS-wide
capture the default. Keep capture an optional platform-specific fallback.

Conceptual additions to `Player` (sketch only):

```rust
pub struct Spectrum {
    pub available: bool,
    pub bands: Vec<f32>, // fixed 19 values, low to high, normalized 0..1
}

impl Player {
    pub fn set_visualizer_enabled(&self, enabled: bool) -> Result<(), Error> {
        // Add/remove only @viz; preserve any other audio filters.
        todo!()
    }

    pub fn spectrum(&self) -> Spectrum {
        // Return unavailable/zeros when disabled, idle, or the filter failed.
        // The caller clears/decays on playback transitions.
        let mut available = true;
        let bands = (0..19).map(|i| {
            let mut power = 0.0_f32;
            for ch in [3 + 2 * i, 4 + 2 * i] {
                let key = format!("af-metadata/viz/by-key/lavfi.astats.{ch}.RMS_level");
                match self.mpv.get_property::<String>(&key)
                    .ok().and_then(|s| s.parse::<f32>().ok()) {
                    Some(db) if db.is_finite() => power += 10.0_f32.powf(db / 10.0),
                    Some(db) if db == f32::NEG_INFINITY => {},
                    _ => available = false,
                }
            }
            let db = if power > 0.0 { 10.0 * (power / 2.0).log10() } else { -60.0 };
            ((db + 60.0) / 60.0).clamp(0.0, 1.0)
        }).collect();
        Spectrum { available, bands }
    }
}
```

The string getter is supported by libmpv2 6.0.0; a generic `HashMap` getter is
not provided by that crate. The straightforward sketch uses 38 synchronous
property reads/tick (~950/s), with potentially mixed adjacent frames. Benchmark
that Rust path. If necessary, implement a small owned `MPV_FORMAT_NODE` getter
through its unsafe `GetData` extension, with correct map copying and
`mpv_free_node_contents`, to fetch one coherent snapshot. Do not parse mpv's
human-readable map string or leak borrowed node pointers. The full-map Python
probe demonstrates the underlying C API, not that Rust wrapper extension.

Desktop: retain `media.rs`'s 250 ms `now`/OS-media cadence. Add a 40 ms spectrum
tick and a small `spectrum` event only to subscribed visible windows; no library
or queue scans per tick. About 19 floats/frame is a small payload. Frontend
stores the newest bands; Classic replaces random targets with those values and
uses its existing `requestAnimationFrame` for attack/release interpolation.
Use one subscription/analysis owner across multiple windows; removing one
window must not disable another window's visualizer. Stop work when no consumer
needs it. Hidden/full views that have no spectrum need no fast events.

TUI: read the same `Player::spectrum()` in-process; reduce the current 250 ms
input wait to a deadline-based ~40 ms redraw while visible/playing, leaving slow
metadata/provider work on its existing cadence. Draw ratatui bars from the
latest frame, and use the slower idle wait otherwise. No Tauri dependency.

Before implementation is considered complete: test mono/stereo and format/rate
changes, seek/pause/stop/gapless transitions, mute/volume semantics, disabled and
missing filters, opposite-phase stereo, CPU on a modest machine, and audible
sync on Linux/Windows/macOS packaged mpv builds. These are pending acceptance
checks, not claims established by silent-source probes.

## Local references used

- Installed `mpv(1)`: `af-metadata/<filter-label>`, `vf-metadata`, metadata
  `by-key` subproperties, `lavfi`, `audio-buffer`, null AO options, `audio-pts`.
- `/usr/share/doc/ffmpeg/ffmpeg-filters.html`: `astats`, `ebur128`,
  `aspectralstats`, `afftfilt`, `showfreqs`, `amerge`, `bandpass`;
  corroborated by `ffmpeg -h filter=<name>` and actual probes.
- `/usr/include/mpv/client.h`, `render.h`, `stream_cb.h`; installed libmpv2
  6.0.0 source (`GetData`, `get_property`, `String` implementation).
- `/usr/include/wine/windows/audiosessiontypes.h` confirms the WASAPI loopback
  flag locally. Native Windows/macOS capture APIs and chosen Rust bindings still
  require platform-specific verification; no online documentation was fetched.
