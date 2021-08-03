/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Profile File Based Providers
//!
//! Profile file based providers combine two pieces:
//!
//! 1. Parsing and resolution of the assume role chain
//! 2. A user-modifiable hashmap of provider name to provider.
//!
//! Profile file based providers first determine the chain of providers that will be used to load
//! credentials. After determining and validating this chain, a `Vec` of providers will be created.
//!
//! Each subsequent provider will provide boostrap providers to the next provider in order to load
//! the final credentials.
//!
//! This module contains two sub modules:
//! - `repr` which contains an abstract representation of a provider chain and the logic to
//! build it from `~/.aws/credentials` and `~/.aws/config`.
//! - `exec` which contains a chain representation of providers to implement passing bootstrapped credentials
//! through a series of providers.
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use aws_auth::provider::env::EnvironmentVariableCredentialsProvider;
use aws_auth::provider::{AsyncProvideCredentials, BoxFuture, CredentialsError, CredentialsResult};
use aws_hyper::DynConnector;
use aws_sdk_sts::Region;
use aws_types::os_shim_internal::{Env, Fs};
use aws_types::profile::ProfileParseError;

use crate::profile::exec::named::NamedProviderFactory;
use crate::profile::exec::{ClientConfiguration, ProviderChain};
use tracing::Instrument;

mod exec;
mod repr;

impl AsyncProvideCredentials for ProfileFileCredentialProvider {
    fn provide_credentials<'a>(&'a self) -> BoxFuture<'a, CredentialsResult>
    where
        Self: 'a,
    {
        Box::pin(self.load_credentials().instrument(tracing::info_span!(
            "load_credentials",
            provider = "ProfileFile"
        )))
    }
}

impl ProfileFileCredentialProvider {
    async fn load_credentials(&self) -> CredentialsResult {
        let inner = self.inner.as_ref().map_err(|err| {
            CredentialsError::Unhandled(format!("failed to load: {}", &err).into())
        })?;
        let mut creds = match inner
            .base()
            .provide_credentials()
            .instrument(tracing::info_span!("load_base_credentials"))
            .await
        {
            Ok(creds) => {
                tracing::info!(creds = ?creds, "loaded base credentials");
                creds
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to load base credentials");
                return Err(e);
            }
        };
        for provider in inner.chain().iter() {
            let next_creds = provider
                .credentials(creds, &self.client_config)
                .instrument(tracing::info_span!("load_assume_role", provider = ?provider))
                .await;
            match next_creds {
                Ok(next_creds) => creds = next_creds,
                Err(e) => {
                    tracing::warn!(provider = ?provider, "failed to load assume role credentials");
                    return Err(e);
                }
            }
        }
        Ok(creds)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ProfileFileError {
    CouldNotParseProfile(ProfileParseError),
    CredentialLoop {
        profiles: Vec<String>,
        next: String,
    },
    MissingCredentialSource {
        profile: String,
        message: Cow<'static, str>,
    },
    InvalidCredentialSource {
        profile: String,
        message: Cow<'static, str>,
    },
    MissingProfile {
        profile: String,
        message: Cow<'static, str>,
    },
    UnknownProvider {
        name: String,
    },
}

impl Display for ProfileFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileFileError::CouldNotParseProfile(err) => {
                write!(f, "could not parse profile file: {}", err)
            }
            ProfileFileError::CredentialLoop { profiles, next } => write!(
                f,
                "profile formed an infinite loop. first we loaded {:?}, \
            then attempted to reload {}",
                profiles, next
            ),
            ProfileFileError::MissingCredentialSource { profile, message } => {
                write!(f, "missing credential source in `{}`: {}", profile, message)
            }
            ProfileFileError::InvalidCredentialSource { profile, message } => {
                write!(f, "invalid credential source in `{}`: {}", profile, message)
            }
            ProfileFileError::MissingProfile { profile, message } => {
                write!(f, "profile `{}` was not defined: {}", profile, message)
            }
            ProfileFileError::UnknownProvider { name } => write!(
                f,
                "profile referenced `{}` provider but that provider is not supported",
                name
            ),
        }
    }
}

