
# VST3 Skeleton (dB UI, vertical canvas meters + tooltips)
- Logarithmic slider curve (UI 0..1 → exponential amplitude → dB)
- Peak & RMS meters (per-channel, dBFS) with gradients, clip indicator, peak-hold
- Hover tooltips on Peak & RMS labels (channel, type, dB, linear amplitude)
- Controller strings in dB; default normalized ≈ 0.833 (0 dB)
- Processor: normalized → dB → linear amplitude
- Child webviews: WebView2 (Windows) / WKWebView (macOS via wry)
- Bundle URL helper for Resources
