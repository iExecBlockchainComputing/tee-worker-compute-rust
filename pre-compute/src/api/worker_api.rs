use crate::compute::{
    errors::ReplicateStatusCause,
    utils::env_utils::{TeeSessionEnvironmentVariable, get_env_var_or_error},
};
use log::error;
use reqwest::{blocking::Client, header::AUTHORIZATION};

/// Thin wrapper around a [`Client`] that knows how to reach the iExec worker API.
///
/// This client can be created directly with a base URL using [`new()`], or
/// configured from environment variables using [`from_env()`].
///
/// # Example
///
/// ```rust
/// use tee_worker_pre_compute::api::worker_api::WorkerApiClient;
///
/// let client = WorkerApiClient::new("http://worker:13100");
/// ```
pub struct WorkerApiClient {
    base_url: String,
    client: Client,
}

const DEFAULT_WORKER_HOST: &str = "worker:13100";

impl WorkerApiClient {
    pub fn new(base_url: &str) -> Self {
        WorkerApiClient {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    /// Creates a new WorkerApiClient instance with configuration from environment variables.
    ///
    /// This method retrieves the worker host from the [`WORKER_HOST_ENV_VAR`] environment variable.
    /// If the variable is not set or empty, it defaults to `"worker:13100"`.
    ///
    /// # Returns
    ///
    /// * `WorkerApiClient` - A new client configured with the appropriate base URL
    ///
    /// # Example
    ///
    /// ```rust
    /// use tee_worker_pre_compute::api::worker_api::WorkerApiClient;
    ///
    /// let client = WorkerApiClient::from_env();
    /// ```
    pub fn from_env() -> Self {
        let worker_host = get_env_var_or_error(
            TeeSessionEnvironmentVariable::WorkerHostEnvVar,
            ReplicateStatusCause::PreComputeWorkerAddressMissing,
        )
        .unwrap_or_else(|_| DEFAULT_WORKER_HOST.to_string());

        let base_url = format!("http://{worker_host}");
        Self::new(&base_url)
    }

    /// Sends exit causes for a pre-compute operation to the Worker API.
    ///
    /// This method reports the exit causes of a pre-compute operation to the Worker API,
    /// which can be used for tracking and debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `authorization` - The authorization token to use for the API request
    /// * `chain_task_id` - The chain task ID for which to report the exit causes
    /// * `exit_causes` - The list of exit causes to report
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the exit causes were successfully reported
    /// * `Err(Error)` - If the exit causes could not be reported due to an HTTP error
    ///
    /// # Errors
    ///
    /// This function will return an [`Error`] if the request could not be sent or
    /// the server responded with a non‑success status.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tee_worker_pre_compute::api::worker_api::WorkerApiClient;
    /// use tee_worker_pre_compute::compute::errors::ReplicateStatusCause;
    ///
    /// let client = WorkerApiClient::new("http://worker:13100");
    /// let exit_causes = vec![ReplicateStatusCause::PreComputeInvalidTeeSignature];
    ///
    /// match client.send_exit_causes_for_pre_compute_stage(
    ///     "authorization_token",
    ///     "0x123456789abcdef",
    ///     &exit_causes,
    /// ) {
    ///     Ok(()) => println!("Exit causes reported successfully"),
    ///     Err(error) => eprintln!("Failed to report exit causes: {error}"),
    /// }
    /// ```
    pub fn send_exit_causes_for_pre_compute_stage(
        &self,
        authorization: &str,
        chain_task_id: &str,
        exit_causes: &Vec<ReplicateStatusCause>,
    ) -> Result<(), ReplicateStatusCause> {
        let url = format!("{}/compute/pre/{chain_task_id}/exit", self.base_url);
        match self
            .client
            .post(&url)
            .header(AUTHORIZATION, authorization)
            .json(exit_causes)
            .send()
        {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    Ok(())
                } else {
                    let body = resp.text().unwrap_or_default();
                    error!("Failed to send exit causes: [status:{status}, body:{body}]");
                    Err(ReplicateStatusCause::PreComputeFailedUnknownIssue)
                }
            }
            Err(err) => {
                error!("HTTP request failed when sending exit causes to {url}: {err:?}");
                Err(ReplicateStatusCause::PreComputeFailedUnknownIssue)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::utils::env_utils::TeeSessionEnvironmentVariable::WorkerHostEnvVar;
    use serde_json::{json, to_string};
    use temp_env::with_vars;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, header, method, path},
    };

    // region Serialization tests
    #[test]
    fn should_serialize_replicate_status_cause() {
        let causes = vec![
            (
                ReplicateStatusCause::PreComputeInvalidTeeSignature,
                r#"{"cause":"PRE_COMPUTE_INVALID_TEE_SIGNATURE","message":"Invalid TEE signature"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeWorkerAddressMissing,
                r#"{"cause":"PRE_COMPUTE_WORKER_ADDRESS_MISSING","message":"Worker address related environment variable is missing"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeDatasetUrlMissing(2),
                r#"{"cause":"PRE_COMPUTE_DATASET_URL_MISSING","message":"Dataset URL related environment variable is missing for dataset 2"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeInvalidDatasetChecksum(1),
                r#"{"cause":"PRE_COMPUTE_INVALID_DATASET_CHECKSUM","message":"Invalid dataset checksum for dataset 1"}"#,
            ),
        ];

        for (cause, expected_json) in causes {
            let serialized = to_string(&cause).expect("Failed to serialize");
            assert_eq!(serialized, expected_json);
        }
    }

    #[test]
    fn should_serialize_vec_of_causes() {
        let causes = vec![
            ReplicateStatusCause::PreComputeDatasetUrlMissing(0),
            ReplicateStatusCause::PreComputeInvalidDatasetChecksum(1),
        ];

        let serialized = to_string(&causes).expect("Failed to serialize");
        let expected = r#"[{"cause":"PRE_COMPUTE_DATASET_URL_MISSING","message":"Dataset URL related environment variable is missing for dataset 0"},{"cause":"PRE_COMPUTE_INVALID_DATASET_CHECKSUM","message":"Invalid dataset checksum for dataset 1"}]"#;
        assert_eq!(serialized, expected);
    }
    // endregion

    // region get_worker_api_client
    #[test]
    fn should_get_worker_api_client_with_env_var() {
        with_vars(
            vec![(WorkerHostEnvVar.name(), Some("custom-worker-host:9999"))],
            || {
                let client = WorkerApiClient::from_env();
                assert_eq!(client.base_url, "http://custom-worker-host:9999");
            },
        );
    }

    #[test]
    fn should_get_worker_api_client_without_env_var() {
        temp_env::with_vars_unset(vec![WorkerHostEnvVar.name()], || {
            let client = WorkerApiClient::from_env();
            assert_eq!(client.base_url, format!("http://{DEFAULT_WORKER_HOST}"));
        });
    }
    // endregion

    // region send_exit_causes_for_pre_compute_stage()
    const CHALLENGE: &str = "challenge";
    const CHAIN_TASK_ID: &str = "0x123456789abcdef";

    #[tokio::test]
    async fn should_send_exit_causes() {
        let mock_server = MockServer::start().await;
        let server_url = mock_server.uri();

        let expected_body = json!([
            {
                "cause": "PRE_COMPUTE_INVALID_TEE_SIGNATURE",
                "message": "Invalid TEE signature"
            }
        ]);

        Mock::given(method("POST"))
            .and(path(format!("/compute/pre/{CHAIN_TASK_ID}/exit")))
            .and(header("Authorization", CHALLENGE))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = tokio::task::spawn_blocking(move || {
            let exit_causes = vec![ReplicateStatusCause::PreComputeInvalidTeeSignature];
            let worker_api_client = WorkerApiClient::new(&server_url);
            worker_api_client.send_exit_causes_for_pre_compute_stage(
                CHALLENGE,
                CHAIN_TASK_ID,
                &exit_causes,
            )
        })
        .await
        .expect("Task panicked");

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_not_send_exit_causes() {
        testing_logger::setup();
        let mock_server = MockServer::start().await;
        let server_url = mock_server.uri();

        Mock::given(method("POST"))
            .and(path(format!("/compute/pre/{CHAIN_TASK_ID}/exit")))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = tokio::task::spawn_blocking(move || {
            let exit_causes = vec![ReplicateStatusCause::PreComputeFailedUnknownIssue];
            let worker_api_client = WorkerApiClient::new(&server_url);
            let response = worker_api_client.send_exit_causes_for_pre_compute_stage(
                CHALLENGE,
                CHAIN_TASK_ID,
                &exit_causes,
            );
            testing_logger::validate(|captured_logs| {
                let logs = captured_logs
                    .iter()
                    .filter(|c| c.level == log::Level::Error)
                    .collect::<Vec<&testing_logger::CapturedLog>>();

                assert_eq!(logs.len(), 1);
                assert_eq!(
                    logs[0].body,
                    "Failed to send exit causes: [status:503 Service Unavailable, body:Service Unavailable]"
                );
            });
            response
        })
        .await
        .expect("Task panicked");

        assert!(result.is_err());
        assert_eq!(
            result,
            Err(ReplicateStatusCause::PreComputeFailedUnknownIssue)
        );
    }

    #[test]
    fn test_send_exit_causes_http_request_failure() {
        testing_logger::setup();
        let exit_causes = vec![ReplicateStatusCause::PreComputeFailedUnknownIssue];
        let worker_api_client = WorkerApiClient::new("wrong_url");
        let result = worker_api_client.send_exit_causes_for_pre_compute_stage(
            CHALLENGE,
            CHAIN_TASK_ID,
            &exit_causes,
        );
        testing_logger::validate(|captured_logs| {
            let logs = captured_logs
                .iter()
                .filter(|c| c.level == log::Level::Error)
                .collect::<Vec<&testing_logger::CapturedLog>>();

            assert_eq!(logs.len(), 1);
            assert_eq!(
                logs[0].body,
                "HTTP request failed when sending exit causes to wrong_url/compute/pre/0x123456789abcdef/exit: reqwest::Error { kind: Builder, source: RelativeUrlWithoutBase }"
            );
        });
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(ReplicateStatusCause::PreComputeFailedUnknownIssue)
        );
    }
    // endregion
}
