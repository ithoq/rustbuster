use futures::Stream;
use hyper::{
    rt::{self, Future},
    Client, Method, StatusCode, Uri,
};

use std::sync::mpsc::Sender;
use hyper_tls::HttpsConnector;

#[derive(Debug, Clone)]
pub struct Target {
    url: Uri,
    method: Method,
    status: StatusCode,
    pub error: Option<String>,
}

fn _fetch_url(
    tx: Sender<Target>,
    client: &hyper::Client<_, hyper::Body>,
    url: Uri,
) -> impl Future<Item = (), Error = ()> {
    let tx_err = tx.clone();
    let mut target = Target {
        url: url.clone(),
        method: Method::GET,
        status: StatusCode::default(),
        error: None,
    };
    let mut target_err = target.clone();

    client
        .get(url)
        .and_then(move |res| {
            target.status = res.status();

            tx.send(target).unwrap();

            Ok(())
        })
        .or_else(move |e| {
            target_err.error = Some(e.to_string());
            tx_err.send(target_err).unwrap();
            Ok(())
        })
}

pub fn _run(tx: Sender<Target>, urls: Vec<hyper::Uri>, n_threads: usize, use_https: bool) {
    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = if use_https { Client::builder().build(https) } else { Client::builder().build_http() };

    let stream = futures::stream::iter_ok(urls)
        .map(move |url| _fetch_url(tx.clone(), &client, url))
        .buffer_unordered(n_threads)
        .for_each(Ok)
        .map_err(|err| eprintln!("Err {:?}", err));

    rt::run(stream);
}
