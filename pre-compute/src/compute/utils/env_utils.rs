use crate::compute::errors::ReplicateStatusCause;
use std::env;

pub enum TeeSessionEnvironmentVariable {
    IexecBulkSliceSize,
    IexecDatasetChecksum(usize),
    IexecDatasetFilename(usize),
    IexecDatasetKey(usize),
    IexecDatasetUrl(usize),
    IexecInputFileUrlPrefix(usize),
    IexecInputFilesNumber,
    IexecPreComputeOut,
    IexecTaskId,
    IsDatasetRequired,
    SignTeeChallengePrivateKey,
    SignWorkerAddress,
    WorkerHostEnvVar,
}

impl TeeSessionEnvironmentVariable {
    pub fn name(&self) -> String {
        match self {
            Self::IexecBulkSliceSize => "IEXEC_BULK_SLICE_SIZE".to_string(),

            Self::IexecDatasetChecksum(0) => "IEXEC_DATASET_CHECKSUM".to_string(),
            Self::IexecDatasetChecksum(index) => {
                format!("IEXEC_DATASET_{index}_CHECKSUM")
            }

            Self::IexecDatasetFilename(0) => "IEXEC_DATASET_FILENAME".to_string(),
            Self::IexecDatasetFilename(index) => {
                format!("IEXEC_DATASET_{index}_FILENAME")
            }

            Self::IexecDatasetKey(0) => "IEXEC_DATASET_KEY".to_string(),
            Self::IexecDatasetKey(index) => {
                format!("IEXEC_DATASET_{index}_KEY")
            }

            Self::IexecDatasetUrl(0) => "IEXEC_DATASET_URL".to_string(),
            Self::IexecDatasetUrl(index) => {
                format!("IEXEC_DATASET_{index}_URL")
            }

            Self::IexecInputFileUrlPrefix(index) => {
                format!("IEXEC_INPUT_FILE_URL_{index}")
            }
            Self::IexecInputFilesNumber => "IEXEC_INPUT_FILES_NUMBER".to_string(),
            Self::IexecPreComputeOut => "IEXEC_PRE_COMPUTE_OUT".to_string(),
            Self::IexecTaskId => "IEXEC_TASK_ID".to_string(),
            Self::IsDatasetRequired => "IS_DATASET_REQUIRED".to_string(),
            Self::SignTeeChallengePrivateKey => "SIGN_TEE_CHALLENGE_PRIVATE_KEY".to_string(),
            Self::SignWorkerAddress => "SIGN_WORKER_ADDRESS".to_string(),
            Self::WorkerHostEnvVar => "WORKER_HOST_ENV_VAR".to_string(),
        }
    }
}

