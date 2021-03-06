use failure::Error;
#[cfg(windows)]
use hex::FromHex;
use hex::ToHex;
#[cfg(windows)]
use hyper::client::connect::Destination;
use hyper::Uri as HyperUri;
use url::Url;

use std::borrow::Cow;
use std::ffi::OsStr;

use docker::ClientType;
use docker::API_VERSION;

#[derive(Debug)]
pub struct Uri<'a> {
    encoded: Cow<'a, str>,
}

impl<'a> Into<HyperUri> for Uri<'a> {
    fn into(self) -> HyperUri {
        self.encoded.as_ref().parse().unwrap()
    }
}

impl<'a> Uri<'a> where {
    pub(crate) fn parse<O, K, V>(
        socket: &'a str,
        client_type: &ClientType,
        path: &'a str,
        query: Option<O>,
    ) -> Result<Self, Error>
    where
        O: IntoIterator,
        O::Item: ::std::borrow::Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let host: String = Uri::socket_host(socket, client_type)?;

        let host_str = format!(
            "{}://{}/{}{}",
            Uri::socket_scheme(client_type),
            host,
            API_VERSION,
            path
        );
        let mut url = Url::parse(host_str.as_ref()).unwrap();
        url = url.join(path).unwrap();

        if let Some(pairs) = query {
            url.query_pairs_mut().extend_pairs(pairs);
        }

        debug!(
            "Parsing uri: {}, client_type: {:?}, socket: {}",
            url.as_str(),
            client_type,
            socket
        );
        Ok(Uri {
            encoded: Cow::Owned(url.as_str().to_owned()),
        })
    }

    fn socket_host<P>(socket: P, client_type: &ClientType) -> Result<String, Error>
    where
        P: AsRef<OsStr>,
    {
        match client_type {
            ClientType::Http | ClientType::SSL => {
                Ok(socket.as_ref().to_string_lossy().into_owned())
            }
            #[cfg(unix)]
            ClientType::Unix => {
                let mut host: String = String::new();
                socket
                    .as_ref()
                    .to_string_lossy()
                    .as_bytes()
                    .write_hex(&mut host)?;
                Ok(host)
            }
            #[cfg(windows)]
            ClientType::NamedPipe => {
                let mut host: String = String::new();
                socket
                    .as_ref()
                    .to_string_lossy()
                    .as_bytes()
                    .write_hex(&mut host)?;
                Ok(host)
            }
        }
    }

    fn socket_scheme(client_type: &ClientType) -> &'a str {
        match client_type {
            ClientType::Http => "http",
            ClientType::SSL => "https",
            #[cfg(unix)]
            ClientType::Unix => "unix",
            #[cfg(windows)]
            ClientType::NamedPipe => "net.pipe",
        }
    }

    #[cfg(windows)]
    fn socket_path(uri: &HyperUri) -> Option<String> {
        uri.host()
            .iter()
            .filter_map(|host| {
                Vec::from_hex(host)
                    .ok()
                    .map(|raw| String::from_utf8_lossy(&raw).into_owned())
            })
            .next()
    }

    #[cfg(windows)]
    pub(crate) fn socket_path_dest(dest: &Destination, client_type: &ClientType) -> Option<String> {
        format!("{}://{}", Uri::socket_scheme(client_type), dest.host())
            .parse()
            .ok()
            .and_then(|uri| Self::socket_path(&uri))
    }
}
