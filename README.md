# scurry: High performance Port scanning tool
[![Build Status](https://travis-ci.org/Skarlett/scurry.svg?branch=master)](https://travis-ci.org/Skarlett/scurry)
[![asciicast](https://asciinema.org/a/6RsstnYyovWmVCjYLdgYADMG0.svg)](https://asciinema.org/a/6RsstnYyovWmVCjYLdgYADMG0)

## Abstract
Scurry is an **experimental** utility that performs network discovery, and security auditing. Likewise to the project [nmap](https://nmap.org/), Scurry attempts to build out similar functionality, including the adoption of nmap's ecosystem. Nmap has been the industry standard for many years, and has a vast ecosystem of lua libraries. Scurry does not aim to replicate/replace nmap. 

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

## Contribution
Please review the license, follow our contribution guide, and take a look at our roadmap. If you wish to change the road map, propose it in an issue. 


## Implementation choices

### Lanuage
Most of the tooling choosen in this project originates from two primary concerns, performance and scaling ability. The reason I've choosen rust as the primary language for writing this project is because it facilitates these properties. 

The type system in rust, with its borrow checking should allow us to write code that both ensures it will run, and that "special" knowledge isn't needed to modify its source code. The elaborate on the term "special knowledge" - I would like to example the case of a C program, where type dynamic casting is provided. This implicates that you now need to read and understand behavior outside of the feature your implementing.

Reading code outside of what feature you're implementing to examine its behavior - just to predict how to write your own code, *in my opinion* is a waste of time. This is exactly what "special knowledge" is. Rust isn't perfect, but with its rich type system, we can reliably ignore behavior outside of our own code due to its correct/safe nature. 

In theory this should be easier for developers to contribute code to the project.
