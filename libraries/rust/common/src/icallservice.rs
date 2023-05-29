use cosmwasm_std::{
    Env, MessageInfo, Response, StdError, StdResult, Storage,
};

#[derive(Clone)]
pub struct ICallService {}

impl ICallService {
    pub fn get_btp_address(&self, _env: Env) -> StdResult<String> {
        Ok("BTP_ADDRESS".to_string())
    }

    pub fn send_call_message(
        &self,
        _env: Env,
        _to: String,
        _data: Vec<u8>,
        _rollback: Vec<u8>,
    ) -> StdResult<Response> {

        Ok(Response::default())
    }
}
