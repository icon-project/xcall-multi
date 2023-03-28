use std::time::Duration;

pub mod setup;
use cw_ibc_core::ics03_connection::event::event_open_ack;
use cw_ibc_core::ics03_connection::event::event_open_init;
use cw_ibc_core::types::ClientType;
use cw_ibc_core::IbcClientType;
use ibc::core::ics03_connection::events::CLIENT_ID_ATTRIBUTE_KEY;
use ibc::core::ics03_connection::events::CONN_ID_ATTRIBUTE_KEY;
use ibc::core::ics03_connection::events::COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY;
use ibc::core::ics03_connection::events::COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::events::IbcEventType;
use setup::*;

use cw_ibc_core::context::CwIbcCoreContext;
use cw_ibc_core::types::ClientId;
use cw_ibc_core::types::ConnectionId;
use cw_ibc_core::ConnectionEnd;
use cw_ibc_core::IbcClientId;
use ibc::core::ics03_connection::connection::Counterparty;
use ibc::core::ics03_connection::connection::State;
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::version::Version;
use ibc_proto::ibc::core::client::v1::Height;
use ibc_proto::ibc::core::connection::v1::Counterparty as RawCounterparty;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm as RawMsgConnectionOpenConfirm;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit as RawMsgConnectionOpenInit;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry as RawMsgConnectionOpenTry;

#[test]
fn test_set_connection() {
    let mut deps = deps();
    let conn_end = ConnectionEnd::default();
    let conn_id = ConnectionId::new(5);
    let contract = CwIbcCoreContext::new();
    contract
        .store_connection(deps.as_mut().storage, conn_id.clone(), conn_end.clone())
        .unwrap();
    let result = contract
        .connection_end(deps.as_ref().storage, conn_id)
        .unwrap();

    assert_eq!(conn_end, result)
}

#[test]
fn test_get_connection() {
    let mut deps = deps();
    let ss = ibc::core::ics23_commitment::commitment::CommitmentPrefix::try_from(
        "hello".to_string().as_bytes().to_vec(),
    );
    let counter_party = Counterparty::new(IbcClientId::default(), None, ss.unwrap());
    let conn_end = ConnectionEnd::new(
        State::Open,
        IbcClientId::default(),
        counter_party,
        vec![Version::default()],
        Duration::default(),
    );
    let conn_id = ConnectionId::new(5);
    let contract = CwIbcCoreContext::new();
    contract
        .store_connection(deps.as_mut().storage, conn_id.clone(), conn_end.clone())
        .unwrap();
    let result = contract
        .connection_end(deps.as_ref().storage, conn_id)
        .unwrap();

    assert_eq!(conn_end, result)
}

#[test]
fn test_connection_sequence() {
    let mut store = deps();
    let contract = CwIbcCoreContext::new();
    contract
        .connection_next_sequence_init(store.as_mut().storage, u128::default())
        .unwrap();
    let result = contract.connection_counter(store.as_ref().storage).unwrap();

    assert_eq!(0, result);

    let increment_sequence = contract
        .increase_connection_counter(store.as_mut().storage)
        .unwrap();
    assert_eq!(1, increment_sequence);
}

#[test]
fn test_client_connection() {
    let mut deps = deps();
    let client_id = ClientId::default();
    let conn_id = ConnectionId::new(5);
    let contract = CwIbcCoreContext::new();

    contract
        .store_connection_to_client(deps.as_mut().storage, client_id.clone(), conn_id.clone())
        .unwrap();

    let result = contract
        .client_connection(deps.as_ref().storage, client_id)
        .unwrap();

    assert_eq!(conn_id, result)
}

#[test]
#[should_panic(expected = "Std(NotFound { kind: \"alloc::vec::Vec<u8>\" })")]
fn test_get_connection_fail() {
    let deps = deps();

    let conn_id = ConnectionId::new(5);
    let contract = CwIbcCoreContext::new();

    contract
        .connection_end(deps.as_ref().storage, conn_id)
        .unwrap();
}

