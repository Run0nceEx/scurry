# Proxbox
---

### Roadmap
[ ] Host Discovery
    [ ] - Nmap db parsers
    [ ] - collection handler

[ ] Sink Hole
    [ ] - Packet Forgery
    [ ] - Transparent TCP proxy/passthrough

[ ] Proxy Chain capable
    [ ] - proxy test
    [ ] - protocol chaining

[ ] Multi-clients
    [ ] - zoo keeper cluster

[ ] script engine
  

#### SinkHole
The idea of the sink hole is that the scanner will spoof the source address of a packet, the source being replaced with the sinkhole. The sink hole will transparently hold the connection, and digest it. Once digested, it will chunk the results and send them back to the scanner. 
This should allow us to bypass basic IDS IP blocking, if the IDS blocks the connection - it would should block the sink hole instead of the server.

##### Host Discovery
Using NMap's data, we're planning on intrigating host discovery. This should allow us to identify services being ran on a server.

#### Proxy Chain capable
The idea of a proxy-chained scanner is that the proxy will be blocked by an IDS instead of the scanner, but in turn we will be limited to the proxy's settings and regulations. 

#### Multi-clients
The idea is that we should be able to strap together many scanners, and have batches of jobs be assigned to clients.

#### Script engine
Eventually one day I'd like a language like lua, and apply it against the engine so that features can be implemented in scripts.