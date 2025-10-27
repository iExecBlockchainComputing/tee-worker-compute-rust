use crate::compute::dataset::Dataset;
use crate::compute::errors::ReplicateStatusCause;
use crate::compute::utils::env_utils::{TeeSessionEnvironmentVariable, get_env_var_or_error};
use log::{error, info};

/// Represents parameters required for pre-compute tasks in a Trusted Execution Environment (TEE).
///
/// This structure aggregates configuration parameters from environment variables and task context,
/// providing a validated interface for subsequent computation phases.
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Default)]
pub struct PreComputeArgs {
    pub output_dir: String,
    // Dataset related fields
    pub is_dataset_required: bool,
    // Input files
    pub input_files: Vec<String>,
    // Bulk processing
    pub iexec_bulk_slice_size: usize,
    pub datasets: Vec<Dataset>,
}

impl PreComputeArgs {
    /// Constructs a validated `PreComputeArgs` instance by reading and validating environment variables.
    ///
    /// # Environment Variables
    /// This method reads the following environment variables:
    /// - Required for all tasks:
    ///   - `IEXEC_PRE_COMPUTE_OUT`: Output directory path
    ///   - `IEXEC_DATASET_REQUIRED`: Boolean ("true"/"false") indicating dataset requirement
    ///   - `IEXEC_INPUT_FILES_NUMBER`: Number of input files to load
    ///   - `IEXEC_BULK_SLICE_SIZE`: Number of bulk datasets (0 means no bulk processing)
    /// - Required when `IEXEC_DATASET_REQUIRED` = "true":
    ///   - `IEXEC_DATASET_URL`: Encrypted dataset URL
    ///   - `IEXEC_DATASET_KEY`: Base64-encoded dataset encryption key
    ///   - `IEXEC_DATASET_CHECKSUM`: Encrypted dataset checksum
    ///   - `IEXEC_DATASET_FILENAME`: Decrypted dataset filename
    /// - Required when `IEXEC_BULK_SLICE_SIZE` > 0 (for each dataset index from 1 to IEXEC_BULK_SLICE_SIZE):
    ///   - `IEXEC_DATASET_#_URL`: Dataset URL
    ///   - `IEXEC_DATASET_#_CHECKSUM`: Dataset checksum
    ///   - `IEXEC_DATASET_#_FILENAME`: Dataset filename
    ///   - `IEXEC_DATASET_#_KEY`: Dataset decryption key
    /// - Input file URLs (`IEXEC_INPUT_FILE_URL_1`, `IEXEC_INPUT_FILE_URL_2`, etc.)
    ///
    /// # Errors
    /// Returns `ReplicateStatusCause` error variants for:
    /// - Missing required environment variables
    /// - Invalid boolean values in `IEXEC_DATASET_REQUIRED`
    /// - Invalid numeric format in `IEXEC_INPUT_FILES_NUMBER` or `IEXEC_BULK_SLICE_SIZE`
    /// - Missing dataset parameters when required
    /// - Missing input file URLs
    /// - Missing bulk dataset parameters when bulk processing is enabled
    ///
    /// # Example
    ///
    /// ```rust
    /// use tee_worker_pre_compute::compute::pre_compute_args::PreComputeArgs;
    ///
    /// // Typically called with task ID from execution context
    /// let args = PreComputeArgs::read_args();
    /// ```
    pub fn read_args() -> (PreComputeArgs, Vec<ReplicateStatusCause>) {
        info!("Starting to read pre-compute arguments from environment variables");
        let mut exit_causes: Vec<ReplicateStatusCause> = vec![];

        let output_dir = match get_env_var_or_error(
            TeeSessionEnvironmentVariable::IexecPreComputeOut,
            ReplicateStatusCause::PreComputeOutputPathMissing,
        ) {
            Ok(output_dir) => {
                info!("Successfully read output directory: {output_dir}");
                output_dir
            }
            Err(e) => {
                error!("Failed to read output directory: {e:?}");
                return (PreComputeArgs::default(), vec![e]);
            }
        };

        let is_dataset_required = match get_env_var_or_error(
            TeeSessionEnvironmentVariable::IsDatasetRequired,
            ReplicateStatusCause::PreComputeIsDatasetRequiredMissing,
        ) {
            Ok(s) => match s.to_lowercase().parse::<bool>() {
                Ok(value) => {
                    info!("Dataset required: {value}");
                    value
                }
                Err(_) => {
                    error!("Invalid boolean format for IS_DATASET_REQUIRED: {s}");
                    exit_causes.push(ReplicateStatusCause::PreComputeIsDatasetRequiredMissing);
                    false
                }
            },
            Err(e) => {
                error!("Failed to read IS_DATASET_REQUIRED: {e:?}");
                exit_causes.push(e);
                false
            }
        };

        let iexec_bulk_slice_size = match get_env_var_or_error(
            TeeSessionEnvironmentVariable::IexecBulkSliceSize,
            ReplicateStatusCause::PreComputeFailedUnknownIssue,
        ) {
            Ok(s) => match s.parse::<usize>() {
                Ok(value) => {
                    info!("Bulk slice size: {value}");
                    value
                }
                Err(_) => {
                    error!("Invalid numeric format for IEXEC_BULK_SLICE_SIZE: {s}");
                    exit_causes.push(ReplicateStatusCause::PreComputeFailedUnknownIssue);
                    0
                }
            },
            Err(e) => {
                error!("Failed to read IEXEC_BULK_SLICE_SIZE: {e:?}");
                exit_causes.push(e);
                0
            }
        }; // TODO: replace with a more specific error

        let mut datasets = Vec::with_capacity(iexec_bulk_slice_size + 1);

        // Read datasets
        let start_index = if is_dataset_required { 0 } else { 1 };
        info!(
            "Reading datasets from index {start_index} to {iexec_bulk_slice_size} (is_dataset_required: {is_dataset_required})"
        );
        
        for i in start_index..=iexec_bulk_slice_size {
            info!("Processing dataset at index {i}");
            
            let filename = match get_env_var_or_error(
                TeeSessionEnvironmentVariable::IexecDatasetFilename(i),
                ReplicateStatusCause::PreComputeDatasetFilenameMissing(format!("dataset_{i}")),
            ) {
                Ok(filename) => {
                    info!("Dataset {i} filename: {filename}");
                    filename
                }
                Err(e) => {
                    error!("Failed to read dataset {i} filename: {e:?}");
                    exit_causes.push(e);
                    continue;
                }
            };

            let url = match get_env_var_or_error(
                TeeSessionEnvironmentVariable::IexecDatasetUrl(i),
                ReplicateStatusCause::PreComputeDatasetUrlMissing(filename.clone()),
            ) {
                Ok(url) => {
                    info!("Dataset {i} URL: {url}");
                    url
                }
                Err(e) => {
                    error!("Failed to read dataset {i} URL: {e:?}");
                    exit_causes.push(e);
                    continue;
                }
            };

            let checksum = match get_env_var_or_error(
                TeeSessionEnvironmentVariable::IexecDatasetChecksum(i),
                ReplicateStatusCause::PreComputeDatasetChecksumMissing(filename.clone()),
            ) {
                Ok(checksum) => {
                    info!("Dataset {i} checksum: {checksum}");
                    checksum
                }
                Err(e) => {
                    error!("Failed to read dataset {i} checksum: {e:?}");
                    exit_causes.push(e);
                    continue;
                }
            };

            let key = match get_env_var_or_error(
                TeeSessionEnvironmentVariable::IexecDatasetKey(i),
                ReplicateStatusCause::PreComputeDatasetKeyMissing(filename.clone()),
            ) {
                Ok(key) => {
                    info!("Dataset {i} key successfully read");
                    key
                }
                Err(e) => {
                    error!("Failed to read dataset {i} key: {e:?}");
                    exit_causes.push(e);
                    continue;
                }
            };

            info!("Successfully loaded dataset {i} ({filename})");
            datasets.push(Dataset::new(url, checksum, filename, key));
        }
        
        info!("Successfully loaded {} datasets", datasets.len());

        let input_files_nb = match get_env_var_or_error(
            TeeSessionEnvironmentVariable::IexecInputFilesNumber,
            ReplicateStatusCause::PreComputeInputFilesNumberMissing,
        ) {
            Ok(s) => match s.parse::<usize>() {
                Ok(value) => {
                    info!("Number of input files: {value}");
                    value
                }
                Err(_) => {
                    error!("Invalid numeric format for IEXEC_INPUT_FILES_NUMBER: {s}");
                    exit_causes.push(ReplicateStatusCause::PreComputeInputFilesNumberMissing);
                    0
                }
            },
            Err(e) => {
                error!("Failed to read IEXEC_INPUT_FILES_NUMBER: {e:?}");
                exit_causes.push(e);
                0
            }
        };

        info!("Reading {input_files_nb} input file URLs");
        let input_files: Vec<String> = (1..=input_files_nb)
            .filter_map(|i| {
                get_env_var_or_error(
                    TeeSessionEnvironmentVariable::IexecInputFileUrlPrefix(i),
                    ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(i),
                )
                .map_err(|e| {
                    error!("Failed to read input file {i} URL: {e:?}");
                    exit_causes.push(e)
                })
                .ok()
                .map(|url| {
                    info!("Input file {i} URL: {url}");
                    url
                })
            })
            .collect();
        
        info!("Successfully loaded {} input files", input_files.len());
        
        if !exit_causes.is_empty() {
            error!(
                "Encountered {} error(s) while reading pre-compute arguments",
                exit_causes.len()
            );
        } else {
            info!("Successfully read all pre-compute arguments without errors");
        }
        
        (
            PreComputeArgs {
                output_dir,
                is_dataset_required,
                input_files,
                iexec_bulk_slice_size,
                datasets,
            },
            exit_causes,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::errors::ReplicateStatusCause;
    use crate::compute::utils::env_utils::TeeSessionEnvironmentVariable::*;
    use std::collections::HashMap;

    const OUTPUT_DIR: &str = "/iexec_out";
    const DATASET_URL: &str = "https://dataset.url";
    const DATASET_KEY: &str = "datasetKey123";
    const DATASET_CHECKSUM: &str = "0x123checksum";
    const DATASET_FILENAME: &str = "dataset.txt";

    fn setup_basic_env_vars() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert(IexecPreComputeOut.name(), OUTPUT_DIR.to_string());
        vars.insert(IsDatasetRequired.name(), "true".to_string());
        vars.insert(IexecInputFilesNumber.name(), "0".to_string());
        vars.insert(IexecBulkSliceSize.name(), "0".to_string()); // Default to no bulk processing
        vars
    }

    fn setup_dataset_env_vars() -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert(IexecDatasetUrl(0).name(), DATASET_URL.to_string());
        vars.insert(IexecDatasetKey(0).name(), DATASET_KEY.to_string());
        vars.insert(IexecDatasetChecksum(0).name(), DATASET_CHECKSUM.to_string());
        vars.insert(IexecDatasetFilename(0).name(), DATASET_FILENAME.to_string());
        vars
    }

