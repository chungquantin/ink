use crate::{
    AccountIdFor,
    ContractExecResultFor,
    ContractResultInstantiate,
    Sandbox,
    H256,
};
use frame_support::{
    pallet_prelude::{
        DispatchError,
        Zero,
    },
    sp_runtime::traits::Bounded,
    traits::{
        fungible::Inspect,
        Get,
        Time,
    },
    weights::Weight,
};
use frame_system::pallet_prelude::OriginFor;
use ink_primitives::{
    Address,
    DepositLimit,
};
use pallet_revive::{
    Code,
    CodeUploadResult,
    CodeUploadReturnValue,
    Config,
    ConversionPrecision,
    Error,
};
use sp_core::U256;
use std::ops::Not;

type BalanceOf<R> =
    <<R as pallet_revive::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

type MomentOf<T> = <<T as pallet_revive::Config>::Time as Time>::Moment;

/// Contract API used to interact with `pallet-revive`.
pub trait ContractAPI {
    /// The runtime contract config.
    type T: pallet_revive::Config;

    /// Interface for `bare_instantiate` contract call with a simultaneous upload.
    ///
    /// # Arguments
    ///
    /// * `contract_bytes` - The contract code.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including constructor
    ///   name).
    /// * `salt` - The salt to be used for contract address derivation.
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn map_account(&mut self, account: OriginFor<Self::T>) -> Result<(), DispatchError>;

    /// Interface for `bare_instantiate` contract call with a simultaneous upload.
    ///
    /// # Arguments
    ///
    /// * `contract_bytes` - The contract code.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including constructor
    ///   name).
    /// * `salt` - The salt to be used for contract address derivation.
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn deploy_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        value: U256,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<U256>,
    ) -> ContractResultInstantiate<Self::T>;

    /// Interface for `bare_instantiate` contract call for a previously uploaded contract.
    ///
    /// # Arguments
    ///
    /// * `code_hash` - The code hash of the contract to instantiate.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including constructor
    ///   name).
    /// * `salt` - The salt to be used for contract address derivation.
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn instantiate_contract(
        &mut self,
        code_hash: H256,
        value: U256,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<U256>,
    ) -> ContractResultInstantiate<Self::T>;

    /// Interface for `bare_upload_code` contract call.
    ///
    /// # Arguments
    ///
    /// * `contract_bytes` - The contract code.
    /// * `origin` - The sender of the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    fn upload_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        origin: OriginFor<Self::T>,
        storage_deposit_limit: U256,
    ) -> CodeUploadResult<U256>;

    /// Interface for `bare_call` contract call.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the contract to be called.
    /// * `value` - The number of tokens to be transferred to the contract.
    /// * `data` - The input data to be passed to the contract (including message name).
    /// * `origin` - The sender of the contract call.
    /// * `gas_limit` - The gas limit for the contract call.
    /// * `storage_deposit_limit` - The storage deposit limit for the contract call.
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn call_contract(
        &mut self,
        address: Address,
        value: U256,
        data: Vec<u8>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<U256>,
    ) -> ContractExecResultFor<Self::T>;
}

