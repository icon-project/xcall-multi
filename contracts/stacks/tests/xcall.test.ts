import { beforeEach, describe, expect, it } from "vitest";
import { encode } from "rlp";
import { Cl } from "@stacks/transactions";

const accounts = simnet.getAccounts();
const deployer = accounts.get("deployer");
const sourceContract = accounts.get("wallet_1")!;
const destinationContract = accounts.get("wallet_2")!;
const XCALL_IMPL_CONTRACT_NAME = "xcall-impl";
const XCALL_PROXY_CONTRACT_NAME = "xcall-proxy";
const CENTRALIZED_CONNECTION_CONTRACT_NAME = "centralized-connection";
const STACKS_NID = "stacks";
const ICON_NID = "icon";
const from = `${STACKS_NID}/${sourceContract}`;
const to = `${ICON_NID}/${destinationContract}`;
const CS_MESSAGE_TYPE_REQUEST = 1;
const CS_MESSAGE_TYPE_RESULT = 2;
const CS_MESSAGE_RESULT_SUCCESS = 1;
const CS_MESSAGE_RESULT_FAILURE = 0;
const xcallImpl = Cl.contractPrincipal(deployer!, XCALL_IMPL_CONTRACT_NAME);
const xcallProxy = Cl.contractPrincipal(deployer!, XCALL_PROXY_CONTRACT_NAME);
const centralizedConnection = Cl.contractPrincipal(
  deployer!,
  CENTRALIZED_CONNECTION_CONTRACT_NAME
);

