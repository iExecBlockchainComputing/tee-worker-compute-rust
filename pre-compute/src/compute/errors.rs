use serde::Serializer;
use serde::ser::SerializeStruct;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error)]
#[allow(clippy::enum_variant_names)]
pub enum ReplicateStatusCause {
    #[error("input file URL {0} is missing")]
    PreComputeAtLeastOneInputFileUrlMissing(usize),
    #[error("Dataset checksum related environment variable is missing for dataset {0}")]
    PreComputeDatasetChecksumMissing(usize),
    #[error("Failed to decrypt dataset {0}")]
    PreComputeDatasetDecryptionFailed(usize),
    #[error("Failed to download encrypted dataset file for dataset {0}")]
    PreComputeDatasetDownloadFailed(usize),
    #[error("Dataset filename related environment variable is missing for dataset {0}")]
    PreComputeDatasetFilenameMissing(usize),
    #[error("Dataset key related environment variable is missing for dataset {0}")]
    PreComputeDatasetKeyMissing(usize),
    #[error("Dataset URL related environment variable is missing for dataset {0}")]
    PreComputeDatasetUrlMissing(usize),
    #[error("Unexpected error occurred")]
    PreComputeFailedUnknownIssue,
    #[error("Invalid TEE signature")]
    PreComputeInvalidTeeSignature,
    #[error("IS_DATASET_REQUIRED environment variable is missing")]
    PreComputeIsDatasetRequiredMissing,
    #[error("Input files download failed")]
    PreComputeInputFileDownloadFailed,
    #[error("Input files number related environment variable is missing")]
    PreComputeInputFilesNumberMissing,
    #[error("Invalid dataset checksum for dataset {0}")]
    PreComputeInvalidDatasetChecksum(usize),
    #[error("Input files number related environment variable is missing")]
    PreComputeOutputFolderNotFound,
    #[error("Output path related environment variable is missing")]
    PreComputeOutputPathMissing,
    #[error("Failed to write plain dataset file")]
    PreComputeSavingPlainDatasetFailed,
    #[error("Task ID related environment variable is missing")]
    PreComputeTaskIdMissing,
    #[error("TEE challenge private key related environment variable is missing")]
    PreComputeTeeChallengePrivateKeyMissing,
    #[error("Worker address related environment variable is missing")]
    PreComputeWorkerAddressMissing,
}

impl serde::Serialize for ReplicateStatusCause {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ReplicateStatusCause", 2)?;

        let cause_name = match self {
            ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(_) => {
                "PRE_COMPUTE_AT_LEAST_ONE_INPUT_FILE_URL_MISSING"
            }
            ReplicateStatusCause::PreComputeDatasetChecksumMissing(_) => {
                "PRE_COMPUTE_DATASET_CHECKSUM_MISSING"
            }
            ReplicateStatusCause::PreComputeDatasetDecryptionFailed(_) => {
                "PRE_COMPUTE_DATASET_DECRYPTION_FAILED"
            }
            ReplicateStatusCause::PreComputeDatasetDownloadFailed(_) => {
                "PRE_COMPUTE_DATASET_DOWNLOAD_FAILED"
            }
            ReplicateStatusCause::PreComputeDatasetFilenameMissing(_) => {
                "PRE_COMPUTE_DATASET_FILENAME_MISSING"
            }
            ReplicateStatusCause::PreComputeDatasetKeyMissing(_) => {
                "PRE_COMPUTE_DATASET_KEY_MISSING"
            }
            ReplicateStatusCause::PreComputeDatasetUrlMissing(_) => {
                "PRE_COMPUTE_DATASET_URL_MISSING"
            }
            ReplicateStatusCause::PreComputeFailedUnknownIssue => {
                "PRE_COMPUTE_FAILED_UNKNOWN_ISSUE"
            }
            ReplicateStatusCause::PreComputeInvalidTeeSignature => {
                "PRE_COMPUTE_INVALID_TEE_SIGNATURE"
            }
            ReplicateStatusCause::PreComputeIsDatasetRequiredMissing => {
                "PRE_COMPUTE_IS_DATASET_REQUIRED_MISSING"
            }
            ReplicateStatusCause::PreComputeInputFileDownloadFailed => {
                "PRE_COMPUTE_INPUT_FILE_DOWNLOAD_FAILED"
            }
            ReplicateStatusCause::PreComputeInputFilesNumberMissing => {
                "PRE_COMPUTE_INPUT_FILES_NUMBER_MISSING"
            }
            ReplicateStatusCause::PreComputeInvalidDatasetChecksum(_) => {
                "PRE_COMPUTE_INVALID_DATASET_CHECKSUM"
            }
            ReplicateStatusCause::PreComputeOutputFolderNotFound => {
                "PRE_COMPUTE_OUTPUT_FOLDER_NOT_FOUND"
            }
            ReplicateStatusCause::PreComputeOutputPathMissing => "PRE_COMPUTE_OUTPUT_PATH_MISSING",
            ReplicateStatusCause::PreComputeSavingPlainDatasetFailed => {
                "PRE_COMPUTE_SAVING_PLAIN_DATASET_FAILED"
            }
            ReplicateStatusCause::PreComputeTaskIdMissing => "PRE_COMPUTE_TASK_ID_MISSING",
            ReplicateStatusCause::PreComputeTeeChallengePrivateKeyMissing => {
                "PRE_COMPUTE_TEE_CHALLENGE_PRIVATE_KEY_MISSING"
            }
            ReplicateStatusCause::PreComputeWorkerAddressMissing => {
                "PRE_COMPUTE_WORKER_ADDRESS_MISSING"
            }
        };

