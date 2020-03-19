
enum Protocol {
    HTTP,
    HTTPS,
    Socks5

}


struct Settings {
    collect_protocols: Vec<Protocol>
}


pub struct State {
    settings: Settings    

}


