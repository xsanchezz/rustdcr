use tokio_tungstenite::{
    tungstenite::{client, Error},
    WebSocketStream,
};

pub struct ConnConfig {
    host: String,

    endpoint: String,

    user: String,

    password: String,

    disable_tls: bool,

    certificates: String,

    proxy: String,

    proxy_username: String,

    proxy_password: String,
}

pub fn dial(config: &ConnConfig) -> Result<Box<WebSocketStream<client::AutoStream>>, Error> {
    if config.disable_tls {}
    todo!()
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn ws_dialler() {
        // let config = ConnConfig {
        //     host: String::from("localhost:9109"),
        //     endpoint: String::from("ws"),
        //     user: String::from("user"),
        //     password: String::from("password"),
        // };

        // dial(&config).unwrap();
    }
}