#[test]
#[should_panic(expected = "Std(NotFound { kind: \"alloc::vec::Vec<u8>\" })")]
fn test_set_connection_fail() {
    let deps = deps();
    let conn_id = ConnectionId::new(0);
    let contract = CwIbcCoreContext::new();
    contract
        .connection_end(deps.as_ref().storage, conn_id)
        .unwrap();
}

#[test]
#[should_panic(expected = "Std(NotFound { kind: \"u128\" })")]
fn test_connection_sequence_fail() {
    let store = deps();
    let contract = CwIbcCoreContext::new();
    contract.connection_counter(store.as_ref().storage).unwrap();
}

#[test]
#[should_panic(expected = "Std(NotFound { kind: \"cw_ibc_core::types::ConnectionId\" })")]
fn test_client_connection_fail() {
    let deps = deps();
    let client_id = ClientId::default();

    let contract = CwIbcCoreContext::new();

    contract
        .client_connection(deps.as_ref().storage, client_id)
        .unwrap();
}

#[test]
pub fn test_to_and_from_connection_open_init() {
    let raw = get_dummy_raw_msg_conn_open_init();
    let msg = MsgConnectionOpenInit::try_from(raw.clone()).unwrap();
    let raw_back = RawMsgConnectionOpenInit::from(msg.clone());
    let msg_back = MsgConnectionOpenInit::try_from(raw_back.clone()).unwrap();
    assert_eq!(raw, raw_back);
    assert_eq!(msg, msg_back);
}
#[test]
fn test_to_and_from_connection_open_try() {
    let raw = get_dummy_raw_msg_conn_open_try(10, 34);
    let msg = MsgConnectionOpenTry::try_from(raw.clone()).unwrap();
    let raw_back = RawMsgConnectionOpenTry::from(msg.clone());
    let msg_back = MsgConnectionOpenTry::try_from(raw_back.clone()).unwrap();
    assert_eq!(raw, raw_back);
    assert_eq!(msg, msg_back);
}

#[test]
fn test_to_and_from_connection_open_ack() {
    let raw = get_dummy_raw_msg_conn_open_ack(10, 34);
    let msg = MsgConnectionOpenAck::try_from(raw.clone()).unwrap();
    let raw_back = RawMsgConnectionOpenAck::from(msg.clone());
    let msg_back = MsgConnectionOpenAck::try_from(raw_back.clone()).unwrap();
    assert_eq!(raw, raw_back);
    assert_eq!(msg, msg_back);
}

#[test]
fn test_to_and_from_connection_open_confirm() {
    let raw = get_dummy_raw_msg_conn_open_confirm();
    let msg = MsgConnectionOpenConfirm::try_from(raw.clone()).unwrap();
    let raw_back = RawMsgConnectionOpenConfirm::from(msg.clone());
    let msg_back = MsgConnectionOpenConfirm::try_from(raw_back.clone()).unwrap();
    assert_eq!(raw, raw_back);
    assert_eq!(msg, msg_back);
}

#[test]
fn connection_open_init_from_raw_valid_parameter() {
    let default_raw_init_msg = get_dummy_raw_msg_conn_open_init();
    let res_msg = MsgConnectionOpenInit::try_from(default_raw_init_msg.clone());
    assert_eq!(res_msg.is_ok(), true)
}

#[test]
fn connection_invalid_client_id_parameter() {
    let default_raw_init_msg = RawMsgConnectionOpenInit {
        client_id: "client".to_string(),
        counterparty: Some(get_dummy_raw_counterparty(None)),
        version: None,
        delay_period: 0,
        signer: get_dummy_bech32_account(),
    };
    let res_msg = MsgConnectionOpenInit::try_from(default_raw_init_msg.clone());
    assert_eq!(res_msg.is_err(), false)
}

