use crate::compute::{
    computed_file::ComputedFile,
    errors::ReplicateStatusCause,
    utils::env_utils::{TeeSessionEnvironmentVariable, get_env_var_or_error},
};
use log::error;
use reqwest::{blocking::Client, header::AUTHORIZATION};

/// Thin wrapper around a [`Client`] that knows how to reach the iExec worker API.
///
/// This client can be created directly with a base URL using [`WorkerApiClient::new`], or
/// configured from environment variables using [`WorkerApiClient::from_env`].
///
/// # Example
///
/// ```rust
/// use tee_worker_post_compute::api::worker_api::WorkerApiClient;
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
            client: Client::builder().build().unwrap(),
        }
    }

    /// Creates a new WorkerApiClient instance with configuration from environment variables.
    ///
    /// This method retrieves the worker host from the [`TeeSessionEnvironmentVariable::WorkerHostEnvVar`] environment variable.
    /// If the variable is not set or empty, it defaults to `"worker:13100"`.
    ///
    /// # Returns
    ///
    /// * `WorkerApiClient` - A new client configured with the appropriate base URL
    ///
    /// # Example
    ///
    /// ```rust
    /// use tee_worker_post_compute::api::worker_api::WorkerApiClient;
    ///
    /// let client = WorkerApiClient::from_env();
    /// ```
    pub fn from_env() -> Self {
        let worker_host = get_env_var_or_error(
            TeeSessionEnvironmentVariable::WorkerHostEnvVar,
            ReplicateStatusCause::PostComputeWorkerAddressMissing,
        )
        .unwrap_or_else(|_| DEFAULT_WORKER_HOST.to_string());

        let base_url = format!("http://{worker_host}");
        Self::new(&base_url)
    }

    /// Sends an exit cause for a post-compute operation to the Worker API.
    ///
    /// This method reports the exit cause of a post-compute operation to the Worker API,
    /// which can be used for tracking and debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `authorization` - The authorization token to use for the API request
    /// * `chain_task_id` - The chain task ID for which to report the exit cause
    /// * `exit_cause` - The exit cause to report
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the exit cause was successfully reported
    /// * `Err(ReplicateStatusCause)` - If the exit cause could not be reported due to an HTTP error
    ///
    /// # Errors
    ///
    /// This function will return an [`tee_worker_post_compute::compute::errors::ReplicateStatusCause`]
    /// if the request could not be sent or the server responded with a non‑success status.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tee_worker_post_compute::{
    ///     api::worker_api::WorkerApiClient,
    ///     compute::errors::ReplicateStatusCause,
    /// };
    ///
    /// let client = WorkerApiClient::new("http://worker:13100");
    /// let exit_causes = vec![ReplicateStatusCause::PostComputeInvalidTeeSignature];
    ///
    /// match client.send_exit_cause_for_post_compute_stage(
    ///     "authorization_token",
    ///     "0x123456789abcdef",
    ///     &exit_causes,
    /// ) {
    ///     Ok(()) => println!("Exit cause reported successfully"),
    ///     Err(error) => eprintln!("Failed to report exit cause: {}", error),
    /// }
    /// ```
    pub fn send_exit_cause_for_post_compute_stage(
        &self,
        authorization: &str,
        chain_task_id: &str,
        exit_causes: &[ReplicateStatusCause],
    ) -> Result<(), ReplicateStatusCause> {
        let url = format!("{}/compute/post/{chain_task_id}/exit", self.base_url);
        match self
            .client
            .post(&url)
            .header(AUTHORIZATION, authorization)
            .json(exit_causes)
            .send()
        {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    let status = response.status();
                    let body = response.text().unwrap_or_default();
                    error!(
                        "Failed to send exit cause to worker: [status:{status:?}, body:{body:#?}]"
                    );
                    Err(ReplicateStatusCause::PostComputeFailedUnknownIssue)
                }
            }
            Err(e) => {
                error!("An error occured while sending exit cause to worker: {e}");
                Err(ReplicateStatusCause::PostComputeFailedUnknownIssue)
            }
        }
    }

    /// Sends the completed computed.json file to the worker host.
    ///
    /// This method transmits the computed file containing task results, signatures,
    /// and metadata to the worker API. The computed file is sent as JSON in the
    /// request body, allowing the worker to verify and process the computation results.
    ///
    /// # Arguments
    ///
    /// * `authorization` - The authorization token/challenge to validate the request on the worker side
    /// * `chain_task_id` - The blockchain task identifier associated with this computation
    /// * `computed_file` - The computed file containing results and signatures to be sent
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the computed file was successfully sent (HTTP 2xx response)
    /// * `Err(Error)` - If the request failed due to an HTTP error
    ///
    /// # Example
    ///
    /// ```rust
    /// use tee_worker_post_compute::{
    ///     api::worker_api::WorkerApiClient,
    ///     compute::computed_file::ComputedFile,
    /// };
    ///
    /// let client = WorkerApiClient::new("http://worker:13100");
    /// let computed_file = ComputedFile {
    ///     task_id: Some("0x123456789abcdef".to_string()),
    ///     result_digest: Some("0xdigest".to_string()),
    ///     enclave_signature: Some("0xsignature".to_string()),
    ///     ..Default::default()
    /// };
    ///
    /// match client.send_computed_file_to_host(
    ///     "Bearer auth_token",
    ///     "0x123456789abcdef",
    ///     &computed_file,
    /// ) {
    ///     Ok(()) => println!("Computed file sent successfully"),
    ///     Err(error) => eprintln!("Failed to send computed file: {}", error),
    /// }
    /// ```
    pub fn send_computed_file_to_host(
        &self,
        authorization: &str,
        chain_task_id: &str,
        computed_file: &ComputedFile,
    ) -> Result<(), ReplicateStatusCause> {
        let url = format!("{}/compute/post/{chain_task_id}/computed", self.base_url);
        match self
            .client
            .post(&url)
            .header(AUTHORIZATION, authorization)
            .json(computed_file)
            .send()
        {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    let status = response.status();
                    let body = response.text().unwrap_or_default();
                    error!(
                        "Failed to send computed file to worker: [status:{status:?}, body:{body:#?}]"
                    );
                    Err(ReplicateStatusCause::PostComputeSendComputedFileFailed)
                }
            }
            Err(e) => {
                error!("An error occured while sending computed file to worker: {e}");
                Err(ReplicateStatusCause::PostComputeSendComputedFileFailed)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::utils::env_utils::TeeSessionEnvironmentVariable::*;
    use logtest::Logger;
    use once_cell::sync::Lazy;
    use serde_json::{json, to_string};
    use serial_test::serial;
    use std::sync::Mutex;
    use temp_env::with_vars;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, header, method, path},
    };

    static TEST_LOGGER: Lazy<Mutex<Logger>> = Lazy::new(|| Mutex::new(Logger::start()));

    const CHALLENGE: &str = "challenge";
    const CHAIN_TASK_ID: &str = "0x123456789abcdef";

    // region serialize List of ReplicateStatusCause
    #[test]
    fn should_serialize_list_of_exit_causes() {
        let causes = vec![
            ReplicateStatusCause::PostComputeInvalidTeeSignature,
            ReplicateStatusCause::PostComputeWorkerAddressMissing,
        ];
        let serialized = to_string(&causes).expect("Failed to serialize");
        let expected = r#"[{"cause":"POST_COMPUTE_INVALID_TEE_SIGNATURE","message":"Invalid TEE signature"},{"cause":"POST_COMPUTE_WORKER_ADDRESS_MISSING","message":"Worker address related environment variable is missing"}]"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn should_serialize_single_exit_cause() {
        let causes = vec![ReplicateStatusCause::PostComputeFailedUnknownIssue];
        let serialized = to_string(&causes).expect("Failed to serialize");
        let expected = r#"[{"cause":"POST_COMPUTE_FAILED_UNKNOWN_ISSUE","message":"Unexpected error occurred"}]"#;
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
        with_vars(vec![(WorkerHostEnvVar.name(), None::<&str>)], || {
            let client = WorkerApiClient::from_env();
            assert_eq!(client.base_url, format!("http://{DEFAULT_WORKER_HOST}"));
        });
    }
    // endregion

    // region send_exit_cause_for_post_compute_stage()
    #[tokio::test]
    async fn should_send_exit_cause() {
        let mock_server = MockServer::start().await;
        let server_url = mock_server.uri();

        let expected_body = json!([ReplicateStatusCause::PostComputeInvalidTeeSignature,]);

        Mock::given(method("POST"))
            .and(path(format!("/compute/post/{CHAIN_TASK_ID}/exit")))
            .and(header("Authorization", CHALLENGE))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = tokio::task::spawn_blocking(move || {
            let exit_causes = vec![ReplicateStatusCause::PostComputeInvalidTeeSignature];
            let worker_api_client = WorkerApiClient::new(&server_url);
            worker_api_client.send_exit_cause_for_post_compute_stage(
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
    #[serial]
    async fn should_not_send_exit_cause() {
        {
            let mut logger = TEST_LOGGER.lock().unwrap();
            while logger.pop().is_some() {}
        }
        let mock_server = MockServer::start().await;
        let server_url = mock_server.uri();

        Mock::given(method("POST"))
            .and(path(format!("/compute/post/{CHAIN_TASK_ID}/exit")))
            .respond_with(ResponseTemplate::new(404))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = tokio::task::spawn_blocking(move || {
            let exit_causes = vec![ReplicateStatusCause::PostComputeFailedUnknownIssue];
            let worker_api_client = WorkerApiClient::new(&server_url);
            worker_api_client.send_exit_cause_for_post_compute_stage(
                CHALLENGE,
                CHAIN_TASK_ID,
                &exit_causes,
            )
        })
        .await
        .expect("Task panicked");

        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(
                error,
                ReplicateStatusCause::PostComputeFailedUnknownIssue,
                "Expected PostComputeFailedUnknownIssue, got: {error:?}"
            );
        }
        let mut logger = TEST_LOGGER.lock().unwrap();
        let mut found = false;
        while let Some(rec) = logger.pop() {
            if rec.args().contains("status:404") {
                found = true;
                break;
            }
        }
        assert!(found, "Expected log to contain HTTP 404 status");
    }
    // endregion

    // region send_computed_file_to_host()
    #[tokio::test]
    async fn should_send_computed_file_successfully() {
        let mock_server = MockServer::start().await;
        let server_uri = mock_server.uri();

        let computed_file = ComputedFile {
            task_id: Some(CHAIN_TASK_ID.to_string()),
            result_digest: Some("0xdigest".to_string()),
            enclave_signature: Some("0xsignature".to_string()),
            ..Default::default()
        };

        let expected_path = format!("/compute/post/{CHAIN_TASK_ID}/computed");
        let expected_body = json!(computed_file);

        Mock::given(method("POST"))
            .and(path(expected_path.as_str()))
            .and(header("Authorization", CHALLENGE))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = tokio::task::spawn_blocking(move || {
            let client = WorkerApiClient::new(&server_uri);
            client.send_computed_file_to_host(CHALLENGE, CHAIN_TASK_ID, &computed_file)
        })
        .await
        .expect("Task panicked");

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn should_fail_send_computed_file_on_server_error() {
        {
            let mut logger = TEST_LOGGER.lock().unwrap();
            while logger.pop().is_some() {}
        }
        let mock_server = MockServer::start().await;
        let server_uri = mock_server.uri();

        let computed_file = ComputedFile {
            task_id: Some(CHAIN_TASK_ID.to_string()),
            result_digest: Some("0xdigest".to_string()),
            enclave_signature: Some("0xsignature".to_string()),
            ..Default::default()
        };
        let expected_path = format!("/compute/post/{CHAIN_TASK_ID}/computed");
        let expected_body = json!(computed_file);

        Mock::given(method("POST"))
            .and(path(expected_path.as_str()))
            .and(header("Authorization", CHALLENGE))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = tokio::task::spawn_blocking(move || {
            let client = WorkerApiClient::new(&server_uri);
            client.send_computed_file_to_host(CHALLENGE, CHAIN_TASK_ID, &computed_file)
        })
        .await
        .expect("Task panicked");

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(
                error,
                ReplicateStatusCause::PostComputeSendComputedFileFailed,
                "Expected PostComputeSendComputedFileFailed, got: {error:?}"
            );
        }
        let mut logger = TEST_LOGGER.lock().unwrap();
        let mut found = false;
        while let Some(rec) = logger.pop() {
            if rec.args().contains("status:500") {
                found = true;
                break;
            }
        }
        assert!(found, "Expected log to contain HTTP 500 status");
    }

    #[tokio::test]
    #[serial]
    async fn should_handle_invalid_chain_task_id_in_url() {
        {
            let mut logger = TEST_LOGGER.lock().unwrap();
            while logger.pop().is_some() {}
        }
        let mock_server = MockServer::start().await;
        let server_uri = mock_server.uri();

        let invalid_chain_task_id = "invalidTaskId";
        let computed_file = ComputedFile {
            task_id: Some(invalid_chain_task_id.to_string()),
            ..Default::default()
        };

        let result = tokio::task::spawn_blocking(move || {
            let client = WorkerApiClient::new(&server_uri);
            client.send_computed_file_to_host(CHALLENGE, invalid_chain_task_id, &computed_file)
        })
        .await
        .expect("Task panicked");

        assert!(result.is_err(), "Should fail with invalid chain task ID");
        if let Err(error) = result {
            assert_eq!(
                error,
                ReplicateStatusCause::PostComputeSendComputedFileFailed,
                "Expected PostComputeSendComputedFileFailed, got: {error:?}"
            );
        }
        let mut logger = TEST_LOGGER.lock().unwrap();
        let mut found = false;
        while let Some(rec) = logger.pop() {
            if rec.args().contains("status:404") {
                found = true;
                break;
            }
        }
        assert!(found, "Expected log to contain HTTP 404 status");
    }

    #[tokio::test]
    async fn should_send_computed_file_with_minimal_data() {
        let mock_server = MockServer::start().await;
        let server_uri = mock_server.uri();

        let computed_file = ComputedFile {
            task_id: Some(CHAIN_TASK_ID.to_string()),
            ..Default::default()
        };

        let expected_path = format!("/compute/post/{CHAIN_TASK_ID}/computed");
        let expected_body = json!(computed_file);

        Mock::given(method("POST"))
            .and(path(expected_path.as_str()))
            .and(header("Authorization", CHALLENGE))
            .and(body_json(&expected_body))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = tokio::task::spawn_blocking(move || {
            let client = WorkerApiClient::new(&server_uri);
            client.send_computed_file_to_host(CHALLENGE, CHAIN_TASK_ID, &computed_file)
        })
        .await
        .expect("Task panicked");

        assert!(result.is_ok());
    }
    // endregion
}
