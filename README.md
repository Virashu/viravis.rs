<center>

# viravis.rs

Cross-platform output audio stream visualization (http, websocket, serial) 

</center>

## Installation

```shell
cargo install --path .
```

## Usage

```shell
viravis <args>
```

- `-m, --mode`: analyzer mode
  - `fft`: by frequency
  - `rolling` (default): wave-like visuals
- `-p, --port` (optional): serial port of compatible arduino (see [arduino](#arduino))
- `--graph` (optional): flag to draw a visualization graph in console
- `--sample-rate` (optional): audio sample rate (use for speed control)

## Web-Interfaces

### HTTP

Available at port `7777`. \
`/`: returns json array with floating numbers.

### WebSocket

Available at port `7778`. \
Sends json array with floating numbers.

## Arduino

Arduino with address LED strip can be connected. \
See [arduino repo](https://github.com/Virashu/viravis_arduino)

## Issues

- No auto device switch: if you connect a new device, you need to restart the program.
