use crate::compute::errors::ReplicateStatusCause;
use crate::compute::pre_compute_args::PreComputeArgs;
use crate::compute::utils::env_utils::{TeeSessionEnvironmentVariable, get_env_var_or_error};
use crate::compute::utils::file_utils::{download_file, write_file};
use crate::compute::utils::hash_utils::sha256;
use log::{error, info};
#[cfg(test)]
use mockall::automock;
use std::path::{Path, PathBuf};

#[cfg_attr(test, automock)]
pub trait PreComputeAppTrait {
    fn run(&mut self) -> Result<(), Vec<ReplicateStatusCause>>;
    fn check_output_folder(&self) -> Result<(), ReplicateStatusCause>;
    fn download_input_files(&self) -> Result<(), Vec<ReplicateStatusCause>>;
    fn save_plain_dataset_file(
        &self,
        plain_content: &[u8],
        plain_dataset_filename: &str,
    ) -> Result<(), ReplicateStatusCause>;
}

pub struct PreComputeApp {
    chain_task_id: String,
    pre_compute_args: PreComputeArgs,
}

impl PreComputeApp {
    pub fn new(chain_task_id: String) -> Self {
        PreComputeApp {
            chain_task_id,
            pre_compute_args: PreComputeArgs::default(),
        }
    }
}

impl PreComputeAppTrait for PreComputeApp {
    /// Runs the complete pre-compute pipeline.
    ///
    /// This method orchestrates the entire pre-compute process:
    /// 1. Reads the output directory from environment variable `IEXEC_PRE_COMPUTE_OUT`
    /// 2. Reads and validates configuration arguments from environment variables
    /// 3. Validates the output folder exists
    /// 4. Downloads and decrypts all datasets (if required)
    /// 5. Downloads all input files
    ///
    /// The method collects all errors encountered during execution and returns them together,
    /// allowing partial completion when possible (e.g., if one dataset fails, others are still processed).
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all operations completed successfully
    /// - `Err(Vec<ReplicateStatusCause>)` containing all errors encountered during execution
    ///
    /// # Example
    ///
    /// ```rust
    /// use tee_worker_pre_compute::compute::pre_compute_app::{PreComputeApp, PreComputeAppTrait};
    ///
    /// let mut app = PreComputeApp::new("task_id".to_string());
    /// app.run();
    /// ```
    fn run(&mut self) -> Result<(), Vec<ReplicateStatusCause>> {
        let (mut args, mut exit_causes): (PreComputeArgs, Vec<ReplicateStatusCause>);
        match get_env_var_or_error(
            TeeSessionEnvironmentVariable::IexecPreComputeOut,
            ReplicateStatusCause::PreComputeOutputPathMissing,
        ) {
            Ok(output_dir) => {
                (args, exit_causes) = PreComputeArgs::read_args();
                args.output_dir = output_dir;
            }
            Err(e) => {
                error!("Failed to read output directory: {e:?}");
                return Err(vec![e]);
            }
        };
        self.pre_compute_args = args;

        if let Err(exit_cause) = self.check_output_folder() {
            return Err(vec![exit_cause]);
        }

        for dataset in self.pre_compute_args.datasets.iter() {
            if let Err(exit_cause) = dataset
                .download_encrypted_dataset(&self.chain_task_id)
                .and_then(|encrypted_content| dataset.decrypt_dataset(&encrypted_content))
                .and_then(|plain_content| {
                    self.save_plain_dataset_file(&plain_content, &dataset.filename)
                })
            {
                exit_causes.push(exit_cause);
            };
        }
        if let Err(exit_cause) = self.download_input_files() {
            exit_causes.extend(exit_cause);
        };
        if !exit_causes.is_empty() {
            Err(exit_causes)
        } else {
            Ok(())
        }
    }

    /// Checks whether the output folder specified in `pre_compute_args` exists.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the output directory (`output_dir`) exists.
    /// - `Err(ReplicateStatusCause::PreComputeOutputFolderNotFound)` if the directory does not exist,
    ///   or if `pre_compute_args` is missing.
    fn check_output_folder(&self) -> Result<(), ReplicateStatusCause> {
        let output_dir: &str = &self.pre_compute_args.output_dir;
        let chain_task_id: &str = &self.chain_task_id;

        info!("Checking output folder [chainTaskId:{chain_task_id}, path:{output_dir}]");

        if Path::new(&output_dir).is_dir() {
            return Ok(());
        }

        error!("Output folder not found [chainTaskId:{chain_task_id}, path:{output_dir}]");

        Err(ReplicateStatusCause::PreComputeOutputFolderNotFound)
    }

