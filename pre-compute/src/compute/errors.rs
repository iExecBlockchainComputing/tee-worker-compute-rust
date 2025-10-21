use serde::{Serializer, ser::SerializeStruct};
use strum_macros::EnumDiscriminants;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error, EnumDiscriminants)]
#[strum_discriminants(derive(serde::Serialize))]
#[strum_discriminants(serde(rename_all = "SCREAMING_SNAKE_CASE"))]
#[allow(clippy::enum_variant_names)]
pub enum ReplicateStatusCause {
    #[error("input file URL {0} is missing")]
    PreComputeAtLeastOneInputFileUrlMissing(usize),
    #[error("Dataset checksum related environment variable is missing for dataset {0}")]
    PreComputeDatasetChecksumMissing(String),
    #[error("Failed to decrypt dataset {0}")]
    PreComputeDatasetDecryptionFailed(String),
    #[error("Failed to download encrypted dataset file for dataset {0}")]
    PreComputeDatasetDownloadFailed(String),
    #[error("Dataset filename related environment variable is missing for dataset {0}")]
    PreComputeDatasetFilenameMissing(String),
    #[error("Dataset key related environment variable is missing for dataset {0}")]
    PreComputeDatasetKeyMissing(String),
    #[error("Dataset URL related environment variable is missing for dataset {0}")]
    PreComputeDatasetUrlMissing(String),
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
    PreComputeInvalidDatasetChecksum(String),
    #[error("Output folder related environment variable is missing")]
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
        state.serialize_field("cause", &ReplicateStatusCauseDiscriminants::from(self))?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    const DATASET_FILENAME: &str = "0xDatasetAddress";

    #[test]
    fn serialize_produces_correct_json_when_error_has_dataset_filename() {
        let cause = ReplicateStatusCause::PreComputeDatasetUrlMissing(DATASET_FILENAME.to_string());
        let serialized = to_string(&cause).unwrap();
        assert_eq!(
            serialized,
            r#"{"cause":"PRE_COMPUTE_DATASET_URL_MISSING","message":"Dataset URL related environment variable is missing for dataset 0xDatasetAddress"}"#
        );
    }

    #[test]
    fn serialize_produces_correct_json_when_error_has_no_index() {
        let cause = ReplicateStatusCause::PreComputeInvalidTeeSignature;
        let serialized = to_string(&cause).unwrap();
        assert_eq!(
            serialized,
            r#"{"cause":"PRE_COMPUTE_INVALID_TEE_SIGNATURE","message":"Invalid TEE signature"}"#
        );
    }

    #[test]
    fn serialize_produces_correct_json_when_multiple_dataset_errors_with_filenames() {
        let test_cases = vec![
            (
                ReplicateStatusCause::PreComputeAtLeastOneInputFileUrlMissing(1),
                r#"{"cause":"PRE_COMPUTE_AT_LEAST_ONE_INPUT_FILE_URL_MISSING","message":"input file URL 1 is missing"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeDatasetChecksumMissing(
                    DATASET_FILENAME.to_string(),
                ),
                r#"{"cause":"PRE_COMPUTE_DATASET_CHECKSUM_MISSING","message":"Dataset checksum related environment variable is missing for dataset 0xDatasetAddress"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeDatasetDecryptionFailed(
                    DATASET_FILENAME.to_string(),
                ),
                r#"{"cause":"PRE_COMPUTE_DATASET_DECRYPTION_FAILED","message":"Failed to decrypt dataset 0xDatasetAddress"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeDatasetDownloadFailed(DATASET_FILENAME.to_string()),
                r#"{"cause":"PRE_COMPUTE_DATASET_DOWNLOAD_FAILED","message":"Failed to download encrypted dataset file for dataset 0xDatasetAddress"}"#,
            ),
            (
                ReplicateStatusCause::PreComputeInvalidDatasetChecksum(
                    DATASET_FILENAME.to_string(),
                ),
                r#"{"cause":"PRE_COMPUTE_INVALID_DATASET_CHECKSUM","message":"Invalid dataset checksum for dataset 0xDatasetAddress"}"#,
            ),
        ];

        for (cause, expected) in test_cases {
            let serialized = to_string(&cause).unwrap();
            assert_eq!(serialized, expected);
        }
    }

    #[test]
    fn serialize_produces_correct_json_when_vector_of_multiple_errors() {
        let causes = vec![
            ReplicateStatusCause::PreComputeDatasetUrlMissing(DATASET_FILENAME.to_string()),
            ReplicateStatusCause::PreComputeInvalidDatasetChecksum("0xAnotherDataset".to_string()),
        ];

        let serialized = to_string(&causes).unwrap();
        let expected = r#"[{"cause":"PRE_COMPUTE_DATASET_URL_MISSING","message":"Dataset URL related environment variable is missing for dataset 0xDatasetAddress"},{"cause":"PRE_COMPUTE_INVALID_DATASET_CHECKSUM","message":"Invalid dataset checksum for dataset 0xAnotherDataset"}]"#;
        assert_eq!(serialized, expected);
    }
}