#[test]
fn connection_open_init_invalid_destination_connection_id() {
    let default_raw_init_msg = get_dummy_raw_msg_conn_open_init;
    let default_raw_init_msg = RawMsgConnectionOpenInit {
        counterparty: Some(RawCounterparty {
            connection_id: "abcdefghijksdffjssdkflweldflsfladfsfwjkrekcmmsdfsdfjflddmnopqrstu"
                .to_string(),
            ..get_dummy_raw_counterparty(None)
        }),
        ..default_raw_init_msg()
    };
    let res_msg = MsgConnectionOpenInit::try_from(default_raw_init_msg.clone());
    assert_eq!(res_msg.is_err(), false)
}

#[test]
fn connection_open_try_from_raw_valid_parameter() {
    let default_raw_try_msg = get_dummy_raw_msg_conn_open_try(1, 3);
    let res_msg = MsgConnectionOpenTry::try_from(default_raw_try_msg.clone());
    assert_eq!(res_msg.is_ok(), true)
}

#[test]
fn connection_open_try_destination_client_id_with_lower_case_and_special_characters() {
    let default_raw_try_msg = get_dummy_raw_msg_conn_open_try(1, 3);
    let try_msg = RawMsgConnectionOpenTry {
        counterparty: Some(RawCounterparty {
            client_id: "ClientId_".to_string(),
            ..get_dummy_raw_counterparty(Some(0))
        }),
        ..default_raw_try_msg.clone()
    };
    let res_msg = MsgConnectionOpenTry::try_from(try_msg.clone());
    assert_eq!(res_msg.is_ok(), true)
}

#[test]
fn connection_open_try_invalid_client_id_name_too_short() {
    let default_raw_try_msg = get_dummy_raw_msg_conn_open_try(1, 3);
    let try_msg = RawMsgConnectionOpenTry {
        client_id: "client".to_string(),
        ..default_raw_try_msg.clone()
    };
    let res_msg = MsgConnectionOpenTry::try_from(try_msg.clone());
    assert_eq!(res_msg.is_ok(), false)
}

#[test]
fn test_commitment_prefix() {
    let contract = CwIbcCoreContext::new();
    let expected = CommitmentPrefix::try_from(b"Ibc".to_vec()).unwrap_or_default();
    let result = contract.commitment_prefix();
    assert_eq!(result, expected);
}
#[test]
fn connection_open_ack_from_raw_valid_parameter() {
    let default_raw_ack_msg = get_dummy_raw_msg_conn_open_ack(5, 5);
    let res_msg = MsgConnectionOpenAck::try_from(default_raw_ack_msg.clone());
    assert_eq!(res_msg.is_ok(), true)
}

#[test]
fn connection_open_ack_invalid_connection_id() {
    let default_raw_ack_msg = get_dummy_raw_msg_conn_open_ack(5, 5);
    let ack_msg = RawMsgConnectionOpenAck {
        connection_id: "con007".to_string(),
        ..default_raw_ack_msg.clone()
    };
    let res_msg = MsgConnectionOpenAck::try_from(ack_msg.clone());
    assert_eq!(res_msg.is_ok(), false)
}

#[test]
fn connection_open_ack_invalid_version() {
    let default_raw_ack_msg = get_dummy_raw_msg_conn_open_ack(5, 5);
    let ack_msg = RawMsgConnectionOpenAck {
        version: None,
        ..default_raw_ack_msg.clone()
    };
    let res_msg = MsgConnectionOpenAck::try_from(ack_msg.clone());
    assert_eq!(res_msg.is_ok(), false)
}

#[test]
fn connection_open_ack_invalid_proof_height() {
    let default_raw_ack_msg = get_dummy_raw_msg_conn_open_ack(5, 5);
    let ack_msg = RawMsgConnectionOpenAck {
        proof_height: Some(Height {
            revision_number: 1,
            revision_height: 0,
        }),
        ..default_raw_ack_msg.clone()
    };
    let res_msg = MsgConnectionOpenAck::try_from(ack_msg.clone());
    assert_eq!(res_msg.is_ok(), false)
}