    /// Downloads the input files listed in `pre_compute_args.input_files` to the specified `output_dir`.
    ///
    /// Each URL is hashed (SHA-256) to generate a unique local filename.
    /// The method continues downloading all files even if some downloads fail.
    ///
    /// # Behavior
    ///
    /// - Downloads continue even when individual files fail
    /// - Successfully downloaded files are saved with SHA-256 hashed filenames
    /// - All download failures are collected and returned together
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all files are downloaded successfully
    /// - `Err(Vec<ReplicateStatusCause>)` containing a `PreComputeInputFileDownloadFailed` error
    ///   for each file that failed to download
    fn download_input_files(&self) -> Result<(), Vec<ReplicateStatusCause>> {
        let mut exit_causes: Vec<ReplicateStatusCause> = Vec::new();
        let args = &self.pre_compute_args;
        let chain_task_id: &str = &self.chain_task_id;

        for url in args.input_files.iter() {
            info!("Downloading input file [chainTaskId:{chain_task_id}, url:{url}]");

            let filename = sha256(url.to_string());
            if download_file(url, &args.output_dir, &filename).is_none() {
                exit_causes.push(ReplicateStatusCause::PreComputeInputFileDownloadFailed(
                    url.to_string(),
                ));
            }
        }

        if !exit_causes.is_empty() {
            Err(exit_causes)
        } else {
            Ok(())
        }
    }

    /// Saves the decrypted (plain) dataset to disk in the configured output directory.
    ///
    /// The output filename is taken from `pre_compute_args.plain_dataset_filename`.
    ///
    /// # Arguments
    ///
    /// * `plain_dataset` - The dataset content to write to a file.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the file is successfully saved.
    /// * `Err(ReplicateStatusCause::PreComputeSavingPlainDatasetFailed)` if the path is invalid or write fails.
    fn save_plain_dataset_file(
        &self,
        plain_dataset: &[u8],
        plain_dataset_filename: &str,
    ) -> Result<(), ReplicateStatusCause> {
        let chain_task_id: &str = &self.chain_task_id;
        let args = &self.pre_compute_args;
        let output_dir: &str = &args.output_dir;

        let mut path = PathBuf::from(output_dir);
        path.push(plain_dataset_filename);

        info!(
            "Saving plain dataset file [chain_task_id:{chain_task_id}, path:{}]",
            path.display()
        );

        write_file(
            plain_dataset,
            &path,
            &format!("chainTaskId:{chain_task_id}"),
        )
        .map_err(|_| ReplicateStatusCause::PreComputeSavingPlainDatasetFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::dataset::Dataset;
    use crate::compute::pre_compute_args::PreComputeArgs;
    use std::fs;
    use tempfile::TempDir;
    use testcontainers::core::WaitFor;
    use testcontainers::runners::SyncRunner;
    use testcontainers::{Container, GenericImage};

    const CHAIN_TASK_ID: &str = "0x123456789abcdef";
    const DATASET_CHECKSUM: &str =
        "0x02a12ef127dcfbdb294a090c8f0b69a0ca30b7940fc36cabf971f488efd374d7";
    const ENCRYPTED_DATASET_KEY: &str = "ubA6H9emVPJT91/flYAmnKHC0phSV3cfuqsLxQfgow0=";
    const HTTP_DATASET_URL: &str = "https://raw.githubusercontent.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/main/src/tests_resources/encrypted-data.bin";
    const PLAIN_DATA_FILE: &str = "plain-data.txt";

    fn get_pre_compute_app(
        chain_task_id: &str,
        urls: Vec<&str>,
        output_dir: &str,
    ) -> PreComputeApp {
        PreComputeApp {
            chain_task_id: chain_task_id.to_string(),
            pre_compute_args: PreComputeArgs {
                input_files: urls.into_iter().map(String::from).collect(),
                output_dir: output_dir.to_string(),
                is_dataset_required: true,
                iexec_bulk_slice_size: 0,
                datasets: vec![Dataset {
                    url: HTTP_DATASET_URL.to_string(),
                    checksum: DATASET_CHECKSUM.to_string(),
                    filename: PLAIN_DATA_FILE.to_string(),
                    key: ENCRYPTED_DATASET_KEY.to_string(),
                }],
            },
        }
    }

    fn start_container() -> (Container<GenericImage>, String, String) {
        let container = GenericImage::new("kennethreitz/httpbin", "latest")
            .with_wait_for(WaitFor::message_on_stderr("Listening at"))
            .start()
            .expect("Failed to start Httpbin");
        let port = container
            .get_host_port_ipv4(80)
            .expect("Could not get host port");

        let json_url = format!("http://127.0.0.1:{port}/json");
        let xml_url = format!("http://127.0.0.1:{port}/xml");

        (container, json_url, xml_url)
    }

    // region check_output_folder
    #[test]
    fn check_output_folder_returns_ok_with_valid_args() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().to_str().unwrap();

        let app = get_pre_compute_app(CHAIN_TASK_ID, vec![], output_path);

        let result = app.check_output_folder();
        assert!(result.is_ok());
    }

    #[test]
    fn check_output_folder_returns_err_with_invalid_file_path() {
        let non_existing_path = "/tmp/some_non_existing_output_dir_xyz_123".to_string();

        let app = get_pre_compute_app(CHAIN_TASK_ID, vec![], &non_existing_path);

        let result = app.check_output_folder();
        assert_eq!(
            result,
            Err(ReplicateStatusCause::PreComputeOutputFolderNotFound)
        );
    }

    // endregion

    // region download_input_files
    #[test]
    fn download_input_files_success_with_single_file() {
        let (_container, json_url, _) = start_container();

        let temp_dir = TempDir::new().unwrap();
        let app = get_pre_compute_app(
            CHAIN_TASK_ID,
            vec![&json_url],
            temp_dir.path().to_str().unwrap(),
        );

        let result = app.download_input_files();
        assert!(result.is_ok());

        let url_hash = sha256(json_url);
        let downloaded_file = temp_dir.path().join(url_hash);
        assert!(downloaded_file.exists());
    }

    #[test]
    fn download_input_files_success_with_multiple_files() {
        let (_container, json_url, xml_url) = start_container();

        let temp_dir = TempDir::new().unwrap();
        let app = get_pre_compute_app(
            CHAIN_TASK_ID,
            vec![&json_url, &xml_url],
            temp_dir.path().to_str().unwrap(),
        );

        let result = app.download_input_files();
        assert!(result.is_ok());

        let json_hash = sha256(json_url);
        let xml_hash = sha256(xml_url);

        assert!(temp_dir.path().join(json_hash).exists());
        assert!(temp_dir.path().join(xml_hash).exists());
    }

    #[test]
    fn test_download_failure_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let app = get_pre_compute_app(
            CHAIN_TASK_ID,
            vec!["https://invalid-url-that-should-fail.com/file.txt"],
            temp_dir.path().to_str().unwrap(),
        );

        let result = app.download_input_files();
        assert_eq!(
            result.unwrap_err(),
            vec![ReplicateStatusCause::PreComputeInputFileDownloadFailed(
                "https://invalid-url-that-should-fail.com/file.txt".to_string()
            )]
        );
    }

