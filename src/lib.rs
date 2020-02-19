pub extern crate influxdb;
pub extern crate r2d2;

mod ext;

use ext::SyncClient;
use influxdb::{Client, Error as InfluxDBError};

use std::convert::From;
use std::error;
use std::error::Error as _StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Other(InfluxDBError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        #[allow(deprecated)] // `cause` is replaced by `Error:source` in 1.33
        match self.cause() {
            Some(cause) => write!(fmt, "{}: {}", self.description(), cause),
            None => write!(fmt, "{}", self.description()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Other(ref err) => match err {
                InfluxDBError::InvalidQueryError { .. } => "query is invalid",
                InfluxDBError::UrlConstructionError { .. } => "failed to build URL",
                InfluxDBError::ProtocolError { .. } => "http protocol error",
                InfluxDBError::DeserializationError { .. } => "http protocol error",
                InfluxDBError::DatabaseError { .. } => "database error",
                InfluxDBError::AuthenticationError => "authentication error",
                InfluxDBError::AuthorizationError => "authorization error",
                InfluxDBError::ConnectionError { .. } => "connection error",
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct InfluxDBConnectionInfo {
    url: String,
    database: String,
    username: Option<String>,
    password: Option<String>,
    is_auth: bool,
}

#[derive(Debug)]
pub struct InfluxDBConnectionManager {
    info: InfluxDBConnectionInfo,
}

impl InfluxDBConnectionManager {
    pub fn new<T: Into<InfluxDBConnectionInfo>>(params: T) -> InfluxDBConnectionManager {
        InfluxDBConnectionManager {
            info: params.into(),
        }
    }
}

pub struct SimpleInfo {
    pub url: String,
    pub database: String,
}

pub struct AuthInfo {
    pub url: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl From<SimpleInfo> for InfluxDBConnectionInfo {
    fn from(item: SimpleInfo) -> Self {
        InfluxDBConnectionInfo {
            url: item.url,
            database: item.database,
            username: None,
            password: None,
            is_auth: false,
        }
    }
}

impl From<AuthInfo> for InfluxDBConnectionInfo {
    fn from(item: AuthInfo) -> Self {
        InfluxDBConnectionInfo {
            url: item.url,
            database: item.database,
            username: Some(item.username),
            password: Some(item.password),
            is_auth: true,
        }
    }
}

impl r2d2::ManageConnection for InfluxDBConnectionManager {
    type Connection = SyncClient;
    type Error = Error;

    fn connect(&self) -> Result<SyncClient, Error> {
        let client = if self.info.is_auth {
            Client::new(self.info.url.to_owned(), self.info.database.to_owned()).with_auth(
                self.info.username.to_owned().unwrap(),
                self.info.password.to_owned().unwrap(),
            )
        } else {
            Client::new(self.info.url.to_owned(), self.info.database.to_owned())
        };
        Ok(SyncClient::new(client))
    }

    fn is_valid(&self, conn: &mut SyncClient) -> Result<(), Error> {
        conn.ping().map_err(Error::Other).map(|_| ())
    }

    fn has_broken(&self, conn: &mut SyncClient) -> bool {
        self.is_valid(conn).is_err()
    }
}