#[test]
fn connection_open_ack_invalid_consensus_height_and_height_is_0() {
    let default_raw_ack_msg = get_dummy_raw_msg_conn_open_ack(5, 5);
    let ack_msg = RawMsgConnectionOpenAck {
        consensus_height: Some(Height {
            revision_number: 1,
            revision_height: 0,
        }),
        ..default_raw_ack_msg
    };
    let res_msg = MsgConnectionOpenAck::try_from(ack_msg.clone());
    assert_eq!(res_msg.is_ok(), false)
}

#[test]
fn connection_open_confirm_with_valid_parameter() {
    let default_raw_confirm_msg = get_dummy_raw_msg_conn_open_confirm();
    let res_msg = MsgConnectionOpenConfirm::try_from(default_raw_confirm_msg.clone());
    assert_eq!(res_msg.is_ok(), true)
}

#[test]
fn connection_open_confirm_invalid_connection_id_non_alpha() {
    let default_raw_confirm_msg = get_dummy_raw_msg_conn_open_confirm();
    let confirm_msg = RawMsgConnectionOpenConfirm {
        connection_id: "con007".to_string(),
        ..default_raw_confirm_msg.clone()
    };
    let res_msg = MsgConnectionOpenConfirm::try_from(confirm_msg.clone());
    assert_eq!(res_msg.is_err(), false)
}

#[test]
fn connection_open_confirm_invalid_proof_height() {
    let default_raw_confirm_msg = get_dummy_raw_msg_conn_open_confirm();
    let confirm_msg = RawMsgConnectionOpenConfirm {
        proof_height: Some(Height {
            revision_number: 1,
            revision_height: 0,
        }),
        ..default_raw_confirm_msg
    };
    let res_msg = MsgConnectionOpenConfirm::try_from(confirm_msg.clone());
    assert_eq!(res_msg.is_err(), false)
}

#[test]
fn connection_to_verify_correct_connection_id() {
    let connection_id = ConnectionId::new(10);
    let client_id = ClientId::default();
    let counterparty_client_id = ClientId::default();
    let event = event_open_init(connection_id, client_id, counterparty_client_id);
    let attribute = event
        .attributes
        .iter()
        .find(|attr| attr.key == CONN_ID_ATTRIBUTE_KEY)
        .expect("Missing attribute");
    assert_eq!(attribute.value, "connection-10");
}

#[test]
fn connection_to_verify_correct_client_id() {
    let connection_id = ConnectionId::new(10);
    let client_id = ClientId::default();
    let counterparty_client_id = ClientId::default();
    let event = event_open_init(connection_id, client_id, counterparty_client_id);
    let attribute = event
        .attributes
        .iter()
        .find(|attr| attr.key == CLIENT_ID_ATTRIBUTE_KEY)
        .expect("Missing attribute");
    assert_eq!(attribute.value, "07-tendermint-0");
}

#[test]
fn connection_to_verify_correct_counterparty_client_id() {
    let connection_id = ConnectionId::new(10);
    let client_id = ClientId::default();
    let counterparty_client_id = ClientId::default();
    let event = event_open_init(connection_id, client_id, counterparty_client_id);
    let attribute = event
        .attributes
        .iter()
        .find(|attr| attr.key == COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY)
        .expect("Missing attribute");
    assert_eq!(attribute.value, "07-tendermint-0");
}

#[test]
fn connection_to_verify_correct_counterparty_conn_id() {
    let connection_id = ConnectionId::new(10);
    let client_id = ClientId::default();
    let counterparty_client_id = ClientId::default();
    let counterparty_conn_id = ConnectionId::new(1);
    let event = event_open_ack(
        connection_id,
        client_id,
        counterparty_conn_id,
        counterparty_client_id,
    );
    let attribute = event
        .attributes
        .iter()
        .find(|attr| attr.key == COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY)
        .expect("Missing attribute");
    assert_eq!(attribute.value, "connection-1");
}