impl Error for ProfileFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProfileFileError::CouldNotParseProfile(err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Builder {
    fs: Option<Fs>,
    env: Option<Env>,
    region: Option<Region>,
    connector: Option<DynConnector>,
    custom_providers: HashMap<String, Arc<dyn AsyncProvideCredentials>>,
}

impl Builder {
    pub fn fs(mut self, fs: Fs) -> Self {
        self.fs = Some(fs);
        self
    }

    pub fn env(mut self, env: Env) -> Self {
        self.env = Some(env);
        self
    }

    pub fn connector(mut self, connector: DynConnector) -> Self {
        self.connector = Some(connector);
        self
    }

    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    pub fn with_custom_provider(
        mut self,
        name: impl Into<String>,
        provider: impl Into<Arc<dyn AsyncProvideCredentials>>,
    ) -> Self {
        self.custom_providers.insert(name.into(), provider.into());
        self
    }

    pub fn build(self) -> ProfileFileCredentialProvider {
        let build_span = tracing::info_span!("build_profile_provider");
        let _enter = build_span.enter();
        let fs = self.fs.unwrap_or_else(Fs::real);
        let env = self.env.unwrap_or_else(Env::real);
        let mut named_providers = self.custom_providers;
        named_providers
            .entry("Environment".to_string())
            .or_insert_with(|| {
                Arc::new(EnvironmentVariableCredentialsProvider::new_with_env(
                    env.clone(),
                ))
            });
        // TODO: ECS, IMDS, and other named providers
        let factory = exec::named::NamedProviderFactory::new(named_providers);
        let chain = build_provider_chain(&fs, &env, &factory);
        let connector = self.connector;
        let core_client = aws_hyper::Builder::<()>::new()
            .map_connector(|_| connector.expect("a connector must be provided"))
            .build();
        ProfileFileCredentialProvider {
            inner: chain,
            client_config: ClientConfiguration {
                core_client,
                region: self.region,
            },
        }
    }
}

pub struct ProfileFileCredentialProvider {
    inner: Result<ProviderChain, ProfileFileError>,
    client_config: ClientConfiguration,
}

fn build_provider_chain(
    fs: &Fs,
    env: &Env,
    factory: &NamedProviderFactory,
) -> Result<ProviderChain, ProfileFileError> {
    let profile_set = aws_types::profile::load(&fs, &env).map_err(|err| {
        tracing::warn!(err = %err, "failed to parse profile");
        ProfileFileError::CouldNotParseProfile(err)
    })?;
    let repr = repr::resolve_chain(&profile_set)?;
    tracing::info!(chain = ?repr, "constructed abstract provider from config file");
    exec::ProviderChain::from_repr(repr, &factory)
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;
    use std::future::Future;
    use std::time::{Duration, UNIX_EPOCH};

    use aws_auth::provider::AsyncProvideCredentials;
    use aws_hyper::DynConnector;
    use aws_sdk_sts::Region;
    use aws_types::os_shim_internal::{Env, Fs};
    use smithy_client::dvr::{NetworkTraffic, RecordingConnection, ReplayingConnection};
    use tracing_test::traced_test;

    use crate::profile::{Builder, ProfileFileCredentialProvider};

    #[allow(dead_code)]
    async fn record_test<F, T>(
        test_name: &str,
        f: impl Fn(ProfileFileCredentialProvider) -> F,
    ) -> (RecordingConnection<impl Debug>, T)
    where
        F: Future<Output = T>,
    {
        let fs = Fs::from_test_dir(
            format!("test-data/{}/aws-config", test_name),
            "/Users/me/.aws",
        );
        let env = Env::from_slice(&[("HOME", "/Users/me")]);
        let http_traffic_path = format!("test-data/{}/http-traffic.json", test_name);
        let conn = RecordingConnection::https();
        let provider = Builder::default()
            .env(env)
            .fs(fs)
            .region(Region::from_static("us-east-1"))
            .connector(DynConnector::new(conn.clone()));
        let provider = provider.build();
        let result = f(provider).await;
        let traffic = serde_json::to_string(&conn.network_traffic()).unwrap();
        std::fs::write(http_traffic_path, traffic).unwrap();
        (conn, result)
    }

    async fn execute_test<F, T>(
        test_name: &str,
        f: impl Fn(ProfileFileCredentialProvider) -> F,
    ) -> (ReplayingConnection, T)
    where
        F: Future<Output = T>,
    {
        let fs = Fs::from_test_dir(
            format!("test-data/{}/aws-config", test_name),
            "/Users/me/.aws",
        );
        let env = Env::from_slice(&[("HOME", "/Users/me")]);
        let events =
            std::fs::read_to_string(format!("test-data/{}/http-traffic.json", test_name)).unwrap();
        let traffic: NetworkTraffic = serde_json::from_str(&events).unwrap();
        let conn = ReplayingConnection::new(traffic.events().clone());
        let provider = Builder::default()
            .env(env)
            .fs(fs)
            .region(Region::from_static("us-east-1"))
            .connector(DynConnector::new(conn.clone()));
        let provider = provider.build();
        (conn, f(provider).await)
    }

    #[tokio::test]
    async fn success_test() {
        let (conn, creds) = execute_test("e2e-assume-role", |provider| async move {
            provider.provide_credentials().await
        })
        .await;
        let creds = creds.expect("credentials should be valid");
        assert_eq!(creds.access_key_id(), "ASIARTESTID");
        assert_eq!(creds.secret_access_key(), "TESTSECRETKEY");
        assert_eq!(creds.session_token(), Some("TESTSESSIONTOKEN"));
        assert_eq!(
            creds.expiry(),
            Some(UNIX_EPOCH + Duration::from_secs(1628193482))
        );
        let reqs = conn.take_requests();
        assert_eq!(reqs.len(), 1);
        let req = reqs.values().next().unwrap();
        // TODO: perform more request validation
        assert_eq!(
            req.uri().to_string(),
            "https://sts.us-east-1.amazonaws.com/"
        );
        println!("{:?}", req);
    }

    #[tokio::test]
    async fn region_override() {
        let (conn, creds) = execute_test("region-override", |mut provider| async move {
            // manually override the region, normally this will be set by the builder during
            // provider construction
            provider.client_config.region = Some(Region::new("us-east-2"));
            provider.provide_credentials().await
        })
        .await;
        let creds = creds.expect("credentials should be valid");
        assert_eq!(creds.access_key_id(), "ASIARTESTID");
        assert_eq!(creds.secret_access_key(), "TESTSECRETKEY");
        assert_eq!(creds.session_token(), Some("TESTSESSIONTOKEN"));
        assert_eq!(
            creds.expiry(),
            Some(UNIX_EPOCH + Duration::from_secs(1628193482))
        );
        let reqs = conn.take_requests();
        assert_eq!(reqs.len(), 1);
        let req = reqs.values().next().unwrap();
        // TODO: perform more request validation
        assert_eq!(
            req.uri().to_string(),
            "https://sts.us-east-2.amazonaws.com/"
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn invalid_profile() {
        let (conn, _) = execute_test("invalid-config", |provider| async move {
            let error = provider
                .provide_credentials()
                .await
                .expect_err("config was invalid");
            assert!(
                format!("{}", error).contains("could not parse profile file"),
                "{} should contain correct error",
                error
            )
        })
        .await;
        assert!(
            conn.take_requests().is_empty(),
            "no network traffic should occur"
        );
    }

    #[traced_test]
    #[tokio::test]
    async fn no_profile() {
        let (conn, _) = execute_test("empty-config", |provider| async move {
            let error = provider
                .provide_credentials()
                .await
                .expect_err("config was invalid");
            assert!(
                format!("{}", error).contains("profile `default` was not defined"),
                "{} should contain correct error",
                error
            )
        })
        .await;
        assert!(
            conn.take_requests().is_empty(),
            "no network traffic should occur"
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn retry_on_error() {
        let (conn, creds) = execute_test("retry-on-error", |provider| async move {
            provider
                .provide_credentials()
                .await
                .expect("eventual success")
        })
        .await;
        assert_eq!(creds.access_key_id(), "ASIARTESTID");
        assert_eq!(creds.secret_access_key(), "TESTSECRETKEY");
        assert_eq!(creds.session_token(), Some("TESTSESSIONTOKEN"));
        assert_eq!(
            creds.expiry(),
            Some(UNIX_EPOCH + Duration::from_secs(1628193482))
        );
        assert_eq!(conn.take_requests().len(), 2);
    }
}
