//! The inter-canister call functions of ic-cdk 0.17, on top of ic-cdk 0.20's `Call` API.
//!
//! Calls are bounded-wait, with the default timeout of `Call::bounded_wait`, so that they can
//! reach a subnet that refuses unbounded-wait calls (e.g., a cloud engine). A bounded-wait call
//! that times out is rejected with `SysUnknown`: the call may or may not have taken effect.
//!
//! Calls that attach cycles are unbounded-wait instead: such a subnet refuses them either way, and
//! a bounded-wait call that times out loses the cycles the callee did not accept.

use candid::utils::{ArgumentDecoder, ArgumentEncoder};
use candid::Principal;
use ic_cdk::call::{Call, Error};

pub use ic_cdk::call::RejectCode as RejectionCode;

pub type CallResult<R> = Result<R, (RejectionCode, String)>;

pub async fn call<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
) -> CallResult<R> {
    call_with_payment128(id, method, args, 0).await
}

/// [`call`], but unbounded-wait: for a call whose caller must see its outcome however long it
/// takes, and that never reaches a subnet that refuses such calls.
pub async fn call_unbounded<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
) -> CallResult<R> {
    let response = Call::unbounded_wait(id, method)
        .with_args(&args)
        .await
        .map_err(|err| rejection(err.into()))?;
    response.candid_tuple().map_err(|err| rejection(err.into()))
}

pub async fn call_with_payment128<T: ArgumentEncoder, R: for<'a> ArgumentDecoder<'a>>(
    id: Principal,
    method: &str,
    args: T,
    cycles: u128,
) -> CallResult<R> {
    let response = wait(id, method, cycles)
        .with_args(&args)
        .with_cycles(cycles)
        .await
        .map_err(|err| rejection(err.into()))?;
    response.candid_tuple().map_err(|err| rejection(err.into()))
}

pub async fn call_raw(
    id: Principal,
    method: &str,
    args_raw: impl AsRef<[u8]>,
    cycles: u64,
) -> CallResult<Vec<u8>> {
    wait(id, method, cycles.into())
        .with_raw_args(args_raw.as_ref())
        .with_cycles(cycles.into())
        .await
        .map(|response| response.into_bytes())
        .map_err(|err| rejection(err.into()))
}

fn wait(id: Principal, method: &str, cycles: u128) -> Call<'_, 'static> {
    if cycles > 0 {
        Call::unbounded_wait(id, method)
    } else {
        Call::bounded_wait(id, method)
    }
}

fn rejection(err: Error) -> (RejectionCode, String) {
    match err {
        Error::CallRejected(rejected) => {
            let code = rejected.reject_code().unwrap_or(RejectionCode::SysUnknown);
            let mut message = rejected.reject_message().to_string();
            if code == RejectionCode::SysUnknown {
                message.push_str(" (the call may or may not have taken effect)");
            }
            (code, message)
        }
        Error::CandidDecodeFailed(err) => (RejectionCode::CanisterError, err.to_string()),
        Error::InsufficientLiquidCycleBalance(err) => {
            (RejectionCode::SysTransient, err.to_string())
        }
        Error::CallPerformFailed(err) => (RejectionCode::SysTransient, err.to_string()),
    }
}
