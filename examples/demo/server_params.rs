pub struct ServerParams {
    pub protocol: &'static str,
    pub host: &'static str,
    pub port: &'static str,
}

pub const SERVER_PARAMS_DEFAULT: ServerParams = ServerParams {
    protocol: "http://",
    host: "127.0.0.1",
    port: "8000",
};

impl ServerParams {
    pub fn address(&self) -> String {
        format!(
            "{protocol}{host}:{port}",
            protocol = self.protocol,
            host = self.host,
            port = self.port
        )
    }
}
