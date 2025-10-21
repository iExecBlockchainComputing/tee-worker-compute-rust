use serde::{Serializer, ser::SerializeStruct};
use strum_macros::EnumDiscriminants;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error, EnumDiscriminants)]
#[strum_discriminants(derive(serde::Serialize))]
#[strum_discriminants(serde(rename_all = "SCREAMING_SNAKE_CASE"))]
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
    use serde_json::{json, to_value};

    #[test]
    fn error_variant_serialize_correctly() {
        let expected = json!({
            "cause": "POST_COMPUTE_TooLongResultFileName",
            "message": "Result file name too long"
        });
        let error_variant = ReplicateStatusCause::PostComputeTooLongResultFileName;
        assert_eq!(to_value(&error_variant).unwrap(), expected);
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
