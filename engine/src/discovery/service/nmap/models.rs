
// https://nmap.org/book/vscan-fileformat.html

#[derive(Debug, Clone)]
struct CPE {
    product_name: String,
    version: String,
    info: String,
    hostname: Option<String>,
    operating_system: String,
    device_type: String,
}