impl<T> ContractAPI for T
where
    T: Sandbox,
    T::Runtime: pallet_revive::Config,

    BalanceOf<T::Runtime>: Into<U256> + TryFrom<U256> + Bounded,
    MomentOf<T::Runtime>: Into<U256>,

    // todo
    <<T as Sandbox>::Runtime as frame_system::Config>::Hash:
        frame_support::traits::IsType<sp_core::H256>,
{
    type T = T::Runtime;

    fn map_account(
        &mut self,
        account_id: OriginFor<Self::T>,
    ) -> Result<(), DispatchError> {
        self.execute_with(|| pallet_revive::Pallet::<Self::T>::map_account(account_id))
    }

    fn deploy_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        value: U256,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<U256>,
    ) -> ContractResultInstantiate<Self::T> {
        let value =
            convert_evm_to_native::<Self::T>(value, ConversionPrecision::Exact).unwrap();
        let storage_deposit_limit =
            storage_deposit_limit_fn::<Self::T>(storage_deposit_limit).unwrap();
        self.execute_with(|| {
            pallet_revive::Pallet::<Self::T>::bare_instantiate(
                origin,
                value,
                gas_limit,
                storage_deposit_limit,
                Code::Upload(contract_bytes),
                data,
                salt,
            )
        })
    }

    fn instantiate_contract(
        &mut self,
        code_hash: H256,
        value: U256,
        data: Vec<u8>,
        salt: Option<[u8; 32]>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<U256>,
    ) -> ContractResultInstantiate<Self::T> {
        let value =
            convert_evm_to_native::<Self::T>(value, ConversionPrecision::Exact).unwrap();
        let storage_deposit_limit =
            storage_deposit_limit_fn::<Self::T>(storage_deposit_limit).unwrap();
        self.execute_with(|| {
            pallet_revive::Pallet::<Self::T>::bare_instantiate(
                origin,
                value,
                gas_limit,
                storage_deposit_limit,
                Code::Existing(code_hash),
                data,
                salt,
            )
        })
    }

    fn upload_contract(
        &mut self,
        contract_bytes: Vec<u8>,
        origin: OriginFor<Self::T>,
        storage_deposit_limit: U256,
    ) -> CodeUploadResult<U256> {
        self.execute_with(|| {
            let storage_deposit_limit = convert_evm_to_native::<Self::T>(
                storage_deposit_limit,
                ConversionPrecision::Exact,
            )
            .unwrap();
            let result = pallet_revive::Pallet::<Self::T>::bare_upload_code(
                origin,
                contract_bytes,
                storage_deposit_limit,
            );
            result.map(|r| {
                CodeUploadReturnValue {
                    code_hash: r.code_hash,
                    deposit: convert_native_to_evm::<Self::T>(r.deposit),
                }
            })
        })
    }

    fn call_contract(
        &mut self,
        address: Address,
        value: U256,
        data: Vec<u8>,
        origin: OriginFor<Self::T>,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<U256>,
    ) -> ContractExecResultFor<Self::T> {
        let value =
            convert_evm_to_native::<Self::T>(value, ConversionPrecision::Exact).unwrap();
        let storage_deposit_limit =
            storage_deposit_limit_fn::<Self::T>(storage_deposit_limit).unwrap();
        self.execute_with(|| {
            pallet_revive::Pallet::<Self::T>::bare_call(
                origin,
                address,
                value,
                gas_limit,
                storage_deposit_limit,
                data,
            )
        })
    }
}

/// Convert a native balance to EVM balance.
pub fn convert_native_to_evm<T>(value: BalanceOf<T>) -> U256
where
    T: Config,
    BalanceOf<T>: Into<U256> + TryFrom<U256> + Bounded,
{
    value
        .into()
        .saturating_mul(T::NativeToEthRatio::get().into())
}

/// Convert an EVM balance to a native balance.
pub fn convert_evm_to_native<T>(
    value: U256,
    precision: ConversionPrecision,
) -> Result<BalanceOf<T>, Error<T>>
where
    T: Config,
    BalanceOf<T>: Into<U256> + TryFrom<U256> + Bounded,
{
    if value.is_zero() {
        return Ok(Zero::zero())
    }

    let (quotient, remainder) = value.div_mod(T::NativeToEthRatio::get().into());
    match (precision, remainder.is_zero()) {
        (ConversionPrecision::Exact, false) => Err(Error::<T>::DecimalPrecisionLoss),
        (_, true) => {
            quotient
                .try_into()
                .map_err(|_| Error::<T>::BalanceConversionFailed)
        }
        (_, false) => {
            quotient
                .saturating_add(U256::one())
                .try_into()
                .map_err(|_| Error::<T>::BalanceConversionFailed)
        }
    }
}

/// todo
fn storage_deposit_limit_fn<T>(
    limit: DepositLimit<U256>,
) -> Result<pallet_revive::DepositLimit<BalanceOf<T>>, Error<T>>
where
    T: Config,
    BalanceOf<T>: Into<U256> + TryFrom<U256> + Bounded,
{
    Ok(match limit {
        DepositLimit::Unchecked => pallet_revive::DepositLimit::Unchecked,
        DepositLimit::Balance(v) => {
            pallet_revive::DepositLimit::Balance(convert_evm_to_native(
                v,
                ConversionPrecision::Exact,
            )?)
        }
    })
}

