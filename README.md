# COSMIC Popup Grab Reproducer

## What is this?

A Hello-World-sized **dummy COSMIC applet** for diagnosing a potential bug in
`cosmic-panel`.

## What does it do?

- It contains one button labelled **POP**.
- A secondary click opens a standard grabbed libcosmic popup containing one
  line of text.
- The button has a standard non-grabbing libcosmic
  [tooltip](src/main.rs#L92-L101).
- Apart from the state needed for those two surfaces, there are no workspace
  protocols, window tracking, icons, configuration watchers, logging
  frameworks, or background tasks.

## Why? What is the purpose?

A bug was identified while using the
[Workspace Icons](https://github.com/crocodile/cosmic-ext-applet-workspace-icons/issues/19)
applet:

> Workspace Icons can trigger a COSMIC panel crash when its right-click popup is
opened across two monitors in an A → B → A sequence. The `cosmic-panel` process stays alive but
stops displaying them, so COSMIC does not restart it automatically.

This applet proves that the Workspace Icons applet's code might not be responsible. This skeleton code-like applet reproduces the same failure using only standard libcosmic tooltip and popup APIs.

### Steps to reproduce the bug with this applet:

1. Add the applet to panels displayed on at least two monitors.
2. Secondary-click (two-finger click) **POP** on monitor A.
3. While its popup is open, secondary-click **POP** on monitor B.
4. Return to monitor A and secondary-click **POP** again.

#### Actual:

- The panel and dock disappear and do not recover on their own.
- The journal reports `NotTheTopmostPopup` and floods with Wayland protocol
  errors. See **Diagnosis** below.
- The `cosmic-panel` process does not exit, so the session manager has nothing
  to restart.

#### Expected:

- Opening the popup on another monitor should close or replace the previous
  popup.
- The panel and dock should remain visible and usable.
- There is no bug, everybody is happy

## Test results

### 1st test run:

#### Environment:

Reproduced on 2026-07-22 with this setup:

| Component | Version / details |
| --- | --- |
| Displays | Four monitors; captured failure crossed `DP-1` and `HDMI-A-1` |
| OS | Fedora Linux 44 (Workstation Edition) |
| Kernel | `7.1.3-201.fc44.x86_64` |
| COSMIC | `cosmic-panel`, `cosmic-session`, and `cosmic-comp` `1.3.0-1.fc44.x86_64` |
| libcosmic | `1.0.0` at [`d9b2b38`](https://github.com/pop-os/libcosmic/commit/d9b2b38bf4dc8f2af3d7dc7be9fa296ab75c3170), pinned to match the failing [Workspace Icons](https://github.com/crocodile/cosmic-ext-applet-workspace-icons) build |
| Panel config | `cosmic-panel-config` `0.1.0` at [`82b76c9`](https://github.com/pop-os/cosmic-panel/commit/82b76c9b01d2697bc10d62b6c511c26cc6cc8e8f) |
| COSMIC protocols | `cosmic-protocols` and `cosmic-client-toolkit` `0.2.0` at [`32283d7`](https://github.com/pop-os/cosmic-protocols/commit/32283d76a8d0342da74c4cc022a533c52dcf378f) |
| UI / Wayland | iced `0.14.0`, `wayland-client` `0.31.14`, `wayland-protocols` `0.32.13` |
| Toolchain | Rust and Cargo `1.96.1` from Fedora 44 |

#### Results:

The same-monitor tests used two desktop entries that launch the same binary.

| Monitor scope | Tooltip | Result |
| --- | --- | --- |
| Same monitor 1 | Enabled | ✅ 100+ popup opens; no crash |
| Same monitor 2 | Enabled | ✅ 100+ popup opens; no crash |
| Cross-monitor | Disabled | ✅ About 70 popup opens across two runs; no crash |
| Cross-monitor | Enabled | ❌ Crashed on the first A → B → A sequence |

---
### 2nd test run with updated dependencies:

#### Environment:

Reproduced on 2026-07-22 with this setup:

| Component | Version / details |
| --- | --- |
| Displays | Four monitors; captured failure crossed `DP-1` and `HDMI-A-1` |
| OS | Fedora Linux 44 (Workstation Edition) |
| Kernel | `7.1.3-201.fc44.x86_64` |
| COSMIC | `cosmic-panel`, `cosmic-session`, and `cosmic-comp` `1.3.0-1.fc44.x86_64` |
| ■ libcosmic | `1.0.0` at [`ef162b8`](https://github.com/pop-os/libcosmic/commit/ef162b8e16ba4493e05c169cd56c7b9f77f0fda5) |
| ■ Panel config | `cosmic-panel-config` `0.1.0` at [`f416dbb`](https://github.com/pop-os/cosmic-panel/commit/f416dbbe72d8600d395d24bf9f96f6ba1299d13b) |
| COSMIC protocols | `cosmic-protocols` and `cosmic-client-toolkit` `0.2.0` at [`32283d7`](https://github.com/pop-os/cosmic-protocols/commit/32283d76a8d0342da74c4cc022a533c52dcf378f) |
| UI / Wayland | iced `0.14.0`, `wayland-client` `0.31.14`, `wayland-protocols` `0.32.13` |
| Toolchain | Rust and Cargo `1.96.1` from Fedora 44 |

■ - changed from the original test. (`cargo update` refreshed 88 lockfile
packages in total. These changes **did not** change the result.)

#### Results:

| Monitor scope | Tooltip | Result |
| --- | --- | --- |
| Cross-monitor | Enabled | ❌ Crashed on the first A → B → A sequence |

The updated build produced the same `NotTheTopmostPopup` error, followed by
35,116 repeated protocol errors in seven seconds.

## Diagnosis

The crash required cross-monitor popup switching with the
[tooltip](src/main.rs#L92-L101) enabled. The tooltip is non-grabbing; it exposes
a popup ordering or lifetime bug rather than requesting the invalid grab.

Updating all compatible Cargo dependencies did not change the result. An
outdated lockfile is not the cause, although libcosmic remains a possible
contributor.

Final confirmation log (irrelevant messages omitted):

```text
19:00:40.223 cosmic-panel: opening popup Id(58) [monitor A]
19:00:44.359 cosmic-panel: opening popup Id(61) [monitor B]
19:00:46.587 cosmic-panel: opening popup Id(59) [monitor A]
19:00:46.619 cosmic-comp: Failed to grab popup: NotTheTopmostPopup
19:00:46.619 cosmic-session: xdg_popup was not created on the topmost popup
19:00:46.619 cosmic-session: Protocol error 2 on object wl_surface@997
```

Monitor labels are annotations. The last error repeated 24,998 times in four
seconds. Earlier runs recorded 54,997 repetitions in 17 seconds and 59,998 in
50 seconds.

Recovery issue: `cosmic-panel` remained alive, so the session manager did not
restart it.

### Likely cause

`cosmic-comp` rejects the grab because another popup is still topmost. The
locked source points to `cosmic-panel`:

- libcosmic sets the [tooltip to `grab: false`](https://github.com/pop-os/libcosmic/blob/d9b2b38bf4dc8f2af3d7dc7be9fa296ab75c3170/src/applet/mod.rs#L293-L324)
  and the [right-click popup to `grab: true`](https://github.com/pop-os/libcosmic/blob/d9b2b38bf4dc8f2af3d7dc7be9fa296ab75c3170/src/applet/mod.rs#L406-L456).
- `cosmic-panel` [routes each popup to one output's panel space](https://github.com/pop-os/cosmic-panel/blob/82b76c9b01d2697bc10d62b6c511c26cc6cc8e8f/cosmic-panel-bin/src/space_container/wrapper_space.rs#L322-L360).
- That space [closes only its own popup list, then requests the new grab](https://github.com/pop-os/cosmic-panel/blob/82b76c9b01d2697bc10d62b6c511c26cc6cc8e8f/cosmic-panel-bin/src/space/wrapper_space.rs#L168-L280).

Likely sequence: A opens a popup, B opens another, then A closes only its local
popup state and requests a grab while B is still topmost. `cosmic-comp`
correctly rejects it.

## Build

```bash
cargo build --release
```

The standard build includes the tooltip. To build the control variant without
it:

```bash
cargo build --release --no-default-features
```

## Local installation

Installation is intentionally manual so diagnostic deployment remains an
explicit step:

```bash
install -Dm0755 target/release/cosmic-popup-grab-reproducer \
  ~/.local/bin/cosmic-popup-grab-reproducer
install -Dm0644 data/io.github.crocodile.CosmicPopupGrabReproducer.desktop \
  ~/.local/share/applications/io.github.crocodile.CosmicPopupGrabReproducer.desktop
```
---
⚠️ **Warning:** Do not install and run this reproducer applet outside a controlled test
unless you are really bored. Its sole purpose is to replicate the bug described above known to make the panel and dock disappear.