pub fn get_env_var_or_error(
    env_var: TeeSessionEnvironmentVariable,
    status_cause_if_missing: ReplicateStatusCause,
) -> Result<String, ReplicateStatusCause> {
    match env::var(env_var.name()) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => Err(status_cause_if_missing),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    #[test]
    fn name_succeeds_when_simple_environment_variable_names() {
        assert_eq!(
            TeeSessionEnvironmentVariable::IexecBulkSliceSize.name(),
            "IEXEC_BULK_SLICE_SIZE"
        );

        assert_eq!(
            TeeSessionEnvironmentVariable::IexecInputFilesNumber.name(),
            "IEXEC_INPUT_FILES_NUMBER"
        );
        assert_eq!(
            TeeSessionEnvironmentVariable::IexecPreComputeOut.name(),
            "IEXEC_PRE_COMPUTE_OUT"
        );
        assert_eq!(
            TeeSessionEnvironmentVariable::IexecTaskId.name(),
            "IEXEC_TASK_ID"
        );
        assert_eq!(
            TeeSessionEnvironmentVariable::IsDatasetRequired.name(),
            "IS_DATASET_REQUIRED"
        );
        assert_eq!(
            TeeSessionEnvironmentVariable::SignTeeChallengePrivateKey.name(),
            "SIGN_TEE_CHALLENGE_PRIVATE_KEY"
        );
        assert_eq!(
            TeeSessionEnvironmentVariable::SignWorkerAddress.name(),
            "SIGN_WORKER_ADDRESS"
        );
        assert_eq!(
            TeeSessionEnvironmentVariable::WorkerHostEnvVar.name(),
            "WORKER_HOST_ENV_VAR"
        );
    }

    #[test]
    fn name_succeeds_when_indexed_environment_variable_names() {
        // Test IexecDatasetChecksum
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetChecksum(0);
        assert_eq!(env_var.name(), "IEXEC_DATASET_CHECKSUM");
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetChecksum(1);
        assert_eq!(env_var.name(), "IEXEC_DATASET_1_CHECKSUM");
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetChecksum(10);
        assert_eq!(env_var.name(), "IEXEC_DATASET_42_CHECKSUM");

        // Test IexecDatasetFilename
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetFilename(0);
        assert_eq!(env_var.name(), "IEXEC_DATASET_FILENAME");
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetFilename(1);
        assert_eq!(env_var.name(), "IEXEC_DATASET_1_FILENAME");
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetFilename(5);
        assert_eq!(env_var.name(), "IEXEC_DATASET_10_FILENAME");

        // Test IexecDatasetKey
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetKey(0);
        assert_eq!(env_var.name(), "IEXEC_DATASET_KEY");
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetKey(1);
        assert_eq!(env_var.name(), "IEXEC_DATASET_1_KEY");
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetKey(20);
        assert_eq!(env_var.name(), "IEXEC_DATASET_5_KEY");

        // Test IexecDatasetUrl
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetUrl(0);
        assert_eq!(env_var.name(), "IEXEC_DATASET_URL");
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetUrl(1);
        assert_eq!(env_var.name(), "IEXEC_DATASET_1_URL");
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetUrl(99);
        assert_eq!(env_var.name(), "IEXEC_DATASET_99_URL");

        // Test IexecInputFileUrlPrefix
        let env_var = TeeSessionEnvironmentVariable::IexecInputFileUrlPrefix(0);
        assert_eq!(env_var.name(), "IEXEC_INPUT_FILE_URL_0");
        let env_var = TeeSessionEnvironmentVariable::IexecInputFileUrlPrefix(1);
        assert_eq!(env_var.name(), "IEXEC_INPUT_FILE_URL_1");
        let env_var = TeeSessionEnvironmentVariable::IexecInputFileUrlPrefix(123);
        assert_eq!(env_var.name(), "IEXEC_INPUT_FILE_URL_123");
    }

    #[test]
    fn get_env_var_or_error_success() {
        let env_var = TeeSessionEnvironmentVariable::IexecTaskId;
        let status_cause = ReplicateStatusCause::PreComputeTaskIdMissing;

        temp_env::with_var("IEXEC_TASK_ID", Some("test-task-id-123"), || {
            let result = get_env_var_or_error(env_var, status_cause.clone());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "test-task-id-123");
        });
    }

    #[test]
    fn get_env_var_or_error_fails_when_missing_variable() {
        let env_var = TeeSessionEnvironmentVariable::IexecTaskId;
        let status_cause = ReplicateStatusCause::PreComputeTaskIdMissing;

        temp_env::with_var_unset("IEXEC_TASK_ID", || {
            let result = get_env_var_or_error(env_var, status_cause.clone());
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), status_cause);
        });
    }

    #[test]
    fn get_env_var_or_error_succeeds_when_empty_variable() {
        let env_var = TeeSessionEnvironmentVariable::IexecTaskId;
        let status_cause = ReplicateStatusCause::PreComputeTaskIdMissing;

        temp_env::with_var("IEXEC_TASK_ID", Some(""), || {
            let result = get_env_var_or_error(env_var, status_cause.clone());
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), status_cause);
        });
    }

    #[test]
    fn get_env_var_or_error_succeeds_when_whitespace_only_variable() {
        let env_var = TeeSessionEnvironmentVariable::IexecTaskId;
        let status_cause = ReplicateStatusCause::PreComputeTaskIdMissing;

        temp_env::with_var("IEXEC_TASK_ID", Some("   "), || {
            let result = get_env_var_or_error(env_var, status_cause.clone());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "   ");
        });
    }

    #[test]
    fn get_env_var_or_error_succeeds_when_indexed_variables() {
        let env_var = TeeSessionEnvironmentVariable::IexecDatasetChecksum(1);
        let status_cause = ReplicateStatusCause::PreComputeDatasetChecksumMissing;

        temp_env::with_var("IEXEC_DATASET_1_CHECKSUM", Some("abc123def456"), || {
            let result = get_env_var_or_error(env_var, status_cause.clone());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "abc123def456");
        });
    }
}