/// todo
/// Converts bytes to a '\n'-split string, ignoring empty lines.
pub fn decode_debug_buffer(buffer: &[u8]) -> Vec<String> {
    let decoded = buffer.iter().map(|b| *b as char).collect::<String>();
    decoded
        .split('\n')
        .filter_map(|s| s.is_empty().not().then_some(s.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::prelude::*,
        DefaultSandbox,
        RuntimeEventOf,
    };

    const STORAGE_DEPOSIT_LIMIT: DepositLimit<U256> = DepositLimit::Unchecked;

    fn compile_module(contract_name: &str) -> Vec<u8> {
        // todo compile the contract, instead of reading the binary
        let path = [
            std::env::var("CARGO_MANIFEST_DIR").as_deref().unwrap(),
            "/test-resources/",
            contract_name,
            ".polkavm",
        ]
        .concat();
        std::fs::read(std::path::Path::new(&path)).unwrap()
    }

    #[test]
    fn can_upload_code() {
        let mut sandbox = DefaultSandbox::default();
        let contract_binary = compile_module("dummy");

        use sha3::{
            Digest,
            Keccak256,
        };
        let hash = Keccak256::digest(contract_binary.as_slice());
        let hash = H256::from_slice(hash.as_slice());

        let origin =
            DefaultSandbox::convert_account_to_origin(DefaultSandbox::default_actor());
        let result = sandbox.upload_contract(
            contract_binary,
            origin,
            U256::from(100000000000000u128),
        );

        assert!(result.is_ok());
        assert_eq!(hash, result.unwrap().code_hash);
    }

    #[test]
    fn can_deploy_contract() {
        let mut sandbox = DefaultSandbox::default();
        let contract_binary = compile_module("dummy");

        let events_before = sandbox.events();
        assert!(events_before.is_empty());

        let origin =
            DefaultSandbox::convert_account_to_origin(DefaultSandbox::default_actor());
        sandbox.map_account(origin.clone()).expect("cannot map");
        let result = sandbox.deploy_contract(
            contract_binary.clone(),
            U256::from(0),
            vec![],
            None,
            origin.clone(),
            DefaultSandbox::default_gas_limit(),
            DepositLimit::Balance(U256::from(100000000000000u128)),
        );
        assert!(result.result.is_ok());
        assert!(!result.result.unwrap().result.did_revert());

        // deploying again must fail due to `DuplicateContract`
        let result = sandbox.deploy_contract(
            contract_binary,
            U256::from(0),
            vec![],
            None,
            origin,
            DefaultSandbox::default_gas_limit(),
            DepositLimit::Balance(U256::from(100000000000000u128)),
        );
        assert!(result.result.is_err());
        let dispatch_err = result.result.unwrap_err();
        assert!(format!("{dispatch_err:?}").contains("DuplicateContract"));
    }

    #[test]
    fn can_call_contract() {
        let mut sandbox = DefaultSandbox::default();
        let _actor = DefaultSandbox::default_actor();
        let contract_binary = compile_module("dummy");

        let origin =
            DefaultSandbox::convert_account_to_origin(DefaultSandbox::default_actor());
        sandbox.map_account(origin.clone()).expect("unable to map");
        let result = sandbox.deploy_contract(
            contract_binary,
            U256::from(0),
            vec![],
            None,
            origin.clone(),
            DefaultSandbox::default_gas_limit(),
            STORAGE_DEPOSIT_LIMIT,
        );
        assert!(!result.result.clone().unwrap().result.did_revert());

        let contract_address = result.result.expect("Contract should be deployed").addr;

        sandbox.reset_events();

        let result = sandbox.call_contract(
            contract_address,
            U256::from(0),
            vec![],
            origin.clone(),
            DefaultSandbox::default_gas_limit(),
            STORAGE_DEPOSIT_LIMIT,
        );
        assert!(result.result.is_ok());
        assert!(!result.result.unwrap().did_revert());

        let events = sandbox.events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].event,
            RuntimeEventOf::<DefaultSandbox>::Revive(
                pallet_revive::Event::ContractEmitted {
                    contract: contract_address,
                    topics: vec![H256::from([42u8; 32])],
                    data: vec![1, 2, 3, 4],
                }
            )
        );
    }
}
