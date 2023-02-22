### node-monitor

This is simple program that monitors the state of a local node and reports whether it's synced, catching up or has stopped syncing.

### Requirements
1. Building from Source
-  Rust (version 1.49 and above);
-  musl-tools, libssl-dev for linux

### Usage

1. Download the binaries [here](https://github.com/Dartroom/node-monitor/releases/)
2. create a settings.json file (see example below)

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
