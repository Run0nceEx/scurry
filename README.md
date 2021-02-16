# scurry: High performance Port scanning tool
[![Build Status](https://travis-ci.org/Skarlett/scurry.svg?branch=master)](https://travis-ci.org/Skarlett/scurry)
[![asciicast](https://asciinema.org/a/6RsstnYyovWmVCjYLdgYADMG0.svg)](https://asciinema.org/a/6RsstnYyovWmVCjYLdgYADMG0)
Scurry attempts to identify services running behind ports, much in the same fashion nmap does. 

Built ontop of the [tokio runtime](https://tokio.rs), Scurry's priority is to build a fast concurrent service discovery tool.
While still maintaining the accuracy of nmap. 

### Scurry is in active developement
Scurry is being developed as a seperate engine to nmap.
Scurry is **not a drop in replacement** for nmap. Though nmap scripts are planned to be compatiable with scurry, they are not currently.

## Planned
Heres some that I plan on delivering on a later date.

feature | integrated
--- | ---
Nmap Version Detection | N/A
Compatible witth Nmap scripts (luaJIT) | N/A

##  Compiling
https://rustup.rs/ (Install compiler) and select **nightly**
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