    #[test]
    fn test_partial_failure_dont_stops_on_first_error() {
        let (_container, json_url, xml_url) = start_container();

        let temp_dir = TempDir::new().unwrap();
        let app = get_pre_compute_app(
            CHAIN_TASK_ID,
            vec![
                &json_url,                                           // This should succeed
                "https://invalid-url-that-should-fail.com/file.txt", // This should fail
                &xml_url,                                            // This should succeed
            ],
            temp_dir.path().to_str().unwrap(),
        );

        let result = app.download_input_files();
        assert_eq!(
            result.unwrap_err(),
            vec![ReplicateStatusCause::PreComputeInputFileDownloadFailed(
                "https://invalid-url-that-should-fail.com/file.txt".to_string()
            )]
        );

        // First file should be downloaded with SHA256 filename
        let json_hash = sha256(json_url);
        assert!(temp_dir.path().join(json_hash).exists());

        // Third file should be downloaded (not stopped on second failure)
        let xml_hash = sha256(xml_url);
        assert!(temp_dir.path().join(xml_hash).exists());
    }
    // endregion

    // region save_plain_dataset_file
    #[test]
    fn save_plain_dataset_file_success_with_valid_output_dir() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().to_str().unwrap();

        let app = get_pre_compute_app(CHAIN_TASK_ID, vec![], output_path);

        let plain_dataset = "Some very useful data.".as_bytes().to_vec();
        let saved_dataset = app.save_plain_dataset_file(&plain_dataset, PLAIN_DATA_FILE);

        assert!(saved_dataset.is_ok());

        let expected_file_path = temp_dir.path().join(PLAIN_DATA_FILE);
        assert!(
            expected_file_path.exists(),
            "The dataset file should have been created."
        );

        let file_content =
            fs::read(&expected_file_path).expect("Should be able to read the created file");
        assert_eq!(
            file_content, plain_dataset,
            "File content should match the original data."
        );
    }

    #[test]
    fn save_plain_dataset_file_failure_with_invalid_output_dir() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().to_str().unwrap();

        let app = get_pre_compute_app(CHAIN_TASK_ID, vec![], output_path);
        let plain_dataset = "Some very useful data.".as_bytes().to_vec();
        let saved_dataset =
            app.save_plain_dataset_file(&plain_dataset, "/some-folder-123/not-found");

        assert_eq!(
            saved_dataset,
            Err(ReplicateStatusCause::PreComputeSavingPlainDatasetFailed)
        );
    }
    // endregion
}
