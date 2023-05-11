Firehose with Substreams


1.
```bash
git clone https://github.com/InjectiveLabs/injective-core.git

cd injective-core
git checkout f/firehose #this branch needs to be updated
make install
./setup.sh
```

Alter the ~/.injectived/config/config.toml

By adding,

```bash
[extractor]
enabled = true
output_file = “stdout”
```

2.
```bash
git clone https://github.com/InjectiveLabs/firehose-cosmos
cd firehose-cosmos
git checkout substreams #this branch is needed for substreams related flags (it is not updated yet so this is the latest version)
go mod download
make install
which firecosmos
cd devel/injective/
```

Change the common-first-streamable-block to the current block of your injectived instance in devel/injective/tmp/firehose.yml

Alter the start.sh with:

```bash
#!/usr/bin/env bash

fh_bin="firehose-cosmos"

if [[ -z $(which $fh_bin || true) ]]; then
  echo "You must install the firehose-binary first. See README for instructions"
  exit 1
fi

echo "Starting firehose"
pushd tmp
$fh_bin start --substreams-enabled=true
popd
```

And the firehose.yml should look like this:

```bash
start:
  args:
    - ingestor
    - merger
    - firehose
  flags:
    common-first-streamable-block: 1
    common-blockstream-addr: localhost:9000
    ingestor-mode: node
    ingestor-node-path: '/Users/macbookpro/go/bin/injectived'
    ingestor-node-args: start --x-crisis-skip-assert-invariants
    ingestor-node-logs-filter: "module=(p2p|pex|consensus|x/bank)"
    firehose-real-time-tolerance: 99999h
    relayer-max-source-latency: 99999h
    verbose: 1
```

3.

Install the Substreams CLI from: https://substreams.streamingfast.io/getting-started/installing-the-cli

I used 
```bash
brew install streamingfast/tap/substreams
```
Validate installation:
```bash
substreams --version
```

You will need a API key. You can get it from: https://app.streamingfast.io

Then, export your keys:

```bash
export STREAMINGFAST_KEY=server_123123 # Use your own API key
export SUBSTREAMS_API_TOKEN=$(curl https://auth.streamingfast.io/v1/auth/issue -s --data-binary '{"api_key":"'$STREAMINGFAST_KEY'"}' | jq -r .token)
```

Now, we can write an example substream:

We will need parts:

The substreams manifest (substreams.yaml file)

```bash
specVersion: v0.1.0
description: 

package:
  name: Transfer
  version: v0.0.1

protobuf:
  files:
    - gogoproto/gogo.proto
    - types.proto
    - cosmos.proto
  importPaths:
    - ./proto

binaries:
  default:
    type: wasm/rust-v1
    file: ./target/wasm32-unknown-unknown/release/substreams.wasm

modules:
  - name: map_transfer
    kind: map
    startBlock: 1
    inputs:
      - source: sf.cosmos.type.v1.Block
    output: 
      type: proto:sf.cosmos.type.v1.ResponseBeginBlock
```


Rust Manifest file (Cargo.toml)

```bash
[package]
name = "substreams"
version = "0.1.0"
description = ""
edition = "2021"
repository = "https://github.com/streamingfast/substreams-template"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2.79"
prost = { version = "0.11" }
prost-types = "0.11"
substreams = "0.5.6" 

[build-dependencies]
prost-build = "0.11"

[profile.release]
lto = true
opt-level = 's'
strip = "debuginfo"
```

Note that we use prost crate for protobuf encoding and decoding, and substreams crate’s version should be 0.5.6 (for map handlers and other things doesn’t work in lower versions)

Protobufs are under the proto folder.

And to generate Rust codes from protobufs:

```bash
substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"
```

The generated Rust codes will be under src/pb folder. And also, the protobufs generate model must be referenced by a Rust module, to do so, create a file named mod.rs within the src/pb directory with the following content:

Now, the substreams module handler will be under src folder:

```bash
mod pb;

#[substreams::handlers::map]
fn map_transfer(blk: pb::cosmos::Block) -> Result<pb::cosmos::ResponseBeginBlock, substreams::errors::Error> {
    let events: Vec<pb::cosmos::Event> = blk.result_begin_block
        .unwrap()
        .events
        .into_iter()
        .filter(|event| event.event_type == "transfer")
        .collect();
    Ok(pb::cosmos::ResponseBeginBlock {events})
}
```

We use the source as sf.cosmos.type.v1.Block which we referenced as cosmos::Block with mod.rs, and the output is the sf.cosmos.ResponseBeginBlock as we specified in the substreams manifest.

Now, compile the substreams module with:

```bash
cargo build --release --target wasm32-unknown-unknown
```

And, one can run the substreams module (firehose should also be running) with the command:

```bash
substreams run -p -e 127.0.0.1:9030 substreams.yaml map_transfer
```



