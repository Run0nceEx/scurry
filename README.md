# scurry 0.0.1 [WIP]
[![asciicast](https://asciinema.org/a/6RsstnYyovWmVCjYLdgYADMG0.svg)](https://asciinema.org/a/6RsstnYyovWmVCjYLdgYADMG0)
Scurry a port scanner built from the [tokio runtime](https://tokio.rs). Scurry's priority is to build a fast concurrent service discovery tool, while maintaining the accuracy of nmap. This attempts to bridge the gap between [nmap](https://nmap.org/) and [masscan](https://github.com/robertdavidgraham/masscan).

### Scurry is in active development
Scurry is currently not accept contributions and is not licensed for anyone to use currently.

+ [Introduction](#introduction)
+ [Current capabilities](#introduction)
+ [planned](#introduction)
+ [Usage](#introduction)



## Introduction
Scurry is a blazingly fast programmable port-scanning engine. The internet is getting more complicated everyday, and it needs a port scanner to meet its design choices, and scale. Scurry is currently in early development but has shown promising results. Currently all testing is done on consumer hardware, but roughly returns `2000 connections/sec`

Scurry is being developed as a seperate engine to nmap.
Scurry is **not a drop in replacement** for nmap. Though nmap scripts are planned to be compatiable with scurry, they are not currently.


## Planned
Heres some that I plan on delivering on a later date.

feature | integrated
--- | ---
Nmap Version Detection | N/A
Compatible witth Nmap scripts (luaJIT) | N/A
Apache Zookeeper Adaptor | N/A
px-swarm-daemon | N/A

#### Version detection
When this feature is added, it should be able to determine services running behind ports, including versions and version ranges.

#### Compatiable with nmap scripts
After we implement probing and discovering capabilities, we sometimes want more information about a service, which includes inter-operating with it. Allowing lua scripts to write custom handlers, we also should support nmap scripts.

#### Apache Zookeeper adaptor
Adding zookeeper should let us take and push results into zookeeper, which in turn allows us to provide cluster computations.

##  Compiling
https://rustup.rs/ (Install compiler)
```
git clone https://github.com/Skarlett/scurry
cd scurry
cargo build --release --bin port_scanner
mv target/release/port_scanner $PWD
./port_scanner --help
```

## Usage 

```./px-engine --target 1.1.1.1/24 -p 80 443 --exclude 1.1.1.155 --format stream```
```
Port scanner

USAGE:
    px-engine [OPTIONS] ...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -x, --exclude <exclude>...       Exclude by IP/cidr address
    -f, --format <format>            Specify output format [default: stdout]
    -m, --method <method>            choice of handler used [default: open]
    -p, --ports <ports>...           Ranges of ports you'd like to scan on every IP, Accepts a sequence of numbers "80"
                                     and ranges "8000-10000"
    -t, --target <target>...         Target IP addresses, supports IPv4 and IPv6. Accepts Accepts a sequence of IPs
                                     "10.0.0.1" and CIDR "10.0.0.1/24"
        --threads <threads>          
        --timeout <timeout>          Specify output format [env: SCURRY_TIMEOUT=]  [default: 5]
```
