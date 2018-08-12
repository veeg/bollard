extern crate boondock;
extern crate hyper;
extern crate tokio;

use boondock::Docker;
use hyper::client::HttpConnector;
use hyper::rt::Future;
use tokio::runtime::Runtime;

fn main() {
    let mut rt = Runtime::new().unwrap();

    // --

    let docker = Docker::<HttpConnector>::new().unwrap();
    let f = docker
        .version()
        .map(|version| println!("version: {:#?}", version))
        .map_err(|err| println!("error: {}, backtrace: {}", err, err.backtrace()));

    // --

    rt.block_on(f).unwrap();
    rt.shutdown_now().wait().unwrap();
}