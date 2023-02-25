### node-monitor

This is simple program that monitors the state of a local node and reports whether it's synced, catching up or has stopped syncing.

### Requirements:(for building from source)

-    Rust (version 1.49 and above);
-    musl-tools, libssl-dev for linux

### Installation

1.   Building from Source

     -    Clone this repository
     -    Bulid binary new using ``cargo build  --release`
     -    To build a static binary instead use the following commands below:<br>
          **windows:** use `cargo build --release --target x86_64-pc-windows-msvc`. <br>
          **linux:** use ` cargo build --release --target x86_64-unknown-linux-musl`.

2.   Download the binaries [here](https://github.com/Dartroom/node-monitor/releases/)

### Usage

Create a settings.json file (see example below)

```json
        {
            "pollingRate": 10, // polling rate in milliseconds
            "xAlgoToken": "",// xAlgoToken
            "validRoundRange": 1,
            "localNode": , // url of the local node
            "clusterNodes":[],// an Array of remote cluster nodes
            "port": 9030, // port to connect run node-monitor
            "nodePort": 9094
        }

```

CLi interface

```bash

     Usage: node-monitor [OPTIONS]

  Options:
    -c, --config <CONFIG>      path to the  configuration file (default: if not specified, settings.json file in the path as executable is used)
    -d, --data-dir <DATA_DIR>  The path to store the data.json file (default is the same directory as executable)
    -v, --verbose              shown more logging information, default is false with log level=info,
    -h, --help                 Print help
    -V, --version              Print version


```