    fn setup_input_files_env_vars(count: usize) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert(IexecInputFilesNumber.name(), count.to_string());

        for i in 1..=count {
            vars.insert(
                IexecInputFileUrlPrefix(i).name(),
                format!("https://input-{i}.txt"),
            );
        }
        vars
    }

    // TODO: Collect all errors instead of propagating immediately, and return the list of errors
    fn setup_bulk_dataset_env_vars(count: usize) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert(IexecBulkSliceSize.name(), count.to_string());

        for i in 1..=count {
            vars.insert(
                IexecDatasetUrl(i).name(),
                format!("https://bulk-dataset-{i}.bin"),
            );
            vars.insert(IexecDatasetChecksum(i).name(), format!("0x{i}23checksum"));
            vars.insert(
                IexecDatasetFilename(i).name(),
                format!("bulk-dataset-{i}.txt"),
            );
            vars.insert(IexecDatasetKey(i).name(), format!("bulkKey{i}23"));
        }
        vars
    }

    fn to_temp_env_vars(map: HashMap<String, String>) -> Vec<(String, Option<String>)> {
        map.into_iter().map(|(k, v)| (k, Some(v))).collect()
    }

    // region Required environment variables
    #[test]
    fn read_args_succeeds_when_no_dataset() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.extend(setup_input_files_env_vars(1));
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            assert!(result.1.is_empty());
            let args = result.0;

            assert_eq!(args.output_dir, OUTPUT_DIR);
            assert!(!args.is_dataset_required);
            assert_eq!(args.input_files.len(), 1);
            assert_eq!(args.input_files[0], "https://input-1.txt");
            assert_eq!(args.iexec_bulk_slice_size, 0);
            assert_eq!(args.datasets.len(), 0);
        });
    }

    #[test]
    fn read_args_succeeds_when_dataset_exists() {
        let mut env_vars = setup_basic_env_vars();

        // Add dataset environment variables
        env_vars.extend(setup_dataset_env_vars());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            assert!(result.1.is_empty());
            let args = result.0;

            assert_eq!(args.output_dir, OUTPUT_DIR);
            assert!(args.is_dataset_required);
            assert_eq!(args.datasets[0].url, DATASET_URL.to_string());
            assert_eq!(args.datasets[0].key, DATASET_KEY.to_string());
            assert_eq!(args.datasets[0].checksum, DATASET_CHECKSUM.to_string());
            assert_eq!(args.datasets[0].filename, DATASET_FILENAME.to_string());
            assert_eq!(args.input_files.len(), 0);
            assert_eq!(args.iexec_bulk_slice_size, 0);
            assert_eq!(args.datasets.len(), 1);
        });
    }

    #[test]
    fn read_args_succeeds_when_multiple_inputs_exist() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());

        // Add input files environment variables
        env_vars.extend(setup_input_files_env_vars(3));

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            assert!(result.1.is_empty());
            let args = result.0;

            assert_eq!(args.output_dir, OUTPUT_DIR);
            assert!(!args.is_dataset_required);
            assert_eq!(args.input_files.len(), 3);
            assert_eq!(args.input_files[0], "https://input-1.txt");
            assert_eq!(args.input_files[1], "https://input-2.txt");
            assert_eq!(args.input_files[2], "https://input-3.txt");
            assert_eq!(args.iexec_bulk_slice_size, 0);
            assert_eq!(args.datasets.len(), 0);
        });
    }
    // endregion

    // region parsing tests
    #[test]
    fn read_args_succeeds_when_insensitive_bool_parsing() {
        let test_values = vec!["false", "FALSE", "False", "fAlSe"];
        for value_str in test_values {
            let mut env_vars = setup_basic_env_vars();
            env_vars.insert(IsDatasetRequired.name(), value_str.to_string());

            temp_env::with_vars(to_temp_env_vars(env_vars), || {
                let result = PreComputeArgs::read_args();
                assert!(result.1.is_empty());
                let args = result.0;
                assert!(!args.is_dataset_required);
            });
        }
    }

    #[test]
    fn read_args_fails_when_invalid_bool_format() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert("IS_DATASET_REQUIRED".to_string(), "not-a-bool".to_string());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();
            assert!(!result.1.is_empty());
            assert_eq!(
                result.1,
                vec![ReplicateStatusCause::PreComputeIsDatasetRequiredMissing]
            );
        });
    }

    #[test]
    fn read_args_fails_when_invalid_input_files_number_format() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(
            "IEXEC_INPUT_FILES_NUMBER".to_string(),
            "not-a-number".to_string(),
        );
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();
            assert!(!result.1.is_empty());
            assert_eq!(
                result.1,
                vec![ReplicateStatusCause::PreComputeInputFilesNumberMissing]
            );
        });
    }
    // endregion

    // region bulk processing tests
    #[test]
    fn read_args_succeeds_with_bulk_datasets() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(0));
        env_vars.extend(setup_bulk_dataset_env_vars(3));

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            assert!(result.1.is_empty());
            let args = result.0;

            assert_eq!(args.output_dir, OUTPUT_DIR);
            assert!(!args.is_dataset_required);
            assert_eq!(args.iexec_bulk_slice_size, 3);
            assert_eq!(args.datasets.len(), 3);
            assert_eq!(args.input_files.len(), 0);

            // Check first bulk dataset
            assert_eq!(args.datasets[0].url, "https://bulk-dataset-1.bin");
            assert_eq!(args.datasets[0].checksum, "0x123checksum");
            assert_eq!(args.datasets[0].filename, "bulk-dataset-1.txt");
            assert_eq!(args.datasets[0].key, "bulkKey123");

            // Check second bulk dataset
            assert_eq!(args.datasets[1].url, "https://bulk-dataset-2.bin");
            assert_eq!(args.datasets[1].checksum, "0x223checksum");
            assert_eq!(args.datasets[1].filename, "bulk-dataset-2.txt");
            assert_eq!(args.datasets[1].key, "bulkKey223");

            // Check third bulk dataset
            assert_eq!(args.datasets[2].url, "https://bulk-dataset-3.bin");
            assert_eq!(args.datasets[2].checksum, "0x323checksum");
            assert_eq!(args.datasets[2].filename, "bulk-dataset-3.txt");
            assert_eq!(args.datasets[2].key, "bulkKey323");
        });
    }

    #[test]
    fn read_args_succeeds_with_both_dataset_and_bulk_datasets() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.extend(setup_dataset_env_vars());
        env_vars.extend(setup_input_files_env_vars(0));
        env_vars.extend(setup_bulk_dataset_env_vars(2));

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            assert!(result.1.is_empty());
            let args = result.0;

            assert_eq!(args.output_dir, OUTPUT_DIR);
            assert!(args.is_dataset_required);
            assert_eq!(args.iexec_bulk_slice_size, 2);
            assert_eq!(args.datasets.len(), 3); // 1 regular + 2 bulk datasets
            assert_eq!(args.input_files.len(), 0);

            // Check regular dataset (first in list)
            assert_eq!(args.datasets[0].url, DATASET_URL);
            assert_eq!(args.datasets[0].checksum, DATASET_CHECKSUM);
            assert_eq!(args.datasets[0].filename, DATASET_FILENAME);
            assert_eq!(args.datasets[0].key, DATASET_KEY);

            // Check bulk datasets
            assert_eq!(args.datasets[1].url, "https://bulk-dataset-1.bin");
            assert_eq!(args.datasets[2].url, "https://bulk-dataset-2.bin");
        });
    }

    #[test]
    fn read_args_fails_when_invalid_iexec_bulk_slice_size_format() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.insert(IexecBulkSliceSize.name(), "not-a-number".to_string());
        env_vars.extend(setup_input_files_env_vars(0));

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();
            assert!(!result.1.is_empty());
            assert_eq!(
                result.1,
                vec![ReplicateStatusCause::PreComputeFailedUnknownIssue]
            );
        });
    }

    #[test]
    fn read_args_fails_when_bulk_dataset_url_missing() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(0));
        env_vars.extend(setup_bulk_dataset_env_vars(2));
        // Remove one of the bulk dataset URLs
        env_vars.remove(&IexecDatasetUrl(1).name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();
            assert!(!result.1.is_empty());
            assert_eq!(
                result.1,
                vec![ReplicateStatusCause::PreComputeDatasetUrlMissing(
                    "bulk-dataset-1.txt".to_string()
                )]
            );
        });
    }

    #[test]
    fn read_args_fails_when_bulk_dataset_checksum_missing() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(0));
        env_vars.extend(setup_bulk_dataset_env_vars(2));
        // Remove one of the bulk dataset checksums
        env_vars.remove(&IexecDatasetChecksum(2).name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();
            assert!(!result.1.is_empty());
            assert_eq!(
                result.1,
                vec![ReplicateStatusCause::PreComputeDatasetChecksumMissing(
                    "bulk-dataset-2.txt".to_string()
                )]
            );
        });
    }

    #[test]
    fn read_args_fails_when_bulk_dataset_filename_missing() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(0));
        env_vars.extend(setup_bulk_dataset_env_vars(3));
        // Remove one of the bulk dataset filenames
        env_vars.remove(&IexecDatasetFilename(2).name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();
            assert!(!result.1.is_empty());
            assert_eq!(
                result.1,
                vec![ReplicateStatusCause::PreComputeDatasetFilenameMissing(
                    "dataset_2".to_string()
                )]
            );
        });
    }

    #[test]
    fn read_args_fails_when_bulk_dataset_key_missing() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(0));
        env_vars.extend(setup_bulk_dataset_env_vars(2));
        // Remove one of the bulk dataset keys
        env_vars.remove(&IexecDatasetKey(1).name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();
            assert!(!result.1.is_empty());
            assert_eq!(
                result.1,
                vec![ReplicateStatusCause::PreComputeDatasetKeyMissing(
                    "bulk-dataset-1.txt".to_string()
                )]
            );
        });
    }
    // endregion

    // region dataset environment variables
    #[test]
    fn read_args_fails_when_dataset_env_var_missing() {
        let missing_env_var_causes = vec![
            (
                IexecPreComputeOut,
                ReplicateStatusCause::PreComputeOutputPathMissing,
            ),
            (
                IsDatasetRequired,
                ReplicateStatusCause::PreComputeIsDatasetRequiredMissing,
            ),
            (
                IexecInputFilesNumber,
                ReplicateStatusCause::PreComputeInputFilesNumberMissing,
            ),
            (
                IexecDatasetUrl(0),
                ReplicateStatusCause::PreComputeDatasetUrlMissing(DATASET_FILENAME.to_string()),
            ),
            (
                IexecDatasetKey(0),
                ReplicateStatusCause::PreComputeDatasetKeyMissing(DATASET_FILENAME.to_string()),
            ),
            (
                IexecDatasetChecksum(0),
                ReplicateStatusCause::PreComputeDatasetChecksumMissing(
                    DATASET_FILENAME.to_string(),
                ),
            ),
            (
                IexecDatasetFilename(0),
                ReplicateStatusCause::PreComputeDatasetFilenameMissing("dataset_0".to_string()),
            ),
            (
                IexecInputFileUrlPrefix(1),
                ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(1),
            ),
        ];
        for (env_var, error) in missing_env_var_causes {
            test_read_args_fails_with_missing_env_var(env_var, vec![error]);
        }
    }

    fn test_read_args_fails_with_missing_env_var(
        env_var: TeeSessionEnvironmentVariable,
        errors: Vec<ReplicateStatusCause>,
    ) {
        //Set up environment variables
        let mut env_vars = setup_basic_env_vars();
        env_vars.extend(setup_dataset_env_vars());
        env_vars.extend(setup_input_files_env_vars(1));
        env_vars.remove(&env_var.name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();
            assert!(!result.1.is_empty());
            assert_eq!(result.1, errors);
        });
    }
    // endregion

    // region error collection tests
    #[test]
    fn read_args_collects_multiple_errors_when_multiple_env_vars_missing() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.extend(setup_dataset_env_vars());
        env_vars.extend(setup_input_files_env_vars(2));

        // Remove dataset URL and an input file URL
        env_vars.remove(&IexecDatasetUrl(0).name());
        env_vars.remove(&IexecInputFileUrlPrefix(1).name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should collect both errors (dataset stops at URL, input file error also collected)
            assert_eq!(result.1.len(), 2);
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeDatasetUrlMissing(
                        DATASET_FILENAME.to_string()
                    ))
            );
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(1))
            );
        });
    }

    #[test]
    fn read_args_collects_errors_for_partial_bulk_dataset_failures() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(0));
        env_vars.extend(setup_bulk_dataset_env_vars(3));

        // Remove various fields from different bulk datasets
        env_vars.remove(&IexecDatasetUrl(1).name());
        env_vars.remove(&IexecDatasetChecksum(2).name());
        env_vars.remove(&IexecDatasetKey(3).name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should collect all 3 errors
            assert_eq!(result.1.len(), 3);
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeDatasetUrlMissing(
                        "bulk-dataset-1.txt".to_string()
                    ))
            );
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeDatasetChecksumMissing(
                        "bulk-dataset-2.txt".to_string()
                    ))
            );
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeDatasetKeyMissing(
                        "bulk-dataset-3.txt".to_string()
                    ))
            );

            // No datasets should be added since they all had errors
            assert_eq!(result.0.datasets.len(), 0);
        });
    }

    #[test]
    fn read_args_continues_processing_after_dataset_errors() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(0));
        env_vars.extend(setup_bulk_dataset_env_vars(3));

        // Remove only the second dataset's URL
        env_vars.remove(&IexecDatasetUrl(2).name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should have one error for the missing URL
            assert_eq!(result.1.len(), 1);
            assert_eq!(
                result.1[0],
                ReplicateStatusCause::PreComputeDatasetUrlMissing("bulk-dataset-2.txt".to_string())
            );

            // Should successfully load the other two datasets
            assert_eq!(result.0.datasets.len(), 2);
            assert_eq!(result.0.datasets[0].url, "https://bulk-dataset-1.bin");
            assert_eq!(result.0.datasets[1].url, "https://bulk-dataset-3.bin");
        });
    }

    #[test]
    fn read_args_collects_all_missing_input_file_urls() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(5));

        // Remove multiple input file URLs
        env_vars.remove(&IexecInputFileUrlPrefix(2).name());
        env_vars.remove(&IexecInputFileUrlPrefix(4).name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should collect errors for missing URLs
            assert_eq!(result.1.len(), 2);
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(2))
            );
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(4))
            );

            // Should successfully load the other three input files
            assert_eq!(result.0.input_files.len(), 3);
            assert_eq!(result.0.input_files[0], "https://input-1.txt");
            assert_eq!(result.0.input_files[1], "https://input-3.txt");
            assert_eq!(result.0.input_files[2], "https://input-5.txt");
        });
    }

    #[test]
    fn read_args_handles_mixed_errors_across_all_categories() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.extend(setup_dataset_env_vars());
        env_vars.extend(setup_input_files_env_vars(3));
        env_vars.extend(setup_bulk_dataset_env_vars(2));

        // Create errors across different categories
        env_vars.insert(IsDatasetRequired.name(), "invalid-bool".to_string());
        // Since invalid bool defaults to false, dataset at index 0 won't be read
        // So we need to remove fields from bulk datasets (indices 1 and 2)
        env_vars.remove(&IexecDatasetChecksum(1).name()); // Bulk dataset 1 error
        env_vars.remove(&IexecInputFileUrlPrefix(2).name()); // Input file error

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should collect: bool parse error, bulk dataset checksum error, input file error
            assert_eq!(result.1.len(), 3);
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeIsDatasetRequiredMissing)
            );
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeDatasetChecksumMissing(
                        "bulk-dataset-1.txt".to_string()
                    ))
            );
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(2))
            );
        });
    }

    #[test]
    fn read_args_processes_valid_datasets_despite_some_failures() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.extend(setup_dataset_env_vars());
        env_vars.extend(setup_input_files_env_vars(0));
        env_vars.extend(setup_bulk_dataset_env_vars(4));

        // Break datasets at indices 1 and 3
        env_vars.remove(&IexecDatasetUrl(1).name());
        env_vars.remove(&IexecDatasetKey(3).name());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should have 2 errors
            assert_eq!(result.1.len(), 2);

            // Should successfully load datasets at indices 0, 2, and 4
            assert_eq!(result.0.datasets.len(), 3);
            assert_eq!(result.0.datasets[0].url, DATASET_URL);
            assert_eq!(result.0.datasets[1].url, "https://bulk-dataset-2.bin");
            assert_eq!(result.0.datasets[2].url, "https://bulk-dataset-4.bin");
        });
    }

    #[test]
    fn read_args_continues_after_bulk_slice_size_parse_error() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(2));
        env_vars.insert(IexecBulkSliceSize.name(), "invalid-number".to_string());

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should collect the parse error
            assert_eq!(result.1.len(), 1);
            assert_eq!(
                result.1[0],
                ReplicateStatusCause::PreComputeFailedUnknownIssue
            );

            // Should still process input files successfully
            assert_eq!(result.0.input_files.len(), 2);
            assert_eq!(result.0.iexec_bulk_slice_size, 0);
        });
    }

    #[test]
    fn read_args_collects_all_dataset_field_errors_for_single_dataset() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(0));

        // Set up only one bulk dataset but with missing filename (first field checked)
        env_vars.insert(IexecBulkSliceSize.name(), "1".to_string());
        // Intentionally not setting filename - this will cause early exit from loop

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should collect error for missing filename (loop exits early, doesn't check other fields)
            assert_eq!(result.1.len(), 1);
            assert_eq!(
                result.1[0],
                ReplicateStatusCause::PreComputeDatasetFilenameMissing("dataset_1".to_string())
            );

            // No dataset should be added
            assert_eq!(result.0.datasets.len(), 0);
        });
    }

    #[test]
    fn read_args_stops_at_first_dataset_field_error() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.extend(setup_input_files_env_vars(0));

        // Set up bulk dataset with filename but missing URL (second field checked)
        env_vars.insert(IexecBulkSliceSize.name(), "1".to_string());
        env_vars.insert(
            IexecDatasetFilename(1).name(),
            "incomplete-dataset.txt".to_string(),
        );
        // Missing URL, checksum, and key - but should only report URL error

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should only collect error for the first missing field (URL)
            assert_eq!(result.1.len(), 1);
            assert_eq!(
                result.1[0],
                ReplicateStatusCause::PreComputeDatasetUrlMissing(
                    "incomplete-dataset.txt".to_string()
                )
            );

            // No dataset should be added
            assert_eq!(result.0.datasets.len(), 0);
        });
    }

    #[test]
    fn read_args_handles_empty_input_files_list_with_errors() {
        let mut env_vars = setup_basic_env_vars();
        env_vars.insert(IsDatasetRequired.name(), "false".to_string());
        env_vars.insert(IexecInputFilesNumber.name(), "3".to_string());
        // Intentionally not setting any input file URLs

        temp_env::with_vars(to_temp_env_vars(env_vars), || {
            let result = PreComputeArgs::read_args();

            // Should collect errors for all missing input files
            assert_eq!(result.1.len(), 3);
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(1))
            );
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(2))
            );
            assert!(
                result
                    .1
                    .contains(&ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(3))
            );

            // Input files should be empty
            assert_eq!(result.0.input_files.len(), 0);
        });
    }
    // endregion
}
