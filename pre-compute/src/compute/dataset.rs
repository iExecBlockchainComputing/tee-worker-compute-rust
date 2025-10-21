use crate::compute::errors::ReplicateStatusCause;
use crate::compute::utils::file_utils::download_from_url;
use crate::compute::utils::hash_utils::sha256_from_bytes;
use aes::Aes256;
use base64::{Engine as _, engine::general_purpose};
use cbc::{
    Decryptor,
    cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7},
};
use log::{error, info};
use multiaddr::Multiaddr;
use std::str::FromStr;

type Aes256CbcDec = Decryptor<Aes256>;
const IPFS_GATEWAYS: &[&str] = &[
    "https://ipfs-gateway.v8-bellecour.iex.ec",
    "https://gateway.ipfs.io",
    "https://gateway.pinata.cloud",
];
const AES_KEY_LENGTH: usize = 32;
const AES_IV_LENGTH: usize = 16;

/// Represents a dataset in a Trusted Execution Environment (TEE).
///
/// This structure contains all the information needed to download, verify, and decrypt
/// a single dataset.
#[cfg_attr(test, derive(Debug))]
#[derive(Clone, Default)]
pub struct Dataset {
    pub url: String,
    pub checksum: String,
    pub filename: String,
    pub key: String,
}

impl Dataset {
    pub fn new(url: String, checksum: String, filename: String, key: String) -> Self {
        Dataset {
            url,
            checksum,
            filename,
            key,
        }
    }

    /// Downloads the encrypted dataset file from a URL or IPFS multi-address, and verifies its checksum.
    ///
    /// # Arguments
    ///
    /// * `chain_task_id` - The chain task ID for logging
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` containing the dataset's encrypted content if download and verification succeed.
    /// * `Err(ReplicateStatusCause::PreComputeDatasetDownloadFailed)` if the download fails.
    /// * `Err(ReplicateStatusCause::PreComputeInvalidDatasetChecksum)` if checksum validation fails.
    pub fn download_encrypted_dataset(
        &self,
        chain_task_id: &str,
    ) -> Result<Vec<u8>, ReplicateStatusCause> {
        info!(
            "Downloading encrypted dataset file [chainTaskId:{chain_task_id}, url:{}]",
            self.url
        );

        let encrypted_content = if is_multi_address(&self.url) {
            IPFS_GATEWAYS.iter().find_map(|gateway| {
                let full_url = format!("{gateway}{}", self.url);
                info!("Attempting to download dataset from {full_url}");

                if let Some(content) = download_from_url(&full_url) {
                    info!("Successfully downloaded from {full_url}");
                    Some(content)
                } else {
                    error!("Failed to download from {full_url}");
                    None
                }
            })
        } else {
            download_from_url(&self.url)
        }
        .ok_or(ReplicateStatusCause::PreComputeDatasetDownloadFailed(
            self.filename.clone(),
        ))?;

        info!("Checking encrypted dataset checksum [chainTaskId:{chain_task_id}]");
        let actual_checksum = sha256_from_bytes(&encrypted_content);

        if actual_checksum != self.checksum {
            error!(
                "Invalid dataset checksum [chainTaskId:{chain_task_id}, expected:{}, actual:{actual_checksum}]",
                self.checksum
            );
            return Err(ReplicateStatusCause::PreComputeInvalidDatasetChecksum(
                self.filename.clone(),
            ));
        }

        info!("Dataset downloaded and verified successfully.");
        Ok(encrypted_content)
    }

    /// Decrypts the provided encrypted dataset bytes using AES-CBC.
    ///
    /// The first 16 bytes of `encrypted_content` are treated as the IV.
    /// The rest is the ciphertext. The decryption key is decoded from a Base64 string.
    ///
    /// # Arguments
    ///
    /// * `encrypted_content` - Full encrypted dataset, including the IV prefix.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` containing the plaintext dataset if decryption succeeds.
    /// * `Err(ReplicateStatusCause::PreComputeDatasetDecryptionFailed)` if the key is missing, decoding fails, or decryption fails.
    pub fn decrypt_dataset(
        &self,
        encrypted_content: &[u8],
    ) -> Result<Vec<u8>, ReplicateStatusCause> {
        let key = general_purpose::STANDARD.decode(&self.key).map_err(|_| {
            ReplicateStatusCause::PreComputeDatasetDecryptionFailed(self.filename.clone())
        })?;

        if encrypted_content.len() < AES_IV_LENGTH || key.len() != AES_KEY_LENGTH {
            return Err(ReplicateStatusCause::PreComputeDatasetDecryptionFailed(
                self.filename.clone(),
            ));
        }

        let key_slice = &key[..AES_KEY_LENGTH];
        let iv_slice = &encrypted_content[..AES_IV_LENGTH];
        let ciphertext = &encrypted_content[AES_IV_LENGTH..];

        Aes256CbcDec::new(key_slice.into(), iv_slice.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|_| {
                ReplicateStatusCause::PreComputeDatasetDecryptionFailed(self.filename.clone())
            })
    }
}

