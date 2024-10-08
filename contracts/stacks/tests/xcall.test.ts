
import { beforeEach, describe, expect, it } from "vitest";
import { encode } from "rlp";
import { Cl } from "@stacks/transactions";

const accounts = simnet.getAccounts();
const deployer = accounts.get("deployer");
const user = accounts.get("wallet_1")!;
const XCALL_IMPL_CONTRACT_NAME = "xcall-impl";
const XCALL_PROXY_CONTRACT_NAME = "xcall-proxy";
const CENTRALIZED_CONNECTION_CONTRACT_NAME = "centralized-connection";
const xcallImpl = Cl.contractPrincipal(deployer!, XCALL_IMPL_CONTRACT_NAME);
const xcallProxy = Cl.contractPrincipal(deployer!, XCALL_PROXY_CONTRACT_NAME);
const centralizedConnection = Cl.contractPrincipal(deployer!, CENTRALIZED_CONNECTION_CONTRACT_NAME);

describe("xcall", () => {
  beforeEach(() => {
    simnet.callPublicFn(
      XCALL_IMPL_CONTRACT_NAME,
      "init",
      [Cl.stringAscii("stacks"), Cl.stringAscii(XCALL_IMPL_CONTRACT_NAME)],
      deployer!
    );

    const upgradeProxyResult = simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "upgrade",
      [xcallImpl, Cl.none()],
      deployer!
    );
    expect(upgradeProxyResult.result).toBeOk(Cl.bool(true));

    simnet.callPublicFn(
      CENTRALIZED_CONNECTION_CONTRACT_NAME,
      "initialize",
      [xcallProxy, Cl.principal(deployer!)],
      deployer!
    );

    simnet.callPublicFn(
      XCALL_PROXY_CONTRACT_NAME,
      "set-default-connection",
      [Cl.stringAscii("icon"), Cl.stringAscii(deployer! + "." + CENTRALIZED_CONNECTION_CONTRACT_NAME), xcallImpl],
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
      [Cl.stringAscii("icon"), Cl.uint(1000000), Cl.uint(500000)],
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

  // it("sends a call", () => {
  //   const to = "icon/hx1234567890123456789012345678901234567890";
  //   const data = Uint8Array.from(encode(["TestMessage", "Hello, ICON!"]));

  //   const result = simnet.callPublicFn(
  //     xcallProxy.contractName.content,
  //     "send-call",
  //     [Cl.stringAscii(to), Cl.buffer(data), xcallImpl],
  //     user
  //   );

  //   expect(result.result).toBeOk(Cl.uint(1));
  //   // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
  //   expect(result.events[0].data.value!.data.event.data).toBe("CallMessageSent");
  //   // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
  //   expect(result.events[0].data.value!.data.from).toStrictEqual(Cl.principal(user));
  //   // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
  //   expect(result.events[0].data.value!.data.to).toStrictEqual(Cl.stringAscii(to));
  //   // @ts-ignore: Property 'data' does not exist on type 'ClarityValue'. Property 'data' does not exist on type 'ContractPrincipalCV'.
  //   expect(result.events[0].data.value!.data.sn).toStrictEqual(Cl.uint(1));

  //   const verifySuccessResult1 = simnet.callPublicFn(
  //     XCALL_PROXY_CONTRACT_NAME,
  //     "verify-success",
  //     [Cl.uint(1), xcallImpl],
  //     user
  //   );
  //   expect(verifySuccessResult1.result).toBeOk(Cl.bool(false));

  //   const messageData = encode([
  //     "stacks/" + deployer!, // from
  //     to, // to
  //     1, // sn
  //     1, // messageType
  //     data, // data
  //     [] // protocols (empty list)
  //   ]);

  //   const csMessageRequest = encode([
  //     1, // type (CS_MESSAGE_TYPE_REQUEST)
  //     messageData // data as buffer
  //   ]);

  //   const handleMessageResult = simnet.callPublicFn(
  //     XCALL_PROXY_CONTRACT_NAME,
  //     "handle-message",
  //     [Cl.stringAscii("stacks"), Cl.buffer(csMessageRequest), xcallImpl],
  //     deployer!
  //   );
  //   console.log(handleMessageResult.events[0].data.value)
  //   expect(handleMessageResult.result).toBeOk(Cl.bool(true));

  //   const verifySuccessResult2 = simnet.callPublicFn(
  //     XCALL_PROXY_CONTRACT_NAME,
  //     "verify-success",
  //     [Cl.uint(1), xcallImpl],
  //     user
  //   );
  //   expect(verifySuccessResult2.result).toBeOk(Cl.bool(true));
  // });

  it("parses cs-message correctly", () => {
    const from = "stacks/ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM";
    const to = "icon/hx1234567890123456789012345678901234567890";
    const sn = 1;
    const messageType = 1;
    const data = Uint8Array.from(encode(["TestMessage", "Hello, ICON!"]));
    const protocols = ["protocol1", "protocol2"];

    const messageData = encode([
      from,
      to,
      sn,
      messageType,
      data,
      protocols
    ]);

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
    expect(parsedResult.protocols).toStrictEqual(Cl.list(protocols.map(p => Cl.stringAscii(p))));
  
    const csMessage = encode([
      1, // type (CS_MESSAGE_TYPE_REQUEST)
      messageData
    ]);
  
    const parseCSMessageResult = simnet.callPrivateFn(
      XCALL_IMPL_CONTRACT_NAME,
      "parse-cs-message",
      [Cl.buffer(csMessage)],
      deployer!
    );

    console.log(Cl.buffer(messageData))
    console.log(parseCSMessageResult.result.value.data.data)

    const parseCSMessageRequestResult2 = simnet.callPrivateFn(
      XCALL_IMPL_CONTRACT_NAME,
      "parse-cs-message-request",
      [parseCSMessageResult.result.value.data.data],
      deployer!
    );

    console.log("parseCSMessageRequestResult: ", parseCSMessageRequestResult.result.value.data)
    console.log("parseCSMessageRequestResult2: ", parseCSMessageRequestResult2.result.value.data)

    expect(parseCSMessageResult.result).toBeOk(Cl.tuple({
      type: Cl.uint(1),
      data: Cl.buffer(messageData)
    }));
  });

  // it("parses network address correctly", () => {
  //   const address = "stacks/ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM";
  //   const result = simnet.callPrivateFn(
  //     XCALL_IMPL_CONTRACT_NAME,
  //     "parse-network-address",
  //     [Cl.stringAscii(address)],
  //     deployer!
  //   );

  //   expect(result.result).toBeOk(Cl.tuple({
  //     net: Cl.stringAscii("stacks"),
  //     account: Cl.stringAscii("ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM")
  //   }));
  // });

});
