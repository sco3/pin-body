use log::info;
use pingora::prelude::*;
use pingora::server::configuration::{Opt, ServerConf};
use pingora::server::Server;

use pin_body::inspector::BodyInspector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let opt = Opt::parse_args();
    let mut conf = ServerConf::default();
    conf.upgrade_sock = "/tmp/pingora_upgrade.sock".to_string();
    //let mut my_server = Server::new(Some(opt))?;
    let mut my_server = Server::new_with_opt_and_conf(Some(opt), conf);
    my_server.bootstrap();

    let mut my_proxy = http_proxy_service(&my_server.configuration, BodyInspector);
    my_proxy.add_tcp("0.0.0.0:6152");

    info!("Body Inspector running on 0.0.0.0:6152");
    my_server.add_service(my_proxy);
    my_server.run_forever();
}
