use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum ReplicateStatusCause {
    #[error("computed.json file missing")]
    PostComputeComputedFileNotFound,
    #[error("Failed to upload to Dropbox")]
    PostComputeDropboxUploadFailed,
    #[error("Encryption stage failed")]
    PostComputeEncryptionFailed,
    #[error("Encryption public key related environment variable is missing")]
    PostComputeEncryptionPublicKeyMissing,
    #[error("Unexpected error occurred")]
    PostComputeFailedUnknownIssue,
    #[error("Invalid TEE signature")]
    PostComputeInvalidTeeSignature,
    #[error("Failed to upload to IPFS")]
    PostComputeIpfsUploadFailed,
    #[error("Encryption public key is malformed")]
    PostComputeMalformedEncryptionPublicKey,
    #[error("Failed to zip result folder")]
    PostComputeOutFolderZipFailed,
    #[error("Empty resultDigest")]
    PostComputeResultDigestComputationFailed,
    #[error("Result file not found")]
    PostComputeResultFileNotFound,
    #[error("Failed to send computed file")]
    PostComputeSendComputedFileFailed,
    #[error("Storage token related environment variable is missing")]
    PostComputeStorageTokenMissing,
    #[error("Task ID related environment variable is missing")]
    PostComputeTaskIdMissing,
    #[error("Tee challenge private key related environment variable is missing")]
    PostComputeTeeChallengePrivateKeyMissing,
    #[error("Result file name too long")]
    PostComputeTooLongResultFileName,
    #[error("Worker address related environment variable is missing")]
    PostComputeWorkerAddressMissing,
}

impl ReplicateStatusCause {
    fn to_screaming_snake_case(&self) -> String {
        let debug_str = format!("{:?}", self);
        let mut result = String::new();
        let mut prev_was_lowercase = false;

        for c in debug_str.chars() {
            if c.is_uppercase() && !result.is_empty() && prev_was_lowercase {
                result.push('_');
            }
            result.push(c.to_ascii_uppercase());
            prev_was_lowercase = c.is_lowercase();
        }

        result
    }
}

#[derive(Debug, Serialize)]
pub struct WorkflowError {
    pub cause: String,
    pub message: String,
}

impl From<&ReplicateStatusCause> for WorkflowError {
    fn from(cause: &ReplicateStatusCause) -> Self {
        WorkflowError {
            cause: cause.to_screaming_snake_case(),
            message: cause.to_string(),
        }
    }
}

impl Serialize for ReplicateStatusCause {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        WorkflowError::from(self).serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::{json, to_value};

    #[rstest]
    #[case(
        ReplicateStatusCause::PostComputeComputedFileNotFound,
        "POST_COMPUTE_COMPUTED_FILE_NOT_FOUND",
        "computed.json file missing"
    )]
    #[case(
        ReplicateStatusCause::PostComputeDropboxUploadFailed,
        "POST_COMPUTE_DROPBOX_UPLOAD_FAILED",
        "Failed to upload to Dropbox"
    )]
    #[case(
        ReplicateStatusCause::PostComputeEncryptionFailed,
        "POST_COMPUTE_ENCRYPTION_FAILED",
        "Encryption stage failed"
    )]
    #[case(
        ReplicateStatusCause::PostComputeEncryptionPublicKeyMissing,
        "POST_COMPUTE_ENCRYPTION_PUBLIC_KEY_MISSING",
        "Encryption public key related environment variable is missing"
    )]
    #[case(
        ReplicateStatusCause::PostComputeFailedUnknownIssue,
        "POST_COMPUTE_FAILED_UNKNOWN_ISSUE",
        "Unexpected error occurred"
    )]
    #[case(
        ReplicateStatusCause::PostComputeInvalidTeeSignature,
        "POST_COMPUTE_INVALID_TEE_SIGNATURE",
        "Invalid TEE signature"
    )]
    #[case(
        ReplicateStatusCause::PostComputeIpfsUploadFailed,
        "POST_COMPUTE_IPFS_UPLOAD_FAILED",
        "Failed to upload to IPFS"
    )]
    #[case(
        ReplicateStatusCause::PostComputeMalformedEncryptionPublicKey,
        "POST_COMPUTE_MALFORMED_ENCRYPTION_PUBLIC_KEY",
        "Encryption public key is malformed"
    )]
    #[case(
        ReplicateStatusCause::PostComputeOutFolderZipFailed,
        "POST_COMPUTE_OUT_FOLDER_ZIP_FAILED",
        "Failed to zip result folder"
    )]
    #[case(
        ReplicateStatusCause::PostComputeResultDigestComputationFailed,
        "POST_COMPUTE_RESULT_DIGEST_COMPUTATION_FAILED",
        "Empty resultDigest"
    )]
    #[case(
        ReplicateStatusCause::PostComputeResultFileNotFound,
        "POST_COMPUTE_RESULT_FILE_NOT_FOUND",
        "Result file not found"
    )]
    #[case(
        ReplicateStatusCause::PostComputeSendComputedFileFailed,
        "POST_COMPUTE_SEND_COMPUTED_FILE_FAILED",
        "Failed to send computed file"
    )]
    #[case(
        ReplicateStatusCause::PostComputeStorageTokenMissing,
        "POST_COMPUTE_STORAGE_TOKEN_MISSING",
        "Storage token related environment variable is missing"
    )]
    #[case(
        ReplicateStatusCause::PostComputeTaskIdMissing,
        "POST_COMPUTE_TASK_ID_MISSING",
        "Task ID related environment variable is missing"
    )]
    #[case(
        ReplicateStatusCause::PostComputeTeeChallengePrivateKeyMissing,
        "POST_COMPUTE_TEE_CHALLENGE_PRIVATE_KEY_MISSING",
        "Tee challenge private key related environment variable is missing"
    )]
    #[case(
        ReplicateStatusCause::PostComputeTooLongResultFileName,
        "POST_COMPUTE_TOO_LONG_RESULT_FILE_NAME",
        "Result file name too long"
    )]
    #[case(
        ReplicateStatusCause::PostComputeWorkerAddressMissing,
        "POST_COMPUTE_WORKER_ADDRESS_MISSING",
        "Worker address related environment variable is missing"
    )]
    fn error_variant_serializes_with_correct_cause_and_message(
        #[case] error: ReplicateStatusCause,
        #[case] expected_cause: &str,
        #[case] expected_message: &str,
    ) {
        let serialized = to_value(&error).unwrap();
        assert_eq!(
            serialized,
            json!({
                "cause": expected_cause,
                "message": expected_message
            })
        );
    }

    #[test]
    fn error_list_serializes_as_json_array() {
        let errors = vec![
            ReplicateStatusCause::PostComputeComputedFileNotFound,
            ReplicateStatusCause::PostComputeInvalidTeeSignature,
            ReplicateStatusCause::PostComputeTaskIdMissing,
        ];
        let serialized = to_value(&errors).unwrap();
        let expected = json!([
            {
                "cause": "POST_COMPUTE_COMPUTED_FILE_NOT_FOUND",
                "message": "computed.json file missing"
            },
            {
                "cause": "POST_COMPUTE_INVALID_TEE_SIGNATURE",
                "message": "Invalid TEE signature"
            },
            {
                "cause": "POST_COMPUTE_TASK_ID_MISSING",
                "message": "Task ID related environment variable is missing"
            }
        ]);
        assert_eq!(serialized, expected);
    }

    #[test]
    fn empty_error_list_serializes_as_empty_json_array() {
        let errors: Vec<ReplicateStatusCause> = vec![];
        let serialized = to_value(&errors).unwrap();
        assert_eq!(serialized, json!([]));
    }
}
