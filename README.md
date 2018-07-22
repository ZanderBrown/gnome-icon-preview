# Icon Preview

A simple utility for testing out icons

Originally created to help debug a problem in Hammond, Icon Preview aims to make it easier to assess how well an icon fits in

## Building

A flatpak manifest is provided so cloning in Builder and hitting play should Just Work™.

Running as a flatpak has the downside that only the named icons provided by the runtime will be available.

You can still build the old fasioned way with meson:
```
meson builddir
ninja -C builddir
ninja -C builddir install
```