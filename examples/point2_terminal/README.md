# point2-terminal

Minimal FMU that exposes a terminal named "position" with output variables `x` and `y`.
The terminal uses `terminal_kind = "Point2"` so `fmi-sim` can map it to a Foxglove
Point2 JSON channel when writing MCAP output.

Bundle the FMU:

    cargo fmi bundle -p point2-terminal

Run `fmi-sim` with MCAP output:

    POINT2_FMU=target/fmu/point2-terminal.fmu \
    cargo run -p fmi-sim --features mcap --example point2_terminal

Inspect `/tmp/point2.mcap` in Foxglove. You should see a channel named
`fmi-sim/terminal/position` with JSON payloads `{ "x": ..., "y": ... }`.