fn is_multi_address(uri: &str) -> bool {
    !uri.trim().is_empty() && Multiaddr::from_str(uri).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAIN_TASK_ID: &str = "0x123456789abcdef";
    const DATASET_CHECKSUM: &str =
        "0x02a12ef127dcfbdb294a090c8f0b69a0ca30b7940fc36cabf971f488efd374d7";
    const ENCRYPTED_DATASET_KEY: &str = "ubA6H9emVPJT91/flYAmnKHC0phSV3cfuqsLxQfgow0=";
    const HTTP_DATASET_URL: &str = "https://raw.githubusercontent.com/iExecBlockchainComputing/tee-worker-pre-compute-rust/main/src/tests_resources/encrypted-data.bin";
    const PLAIN_DATA_FILE: &str = "0xDatasetAddress";
    const IPFS_DATASET_URL: &str = "/ipfs/QmUVhChbLFiuzNK1g2GsWyWEiad7SXPqARnWzGumgziwEp";

    fn get_test_dataset() -> Dataset {
        Dataset::new(
            HTTP_DATASET_URL.to_string(),
            DATASET_CHECKSUM.to_string(),
            PLAIN_DATA_FILE.to_string(),
            ENCRYPTED_DATASET_KEY.to_string(),
        )
    }

    // region download_encrypted_dataset
    #[test]
    fn download_encrypted_dataset_success() {
        let dataset = get_test_dataset();
        let actual_content = dataset.download_encrypted_dataset(CHAIN_TASK_ID);
        assert!(actual_content.is_ok());
    }

    #[test]
    fn download_encrypted_dataset_failure_with_invalid_dataset_url() {
        let mut dataset = get_test_dataset();
        dataset.url = "http://bad-url".to_string();
        let actual_content = dataset.download_encrypted_dataset(CHAIN_TASK_ID);
        assert_eq!(
            actual_content,
            Err(ReplicateStatusCause::PreComputeDatasetDownloadFailed(
                PLAIN_DATA_FILE.to_string()
            ))
        );
    }

    #[test]
    fn download_encrypted_dataset_success_with_valid_iexec_gateway() {
        let mut dataset = get_test_dataset();
        dataset.url = IPFS_DATASET_URL.to_string();
        dataset.checksum =
            "0x323b1637c7999942fbebfe5d42fe15dbfe93737577663afa0181938d7ad4a2ac".to_string();
        let actual_content = dataset.download_encrypted_dataset(CHAIN_TASK_ID);
        let expected_content = Ok("hello world !\n".as_bytes().to_vec());
        assert_eq!(actual_content, expected_content);
    }

    #[test]
    fn download_encrypted_dataset_failure_with_invalid_gateway() {
        let mut dataset = get_test_dataset();
        dataset.url = "/ipfs/INVALID_IPFS_DATASET_URL".to_string();
        let actual_content = dataset.download_encrypted_dataset(CHAIN_TASK_ID);
        let expected_content = Err(ReplicateStatusCause::PreComputeDatasetDownloadFailed(
            PLAIN_DATA_FILE.to_string(),
        ));
        assert_eq!(actual_content, expected_content);
    }

    #[test]
    fn download_encrypted_dataset_failure_with_invalid_dataset_checksum() {
        let mut dataset = get_test_dataset();
        dataset.checksum = "invalid_dataset_checksum".to_string();
        let actual_content = dataset.download_encrypted_dataset(CHAIN_TASK_ID);
        let expected_content = Err(ReplicateStatusCause::PreComputeInvalidDatasetChecksum(
            PLAIN_DATA_FILE.to_string(),
        ));
        assert_eq!(actual_content, expected_content);
    }
    // endregion

    // region decrypt_dataset
    #[test]
    fn decrypt_dataset_success_with_valid_dataset() {
        let dataset = get_test_dataset();

        let encrypted_data = dataset.download_encrypted_dataset(CHAIN_TASK_ID).unwrap();
        let expected_plain_data = Ok("Some very useful data.".as_bytes().to_vec());
        let actual_plain_data = dataset.decrypt_dataset(&encrypted_data);

        assert_eq!(actual_plain_data, expected_plain_data);
    }

    #[test]
    fn decrypt_dataset_failure_with_bad_key() {
        let mut dataset = get_test_dataset();
        dataset.key = "bad_key".to_string();
        let encrypted_data = dataset.download_encrypted_dataset(CHAIN_TASK_ID).unwrap();
        let actual_plain_data = dataset.decrypt_dataset(&encrypted_data);

        assert_eq!(
            actual_plain_data,
            Err(ReplicateStatusCause::PreComputeDatasetDecryptionFailed(
                PLAIN_DATA_FILE.to_string()
            ))
        );
    }
    // endregion
}
