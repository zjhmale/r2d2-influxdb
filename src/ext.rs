use influxdb::{Client, Error, Query, QueryTypes};
use reqwest::blocking as rb;
use reqwest::blocking::Client as ReqwestClient;
use reqwest::blocking::Response;
use reqwest::{StatusCode, Url};

pub struct SyncClient {
    client: Client,
}

impl SyncClient {
    pub fn new(client: Client) -> Self {
        SyncClient { client }
    }

    pub fn ping(&self) -> Result<(String, String), Error> {
        let res =
            rb::get(format!("{}/ping", self.client.database_url()).as_str()).map_err(|err| {
                Error::ProtocolError {
                    error: format!("{}", err),
                }
            })?;

        let build = res
            .headers()
            .get("X-Influxdb-Build")
            .unwrap()
            .to_str()
            .unwrap();
        let version = res
            .headers()
            .get("X-Influxdb-Version")
            .unwrap()
            .to_str()
            .unwrap();

        Ok((build.to_owned(), version.to_owned()))
    }

    pub fn query<'q, Q>(&self, q: &'q Q) -> Result<String, Error>
    where
        Q: Query,
        &'q Q: Into<QueryTypes<'q>>,
    {
        let query = q.build().map_err(|err| Error::InvalidQueryError {
            error: format!("{}", err),
        })?;

        let basic_parameters: Vec<(String, String)> = self.client.to_owned().into();

        let client = match q.into() {
            QueryTypes::Read(_) => {
                let read_query = query.get();
                let mut url = Url::parse_with_params(
                    format!("{url}/query", url = self.client.database_url()).as_str(),
                    basic_parameters,
                )
                .map_err(|err| Error::UrlConstructionError {
                    error: format!("{}", err),
                })?;

                url.query_pairs_mut().append_pair("q", &read_query);

                if read_query.contains("SELECT") || read_query.contains("SHOW") {
                    ReqwestClient::new().get(url)
                } else {
                    ReqwestClient::new().post(url)
                }
            }
            QueryTypes::Write(write_query) => {
                let mut url = Url::parse_with_params(
                    format!("{url}/write", url = self.client.database_url()).as_str(),
                    basic_parameters,
                )
                .map_err(|err| Error::InvalidQueryError {
                    error: format!("{}", err),
                })?;

                url.query_pairs_mut()
                    .append_pair("precision", &write_query.get_precision());

                ReqwestClient::new().post(url).body(query.get())
            }
        };

        let res: Response = client
            .send()
            .map_err(|err| Error::ConnectionError { error: err })?;

        match res.status() {
            StatusCode::UNAUTHORIZED => return Err(Error::AuthorizationError),
            StatusCode::FORBIDDEN => return Err(Error::AuthenticationError),
            _ => {}
        }

        let s = res.text().map_err(|_| Error::DeserializationError {
            error: "response could not be converted to UTF-8".to_string(),
        })?;

        // todo: improve error parsing without serde
        if s.contains("\"error\"") {
            return Err(Error::DatabaseError {
                error: format!("influxdb error: \"{}\"", s),
            });
        }

        Ok(s)
    }
}
