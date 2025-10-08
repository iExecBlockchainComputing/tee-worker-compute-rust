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