        state.serialize_field("cause", cause_name)?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn test_serialize_dataset_error_with_index() {
        let cause = ReplicateStatusCause::PreComputeDatasetUrlMissing(2);
        let serialized = to_string(&cause).unwrap();
        assert_eq!(
            serialized,
            r#"{"cause":"PRE_COMPUTE_DATASET_URL_MISSING","message":"Dataset URL related environment variable is missing for dataset 2"}"#
        );
    }

    #[test]
    fn test_serialize_non_dataset_error() {
        let cause = ReplicateStatusCause::PreComputeInvalidTeeSignature;
        let serialized = to_string(&cause).unwrap();
        assert_eq!(
            serialized,
            r#"{"cause":"PRE_COMPUTE_INVALID_TEE_SIGNATURE","message":"Invalid TEE signature"}"#
        );
    }

    #[test]
    fn test_serialize_all_dataset_errors() {
        let test_cases = vec![
            (
                ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(1),
                r#"{"cause":"PRE_COMPUTE_AT_LEAST_ONE_INPUT_FILE_URL_MISSING","message":"input file URL 1 is missing"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeDatasetChecksumMissing(3),
                r#"{"cause":"PRE_COMPUTE_DATASET_CHECKSUM_MISSING","message":"Dataset checksum related environment variable is missing for dataset 3"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeDatasetDecryptionFailed(0),
                r#"{"cause":"PRE_COMPUTE_DATASET_DECRYPTION_FAILED","message":"Failed to decrypt dataset 0"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeDatasetDownloadFailed(5),
                r#"{"cause":"PRE_COMPUTE_DATASET_DOWNLOAD_FAILED","message":"Failed to download encrypted dataset file for dataset 5"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeInvalidDatasetChecksum(2),
                r#"{"cause":"PRE_COMPUTE_INVALID_DATASET_CHECKSUM","message":"Invalid dataset checksum for dataset 2"}"#,
            ),
        ];

        for (cause, expected) in test_cases {
            let serialized = to_string(&cause).unwrap();
            assert_eq!(serialized, expected);
        }
    }

    #[test]
    fn test_serialize_vec_of_errors() {
        let causes = vec![
            ReplicateStatusCause::PreComputeDatasetUrlMissing(5),
            ReplicateStatusCause::PreComputeInvalidDatasetChecksum(99),
        ];

        let serialized = to_string(&causes).unwrap();
        let expected = r#"[{"cause":"PRE_COMPUTE_DATASET_URL_MISSING","message":"Dataset URL related environment variable is missing for dataset 5"},{"cause":"PRE_COMPUTE_INVALID_DATASET_CHECKSUM","message":"Invalid dataset checksum for dataset 99"}]"#;
        assert_eq!(serialized, expected);
    }
}
