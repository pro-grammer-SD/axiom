# Axiom Monolith Intrinsics Reference

24 modules are statically linked — no `load` statement needed.

## Module Index

| # | Namespace | Backend | Key Functions |
|---|-----------|---------|---------------|
| 1 | `alg` | rayon, petgraph | `range` `sum` `map` `filter` `sort` `parallel_map` `len` |
| 2 | `ann` | std | `type` `fields` `methods` `is_nil` |
| 3 | `aut` | chrono, notify | `schedule` `cron` `watch` `sleep` |
| 4 | `clr` | colored | `rgb` `ansi` `bold` `italic` `reset` |
| 5 | `col` | dashmap | `new_map` `concurrent_map` `keys` `values` |
| 6 | `con` | tokio | `spawn` `await_all` `channel` `sleep_async` |
| 7 | `csv` | csv | `parse` `stringify` `read_file` `write_file` |
| 8 | `dfm` | polars | `read_csv` `select` `filter` `groupby` `join` |
| 9 | `env` | dotenvy | `load` `get` `set` `all` |
| 10 | `git` | git2 | `init` `clone_repo` `commit` `push` `status` |
| 11 | `ioo` | std::fs | `read` `write` `append` `mkdir` `ls` `rm` `exists` |
| 12 | `jsn` | serde_json | `parse` `stringify` `pretty` |
| 13 | `log` | indicatif | `progress` `spinner` `info` `warn` `error` |
| 14 | `mth` | f64 | `sqrt` `pow` `sin` `cos` `pi` `e` `abs` `floor` `ceil` |
| 15 | `net` | reqwest | `get` `post` `put` `json` `headers` |
| 16 | `num` | ndarray | `matrix` `zeros` `ones` `dot` `transpose` |
| 17 | `plt` | plotters | `line_chart` `scatter` `bar` `save_png` |
| 18 | `pth` | walkdir | `walk` `exists` `join` `basename` `dirname` |
| 19 | `str` | regex, unicode | `upper` `lower` `trim` `split` `replace` `match` `len` |
| 20 | `sys` | sysinfo | `info` `cpu_usage` `memory` `cwd` `chdir` |
| 21 | `tim` | chrono | `now` `format` `parse` `diff` `timestamp` |
| 22 | `tui` | ratatui+crossterm | `block` `list` `table` `gauge` `sparkline` `dashboard` `fx_*` |
| 23 | `cli` | std::process | `exec` `shell` `env` `args` |
| 24 | `usb` | **rusb** | `list` `open` `transfer` |

---

## Module 22: tui

```axiom
// Widgets (display 1.5s then return)
tui.block("Title", "content text")
tui.list(["a", "b", "c"], "List Title")
tui.table(["Name","Val"], [["x","1"],["y","2"]])
tui.gauge("Progress", 75)          // 0-100
tui.sparkline([1,3,2,5,4], "Data")

// Interactive dashboard — press q to quit
tui.dashboard("My App")

// TachyonFX shader descriptors
let fade  = tui.fx_fade(500)        // {shader:"fade",     duration_ms:500}
let gl    = tui.fx_glitch(300)      // {shader:"glitch",   duration_ms:300, intensity:0.7}
let rgb   = tui.fx_rgb_split(400)   // {shader:"rgb_split",duration_ms:400, offset_px:3}
let bnc   = tui.fx_bounce(600)      // {shader:"bounce",   duration_ms:600, amplitude:2}
```

### Dashboard Panels

| Panel | Widget | Content |
|-------|--------|---------|
| Header (3 rows) | Paragraph | Title + [q to quit] |
| Left 50% | Animated List | 8 modules, rotating highlight |
| Right 50% | Sparkline | Live waveform |
| Bottom (5 rows) | Gauge | VM Load 0-100% animated |

---

## Module 24: usb

```axiom
// Enumerate all USB devices
let devs = usb.list()
// [{vendor_id:1234, product_id:5678, bus:1, address:3, class:0}, ...]

// Open by vendor:product ID
let h = usb.open(0x0483, 0x5740)
// {vendor_id:1155, product_id:22336, open:true}
// AXM_502 if device not found

// Bulk write
let r = usb.transfer(h, 0x01, [0x01, 0x02, 0x03], 1000)
// {ok:true, bytes_written:3}
// AXM_502 on failure
```