describe("xcall", () => {
  beforeEach(() => {
    simnet.callPublicFn(
      XCALL_IMPL_CONTRACT_NAME,
      "init",
      [Cl.stringAscii(STACKS_NID), Cl.stringAscii(XCALL_IMPL_CONTRACT_NAME)],
      deployer!
    );

    const upgradeProxyResult = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "upgrade",
      [xcallImpl, Cl.none()],
      deployer!
    );
    expect(upgradeProxyResult.result).toBeOk(Cl.bool(true));

    const setAdminResult = simnet.callPublicFn(
      XCALL_IMPL_CONTRACT_NAME,
      "set-admin",
      [Cl.principal(deployer!)],
      deployer!
    );
    expect(setAdminResult.result).toBeOk(Cl.bool(true));    

    simnet.callPublicFn(
      CENTRALIZED_CONNECTION_CONTRACT_NAME,
      "initialize",
      [xcallProxy, Cl.principal(deployer!)],
      deployer!
    );

    simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "set-default-connection",
      [
        Cl.stringAscii(STACKS_NID),
        Cl.stringAscii(deployer! + "." + CENTRALIZED_CONNECTION_CONTRACT_NAME),
        xcallImpl,
      ],
      deployer!
    );

    simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "set-default-connection",
      [
        Cl.stringAscii(ICON_NID),
        Cl.stringAscii(deployer! + "." + CENTRALIZED_CONNECTION_CONTRACT_NAME),
        xcallImpl,
      ],
      deployer!
    );

    simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "set-protocol-fee-handler",
      [centralizedConnection, xcallImpl],
      deployer!
    );

    simnet.callPublicFn(
      CENTRALIZED_CONNECTION_CONTRACT_NAME,
      "set-fee",
      [Cl.stringAscii(STACKS_NID), Cl.uint(500000), Cl.uint(250000)],
      deployer!
    );

    simnet.callPublicFn(
      CENTRALIZED_CONNECTION_CONTRACT_NAME,
      "set-fee",
      [Cl.stringAscii(ICON_NID), Cl.uint(1000000), Cl.uint(500000)],
      deployer!
    );

    const protocolFee = 100000;
    simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "set-protocol-fee",
      [Cl.uint(protocolFee), xcallImpl],
      deployer!
    );
  });

  it("sends and executes a call", () => {
    const data = Uint8Array.from(encode(["Hello, Destination Contract!"]));
    const expectedSn = 1;
    const expectedReqId = 1;

    const sendCallResult = simnet.callPublicFn(
      xcallProxy.contractName.content,
      "send-call",
      [Cl.stringAscii(to), Cl.buffer(data), xcallImpl],
      sourceContract
    );

    expect(sendCallResult.result).toBeOk(Cl.uint(expectedSn));
    const callMessageSentEvent = sendCallResult.events.find(e => 
      e.event === 'print_event' &&
      // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
      e?.data.value?.data.event.data === 'CallMessageSent'
    );

    // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
    expect(callMessageSentEvent?.data.value!.data.event.data).toBe("CallMessageSent");
    // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
    expect(callMessageSentEvent?.data.value!.data.from).toStrictEqual(Cl.principal(sourceContract));
    // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
    expect(callMessageSentEvent?.data.value!.data.to).toStrictEqual(Cl.stringAscii(to));
    // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
    expect(callMessageSentEvent?.data.value!.data.sn).toStrictEqual(Cl.uint(1));

    const messageData = encode([
      from,
      to,
      expectedSn,
      expectedReqId,
      data,
      []
    ]);

    const csMessageRequest = encode([
      CS_MESSAGE_TYPE_REQUEST,
      messageData
    ]);

    const handleMessageResult = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "handle-message",
      [Cl.stringAscii(STACKS_NID), Cl.buffer(csMessageRequest), xcallImpl],
      deployer!
    );
    
    expect(handleMessageResult.result).toBeOk(Cl.bool(true));

    const callMessageEvent = handleMessageResult.events.find(e => 
      e.event === 'print_event' &&
      // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
      e.data.value!.data.event.data === 'CallMessage'
    );
    expect(callMessageEvent).toBeDefined();
    // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
    const callMessageData = callMessageEvent!.data.value!.data;
    expect(callMessageData.from.data).toBe(from);
    expect(callMessageData.to.data).toBe(to);
    expect(Number(callMessageData.sn.value)).toBe(expectedSn);
    expect(Number(callMessageData['req-id'].value)).toBe(expectedReqId);

    const slicedData = data.slice(1); // rlp decode drops length byte
    const executeCallResult = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "execute-call",
      [Cl.uint(expectedReqId), Cl.buffer(slicedData), xcallImpl],
      destinationContract
    );
    expect(executeCallResult.result).toBeOk(Cl.bool(true));

    const callExecutedEvent = executeCallResult.events.find(e => 
      e.event === 'print_event' &&
      // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
      e.data.value!.data.event.data === 'CallExecuted'
    );
    expect(callExecutedEvent).toBeDefined();
    // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
    const callExecutedData = callExecutedEvent!.data.value!.data;
    expect(Number(callExecutedData.code.value)).toBe(CS_MESSAGE_RESULT_SUCCESS);
    expect(Number(callExecutedData['req-id'].value)).toBe(expectedReqId);
    expect(callExecutedData.msg.data).toBe("");

    const responseData = encode([
      expectedSn,
      CS_MESSAGE_RESULT_SUCCESS
    ]);

    const csMessageResponse = encode([
      CS_MESSAGE_TYPE_RESULT,
      responseData
    ]);

    const handleResponseResult = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "handle-message",
      [Cl.stringAscii(STACKS_NID), Cl.buffer(csMessageResponse), xcallImpl],
      deployer!
    );
    
    expect(handleResponseResult.result).toBeOk(Cl.bool(true));    

    const verifySuccessResult = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "verify-success",
      [Cl.uint(1), xcallImpl],
      sourceContract
    );
    expect(verifySuccessResult.result).toBeOk(Cl.bool(true));
  });

  it("sends a message with rollback and executes rollback on failure", () => {
    const expectedSn = 1;
    const expectedReqId = 1;
    const data = Uint8Array.from(encode(["Hello, Destination Contract!"]));
    const rollbackData = Uint8Array.from(encode(["Rollback data"]));

    const sendCallResult = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "send-call-message",
      [Cl.stringAscii(to), Cl.buffer(data), Cl.some(Cl.buffer(rollbackData)), Cl.none(), Cl.none(), xcallImpl],
      sourceContract
    );
    expect(sendCallResult.result).toBeOk(Cl.uint(1));

    const messageData = encode([
      from,
      to,
      expectedSn,
      expectedReqId,
      data,
      []
    ]);

    const csMessageRequest = encode([
      CS_MESSAGE_TYPE_REQUEST,
      messageData
    ]);

    simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "handle-message",
      [Cl.stringAscii(ICON_NID), Cl.buffer(csMessageRequest), xcallImpl],
      deployer!
    );

    const failureResponseData = encode([expectedSn, CS_MESSAGE_RESULT_FAILURE]);
    const csMessageResponse = encode([CS_MESSAGE_TYPE_RESULT, failureResponseData]);

    const handleFailureResult = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "handle-message",
      [Cl.stringAscii(ICON_NID), Cl.buffer(csMessageResponse), xcallImpl],
      deployer!
    );
    expect(handleFailureResult.result).toBeOk(Cl.bool(true));

    const responseMessageEvent = handleFailureResult.events.find(e =>
      // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
      e.event === 'print_event' && e.data.value!.data.event.data === 'ResponseMessage'
    );
    expect(responseMessageEvent).toBeDefined();
    // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
    expect(responseMessageEvent!.data.value!.data.code).toStrictEqual(Cl.uint(CS_MESSAGE_RESULT_FAILURE));

    const rollbackMessageEvent = handleFailureResult.events.find(e =>
      // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
      e.event === 'print_event' && e.data.value!.data.event.data === 'RollbackMessage'
    );
    expect(rollbackMessageEvent).toBeDefined();

    const executeRollbackResult = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "execute-rollback",
      [Cl.uint(expectedSn), xcallImpl],
      sourceContract
    );
    expect(executeRollbackResult.result).toBeOk(Cl.bool(true));

    const rollbackExecutedEvent = executeRollbackResult.events.find(e =>
      // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
      e.event === 'print_event' && e.data.value!.data.event.data === 'RollbackExecuted'
    );
    expect(rollbackExecutedEvent).toBeDefined();

    const getOutgoingMessage = simnet.callReadOnlyFn(
      XCALL_IMPL_CONTRACT_NAME,
      "get-outgoing-message",
      [Cl.uint(expectedSn)],
      deployer!
    );
    expect(getOutgoingMessage.result).toBeNone();
  });

  it("calculates fees correctly for different scenarios", () => {
    
    const sourceContract = accounts.get("wallet_1")!;
  
    const feeStacksNoRollback = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "get-fee",
      [Cl.stringAscii(from), Cl.bool(false), Cl.none(), xcallImpl],
      sourceContract
    );
    expect(feeStacksNoRollback.result).toBeOk(Cl.uint(600000)); // 500000 base fee + 100000 protocol fee

    const feeStacksWithRollback = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "get-fee",
      [Cl.stringAscii(from), Cl.bool(true), Cl.none(), xcallImpl],
      sourceContract
    );
    expect(feeStacksWithRollback.result).toBeOk(Cl.uint(850000)); // 500000 base fee + 250000 rollback fee + 100000 protocol fee
  
    const feeIconWithRollback = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "get-fee",
      [Cl.stringAscii(to), Cl.bool(true), Cl.none(), xcallImpl],
      sourceContract
    );
    expect(feeIconWithRollback.result).toBeOk(Cl.uint(1600000)); // 1000000 base fee + 500000 rollback fee + 100000 protocol fee
  });

  it("parses cs-message correctly", () => {
    const sn = 1;
    const messageType = 1;
    const data = Buffer.from("Hello, ICON!");
    const protocols = ["protocol1", "protocol2"];

    const messageData = encode([from, to, sn, messageType, data, protocols]);

    const parseCSMessageRequestResult = simnet.callPrivateFn(
      XCALL_IMPL_CONTRACT_NAME,
      "parse-cs-message-request",
      [Cl.buffer(messageData)],
      deployer!
    );

    // @ts-ignore: Property 'value' does not exist on type 'ClarityValue'. Property 'value' does not exist on type 'ContractPrincipalCV'.
    const parsedResult = parseCSMessageRequestResult.result.value.data;
    expect(parsedResult.from).toStrictEqual(Cl.stringAscii(from));
    expect(parsedResult.to).toStrictEqual(Cl.stringAscii(to));
    expect(parsedResult.sn).toStrictEqual(Cl.uint(sn));
    expect(parsedResult.type).toStrictEqual(Cl.uint(messageType));
    expect(parsedResult.protocols).toStrictEqual(
      Cl.list(protocols.map((p) => Cl.stringAscii(p)))
    );

    const csMessage = encode([
      CS_MESSAGE_TYPE_REQUEST,
      messageData,
    ]);

    const parseCSMessageResult = simnet.callPrivateFn(
      XCALL_IMPL_CONTRACT_NAME,
      "parse-cs-message",
      [Cl.buffer(csMessage)],
      deployer!
    );

    expect(parseCSMessageResult.result).toBeOk(
      Cl.tuple({
        type: Cl.uint(1),
        data: Cl.buffer(messageData),
      })
    );

    const parseCSMessageRequestResult2 = simnet.callPrivateFn(
      XCALL_IMPL_CONTRACT_NAME,
      "parse-cs-message-request",
      // @ts-ignore: Property 'value' does not exist on type 'ClarityValue'. Property 'value' does not exist on type 'ContractPrincipalCV'.
      [parseCSMessageResult.result.value.data.data],
      deployer!
    );

    // @ts-ignore: Property 'value' does not exist on type 'ClarityValue'. Property 'value' does not exist on type 'ContractPrincipalCV'.
    const parsedResult2 = parseCSMessageRequestResult2.result.value.data;
    expect(parsedResult2.from).toStrictEqual(Cl.stringAscii(from));
    expect(parsedResult2.to).toStrictEqual(Cl.stringAscii(to));
    expect(parsedResult2.sn).toStrictEqual(Cl.uint(sn));
    expect(parsedResult2.type).toStrictEqual(Cl.uint(messageType));
    expect(parsedResult2.protocols).toStrictEqual(
      Cl.list(protocols.map((p) => Cl.stringAscii(p)))
    );
  });
});
