# Changelog

## [0.5.0](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/compare/tee-worker-pre-compute-v0.4.1...tee-worker-pre-compute-v0.5.0) (2026-09-03)


### Features

* upgrade to Rust 1.98.0 ([#39](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/39)) ([96b3208](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/96b32085b57a2273a832c56514a48fa2557ad8ca))

## [0.4.1](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/compare/tee-worker-pre-compute-v0.4.0...tee-worker-pre-compute-v0.4.1) (2026-06-04)


### Bug Fixes

* **pre-compute:** update IPFS internal URL to ipfs.iex.ec ([#31](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/31)) ([f5a0909](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/f5a09097ed647c6ff0f98f5fbe898e6beaa01e91))
* remove sconification steps ([#28](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/28)) ([271248e](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/271248e29a90c29171e9f7cad36e9123ca1a8239))

## [0.4.0](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/compare/tee-worker-pre-compute-v0.3.0...tee-worker-pre-compute-v0.4.0) (2025-11-14)


### Features

* Return all bulk dataset processing errors ([#25](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/25)) ([2d55149](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/2d551496f75934a2f46dac64aa510d645a37a267))
* update ReplicateStatusCause serialization and worker_api to support new WorkflowError format ([#21](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/21)) ([7491cf4](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/7491cf4f4754fbe511c055ccd91cfde64a82698e))


### Bug Fixes

* delete file on write failure and preserve parent directory ([#27](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/27)) ([c7fcbb9](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/c7fcbb9b2495e2e376c72e612417d5c63fefd45f))

## [0.3.0](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/compare/tee-worker-pre-compute-v0.2.0...tee-worker-pre-compute-v0.3.0) (2025-10-09)


### Features

* add bulk dataset support with environment variables ([#16](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/16)) ([c389c2e](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/c389c2e7af2d3b72b4f523b4b81fb1ba4dcaf8be))
* rename bulk environment variables ([#20](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/20)) ([94c91a5](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/94c91a5cc58a948b13af21e9e2b3dc1a7f774caf))

## [0.2.0](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/compare/tee-worker-pre-compute-v0.1.0...tee-worker-pre-compute-v0.2.0) (2025-09-19)


### Features

* add pre-compute module ([#3](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/3)) ([4821929](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/4821929102afc1cb5db0b9e77264179640678fc0))


### Bug Fixes

* ensure exit cause is propagated and sent to worker ([#8](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/8)) ([e3310b9](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/e3310b94e5a075e23cfaf51083184fb3e7cfc527))

## 0.1.0 (2025-09-18)


### Features

* add pre-compute module ([#3](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/3)) ([4821929](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/4821929102afc1cb5db0b9e77264179640678fc0))


### Bug Fixes

* ensure exit cause is propagated and sent to worker ([#8](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/issues/8)) ([e3310b9](https://github.com/iExecBlockchainComputing/tee-worker-compute-rust/commit/e3310b94e5a075e23cfaf51083184fb3e7cfc527))
