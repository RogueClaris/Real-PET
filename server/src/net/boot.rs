// a boot to kick people with
pub struct Boot {
    pub socket_address: std::net::SocketAddr,
    pub reason: String,
    pub notify_client: bool,
    pub warp_out: bool,
}
