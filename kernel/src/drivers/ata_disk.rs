struct HbaPort;

struct AtaDisk {
    id: u64,
    port: &'static mut HbaPort,
    size: Option<u64>,
}
