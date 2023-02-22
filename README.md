### node-monitor

This is simple program that monitor the state of a local node and reports whether it's synced, catching up or has stopped syncing.

### Requirements

-    Rust (version 1.49 and above)

### Usage

1. Download the binaries here
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
