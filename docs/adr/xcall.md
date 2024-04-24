# XCall

## Introduction

xCall is a standard interface to make calls between different blockchain networks.

## Protocol Overview

![](https://hackmd.io/_uploads/SkBUxFhBh.png)

## Prerequisites

### Network Addresses

XCall uses network address to refer to different addresses across many networks.
A `network address` consists of a network and an account section.
A network address is represented as a string with "networkId" and "account" separated by /.
A networkId is a unique id of a network, and there can't be two networks with the same id connected to the same xCall
network.

```
NetworkAddress {
    String account,
    String net
}
```

### Connections

XCall is designed to utilize a wide range of bridging protocols that facilitate data transfer, known as connections.
These connections can be selected by users and dApps, ensuring a permissionless protocol.
However, this places a responsibility on the dApps to verify that they exclusively accept messages from trusted
protocols.
Users can also opt to use the default connections set up by the xCall admin, and in this case does not need to manage
connections at all.

## Protocol Specification

### Sending Messages

Sending messages via xCall is done by constructing a xCall message envelope and calling `sendCall` with a
destination networkAddress `_to`.
```
/**
 * Sends a call message to the contract on the destination chain.
 *
 * @param _to The network address of the callee on the destination chain
 * @param _data The xCall envelope
 * @return The serial number of the request
 */
payable external sendCall(String _to, byte[] _data) returns Integer
```

##### Message Envelope Structure

All structure are RLP encoded in the order as shown below.

The Envelope is the structure received by sendCall method.
```
Envelope {
    int messageType,
    Message message,
    String[] sources,
    String[] destinations,
}
```

##### Message Objects
All Message objects have a TypeId specifying its type

A CallMessage will try to execute on the destination chain, and will be removed even if the execution fails and no response will be relayed back
```
TypeId = 1
CallMessage {
    byte[] data
}

```

```
TypeId = 2
CallMessageWithRollback {
    byte[] data,
    byte[] rollback,

}
```

#### Legacy send Interface
Sending messages via xCall can be done by calling sendCallMessage on the xCall contract.
`_to` address is a networkAddress used by xCall to figure out the destination chain.
The user can also specify which connections to use, if not specified, the default connections will be used.
This also allows dapps to have their messages secured by multiple protocols.

The default connections are specified by and admin and can be changed at any time.
The `_data` is limited to 2048 KB.
The `_rollback` data is limited to  1024KB.

```
/**
 * Sends a call message to the contract on the destination chain.
 *
 * @param _to The network address of the callee on the destination chain
 * @param _data The calldata specific to the target contract. Max 2048KB
 * @param _rollback (Optional) Data used to specify error handling of a two-way messages. Max 1024KB
 * @param _sources  (Optional) The contracts that will be used to send the message
 * @param _destinations (Optional) The addresses of the contracts that xcall will expect the message from.
 *
 * @return The serial number of the request
 */
payable external sendCallMessage(String _to,
                                byte[] _data,
                                @Optional byte[] _rollback,
                                @Optional String[] _sources
                                @Optional String[] _destinations) return Integer
```

### Events

#### CallMessage

CallMessage event is emitted when a new message is received by xCall and is ready to be executed.

- `_from` The network address of the caller on the source chain
- `_to` A string representation of the callee address
- `_sn` The serial number of the request from the source
- `_reqId` The request id of the destination chain used in execute call
- `_data` The calldata

```
CallMessage {
    String _from,
    String _to,
    Integer _sn,
    Integer _reqId,
    byte[] _data
}
```

#### CallExecuted

CallExecuted event is emitted when a message is executed

- `_reqId` The message id
- `_code` The execution result code (1: Success, 0: failure)
- `_msg` Error message

```
CallExecuted{
    Integer _reqId,
    Integer _code,
    String _msg
}
```

#### ResponseMessage

ResponseMessage is emitted for all two-way messages (i.e., _rollback is non-null), the xcall on the source chain
receives a response message from the xcall on the destination chain and emits the following event regardless of its
success or not.

- `_sn` The message id
- `_code` The execution result code (1: Success, 0: failure)

```
ResponseMessage{
    Integer _sn,
    Integer _code
}
```

#### RollbackMessage

RollbackMessage is emitted when an error occurred on the destination chain and the _rollback is non-null, xcall on the
source chain emits the following event for notifying the user that an additional rollback operation is required.

- `_sn` The message id

```
RollbackMessage {
    Integer _sn
}
```

#### RollbackExecuted

RollbackExecuted event is emitted when a rollback message is executed.

- `_sn` The message id

```
RollbackExecuted{
    Integer _sn
}
```

#### CallMessageSent

CallMessageSent is emitted for each sent message.

- ` _from` The network address of the caller
- `_to` The network address of the callee
- `_sn` The serial number of the request

```
CallMessageSent{
    Address _from,
    String _to,
    Integer _sn
}
```

### Receiving Messages

#### Execution

The user on the destination chain recognizes the call request and invokes the following method on xcall with the
given `_reqId` and `_data`.
To minimize the gas cost, the calldata payload delivered from the source chain is exported to event, `_data` field,
instead of storing it in the state db.
In the case of a two-way message, rollback will be triggered in case of failure.
The `_data` payload should be repopulated by the user (or client) when calling the following `executeCall` method.
Then `xcall` compares it with the saved hash value to validate its integrity.

```
/**
 * Executes the requested call message.
 *
 * @param _reqId The request Id
 * @param _data The calldata
 */
external executeCall(BigInteger _reqId, byte[] _data)
```

The user on the source chain recognizes the rollback situation and invokes the following method on xcall with the
given `_sn`.
Note that the executeRollback can be called only when the original call request has responded with a failure.
It should be reverted when there is no failure response with the call request.

```
/**
 * Rollbacks the caller state of the request '_sn'.
 *
 * @param _sn The serial number of the previous request
 */
external executeRollback(BigInteger _sn)
```

#### Handling

When the user calls executeCall or executeRollback method, the xcall invokes the following predefined method in the
target DApp with the calldata associated in _reqId.
If only using default protocols, implementing only the two parameter versions of function call is preferred.

```
/**
 * Handles the call message received from the source chain.
 * Only called from the Call Message Service.
 *
 * @param _from The network address of the caller on the source chain
 * @param _data The calldata delivered from the caller
 * @param _protocols The contract addresses that delivered the data, if omitted the default protocol was used
 */
external handleCallMessage(String _from, byte[] _data)
external handleCallMessage(String _from, byte[] _data, @Optional String[] _protocols)
```

In case of rollback, the `_from` will be the network address of the xCall contract.
A rollback can only be delivered by the same protocols used to send the message,
so the `_protocols` will be the protocols used to send the message.
So they can be assumed to be safe as long as all messages sent have been done using trusted protocols.

Example implementations of handleCallMessage:
```javascript
external handleCallMessage(String _from, byte[] _data) {
    assert caller == xCall
    if (_from == xCallNetworkAddress):
        //Rollback logic
    else:
        //Application logic
}
```

The below example requires that all the trusted protocols where used in the message but many different strategies could be used. For example asserting that one of the trusted protocols was used would also be a possible verification strategy.
```javascript
external handleCallMessage(String _from, byte[] _data, String[] _protocols) {
    assert caller == xCall
    if (_from == xCallNetworkAddress):
        //Rollback logic
    else:
        nid = NetworkAddress(_from).net()
        assert myTrustedProtocols[nid] is in _protocols
        //Application logic
}
```

#### Success verification

If rollback was specified and the call was successful, the success can be verified.

```
/**
 * checks if message '_sn' did succeed on target chain.
 *
 * @param _sn The serial number of the request
 *
 * @return If the '_sn' has received a success response
 */
external readonly verifySuccess(BigInteger _sn) returns boolean
```

### Fee Management

Sending a message through xCall has two types of fees. One for using the protocol and one for each connection used.

```
/**
 * Gets the fee for delivering a message to the _net.
 * If the sender is going to provide rollback data, the _rollback param should set as true.
 * The returned fee is the total fee required to send the message.
 *
 * @param _protocol The protocol/connection used
 * @param _net The network id
 * @param _rollback Indicates whether it provides rollback data
 * @param _sources The protocols used to send the message is omitted default protocol is used.
 * @return The total fee of sending the message
 */
external readonly getFee(String _net,
                          boolean _rollback
                          @Optional String[] _sources) returns Integer
```

```
/**
 * Gets the protocol fee for sending a xCall message
 *
 * @return the xCall protocol fee
 */
external readonly getProtocolFee() Returns Integer
```

### Security Considerations

The security of xCall comes from the security of the underlying connections.
It is up to the dapp to verify that protocols are checked to be valid during `handleCallMessage`.
Any address can deliver messages to xCall and are assumed to be correct and will be delivered to the dapp which can then discard the message in case of invalid protocols.

## Implementation Guidelines

### Connections

The provided code snippet demonstrates a specific behavior of a connection.
It consists of two external functions: `sendMessage(targetNetwork, svc, sn, msg)` and `getFee(network, response)`.

The `sendMessage` function is responsible for sending a message to a specified targetNetwork.
It accepts four parameters: `targetNetwork`, `svc`, `sn`, and `msg`.

The behavior of sendMessage depends on the value of sn (sequence number).

- If `sn > 0`, it indicates a new message that requires a response.
  In this case, both the sending fee and the response fee should be included.
- If `sn == 0`, it signifies a one-way message where no response is expected.
- If `sn < 0`, it implies that the message is a response to a previously received message.
  In this scenario, no fee is included in the sending message since it should have already been paid when the positive
  sn was sent.

After handling the sn value, the sendMessage function triggers the `handleMessage` function on the targetNetwork.
It passes targetNetwork and msg as arguments to handleMessage on xCall.

In case the message fails to be delivered for any reason, the connection triggers the `handleError` function.
It passes the failed sn to the function.
The responsibility of this function is to handle errors that occur during the message delivery process.

The second external function, `getFee(network, response)`, calculates and returns the fee required to send a message to
the specified network and back.
It takes into account the optional response parameter when determining the fee.

In summary, this code snippet illustrates a specific behavior expected from a connection regarding message sending and
error handling.

```
external function sendMessage(targetNetwork, svc, sn, msg)
    On targetNetwork, trigger handleMessage(targetNetwork, msg)
    if message fails to deliver:
        trigger handleError(sn)
```

```
external function getFee(network, response)
    Returns the fee required to send a message to "network" and back, considering the optional response parameter.
```

### Data Structures

#### Messages

All messages when passed to a connection are RLP encoded.
RLP encoding order is the same as the order they are defined in below.

##### CSMessageRequest

```
CSMessageRequest {
    String from
    String to
    BigInteger sn
    int messageType
    // RLP encoded message
    byte[] message
    String[] protocols
}
```

##### CSMessageResult

```
int SUCCESS = 1
int FAILURE = 0
CSMessageResult {
    BigInteger sn
    int code
    byte[] message
}
```

##### CSMessage

```
int REQUEST = 1
int RESULT = 2
CSMessage {
  // The message type, either REQUEST or RESULT
  int type
  // RLP encoded bytes of the Message
  byte[] data
}
```

#### Internal structs

```
RollbackData {
    Address from
    String netTo
    String[] protocols
    byte[] rollback
    boolean enabled = false // defaults to false
}
```

### Storage

```
MAX_DATA_SIZE: 2048
MAX_ROLLBACK_SIZE: 1024
NID: <networkId>

sn: <current send message sequence>
reqId: <current incoming message sequence>
rollbacks: sn -> RollbackData
proxyReqs: reqId -> CSMessageRequest

// default values should be false in case of boolean storage
pendingMessages: msgHash -> connection address -> boolean
successfulResponses: sn -> boolean

admin: <admin>
defaultConnection: networkId -> Address
protocolFee: <protocolFee>
feeHandler: <Address>
replyState: CallRequest,
callReply: CSMessageRequest
```

### Contract initialization

```
function init(String networkId):
    NID = networkId
    admin = getCaller()
    feeHandler = getCaller()
```

### Communication

#### Sending messages

`sendCall` sends some arbitrary data to `_to` via a path specified by the caller.

- `_to`: The network address of the target contract.
- `_data`: The rlp encoded xCall envelope.

```
payable external sendCall(String _to, byte[] _data) returns Integer {
    caller = getCaller()
    envelope = Envelope.decode(_data)
    sn++
    from = NetworkAddress(NID, caller)
    to = NetworkAddress(_to)

    needResponse, msg = preProcessMessage(sn, to, envelope)

    msgReq = CSMessageRequest(from, to.account(), sn, envelope.type, msg, envelope.destinations)
    msg = CSMessage(CSMessage.REQUEST, msgReq.toBytes()).toBytes()
    assert msg.length <= MAX_DATA_SIZE

     if isReply(_to.netId,envelope.sources) && !needResponse:
        replyState = null
        callReply = msg
        emit CallMessageSent(caller, dst.toString(), sn)
        return sn

    sendSn = needResponse ? sn : 0
    if protocolConfig.sources == []:
        src = defaultConnection[to.net()]
        fee = src->getFee(to.net(), needResponse)
        src->sendMessage(fee, to.net(), "xcall-multi", sendSn, msg)
    else:
        for src in protocolConfig.sources:
            fee = src->getFee(to.net(), needResponse)
            src->sendMessage(fee, to.net(), "xcall-multi", sendSn, msg)


    remainingBalance = getBalance()
    assert remainingBalance >= getProtocolFee()
    transfer(feeHandler, balance)
    emit CallMessageSent(caller, dst.toString(), sn)
    return sn
}

```
`sendCallMessage` sends some arbitrary data to `_to` via a path specified by the caller.

- `_to`: The network address of the target contract.
- `_data`: The data to be sent to the `_to` contract.
- `_rollback`: The data to be returned to the caller in case of failure.
- `_sources`: A set of addresses representing the connections to be used when sending the message.
  These connections are also used to verify potential rollbacks
- `_destination`: The addresses that the target contract should wait for messages from before considering it complete.

```
payable external function sendCallMessage(String _to,
                                          byte[] _data,
                                          @Optional byte[] _rollback,
                                          @Optional String[] _sources
                                          @Optional String[] _destinations) returns Integer {
    if (_rollback == null || _rollback.length == 0):
        msg = new CallMessage(_data)
    else:
        msg = new CallMessageWithRollback(_data, _rollback)

    envelope = new Envelope(msg, _sources, _destinations)
    return sendCall(_to, envelope.toBytes())


}
```

Internal method where message types can implement type specific logic.
```
internal function preProcessMessage(int sn, NetworkAddress to, Envelope envelope) return (boolean, byte[]) {
    switch (envelope.type) {
        case CallMessage.Type:
            return false, message
        case CallMessageWithRollback.Type:
            assert caller.isContract()
            msg = CallMessageWithRollback(message)
            req = RollbackData(caller, to.net(), envelope.sources, msg.rollback)
            rollbacks[sn] = req

            return true, msg.data
    }
}
```

#### Receiving messages

`handleMessage` is the external function used by connections to deliver messages.
```
external function handleMessage(String _fromNid, byte[] _msg) {
    msg = CSMessage.decode(_msg)
    assert _fromNid != nid
    switch (msg.type) :
        case CSMessage.REQUEST:
            handleRequest(_fromNid, msg.data)
            break
        case CSMessage.RESULT:
            handleResult(msg.data)
            break
        default:
            Context.revert("UnknownMsgType(" + msg.type + ")")
}
```

`handleError` is the external function used by connections to report error messages.

```
external function handleError(BigInteger _sn) {
        CSMessageResult res = CSMessageResult(_sn, CSMessageResult.FAILURE)
        handleResult(res.toBytes())
}
```

`handleBTPMessage` Can be added to natively support the BTP protocol without a standalone connection.

```
external function handleBTPMessage(String _from, _svc String, Integer _sn, byte[] _msg) {
    handleMessage(_from, _msg)
}
 ```

`handleBTPError` Can be added to natively support the BTP protocol without a standalone connection.

```
external function handleBTPError(String _src, String _svc, BigInteger _sn, long _code, String _msg) {
    handleError(_sn)
}
```

```
internal function verifyProtocols(String srcNet, String[] protocols, byte[] data) returns boolean {
    source = getCaller()
    _hash = hash(data)
    if (protocols.length > 1):
        pendingMessages[_hash][source] = true
        for (protocol : protocols):
            if (!pendingMessages[_hash][protocol]):
                return false
        for (protocol : protocols):
            pendingMessages[_hash][protocol] = null
    else if (protocols.length == 1):
        assert source == protocols[0]
    else:
        assert source == defaultConnection[srcNet]

    return true
}
```

```
internal function handleRequest(String srcNet, byte[] data) {
    msgReq = CSMessageRequest.decode(data)
    if !verifyProtocols(srcNet, msgReq.protocolConfig, hash(data)):
        return

    reqId = getNextReqId()
    from = NetworkAddress(msgReq.from)
    assert from.net == srcNet
    emit CallMessage(msgReq.from, msgReq.to, msgReq.sn, reqId, msgReq.data)
    msgReq.data = hash(msgReq.data)
    proxyReqs[reqId] = msgReq
}
```
```
internal function handleReply(RollbackData rollback, CSMessageRequest reply) {
    // The reply must be from the same chain as the response originated from
    assert rollback.netTo == reply.from.netId
    // The protocols need to same as the reply
    rollback.protocols = reply.protocols

    reqId = getNextReqId()
    emit CallMessage(reply.from, reply.to, reply.sn, reqId, reply.data)
    reply.data = hash(reply.data)
    proxyReqs[reqId] = msgReq
}
```

```
internal function handleResult(data byte[]) {
        result = CSMessageResult.decode(data)
        resSn = result.sn
        req = rollbacks[resSn]

        if req == null:
            throw "CallRequest Not Found For {resSn}"

        if !verifyProtocols(req.netTo, req.protocolConfig, hash(data)):
            return

        emit ResponseMessage(resSn, result.getCode())
        switch result.getCode():
            case CSMessageResult.SUCCESS:
                if result.getMessage() != null:
                    handleReply(req, result.getMessage())

                rollbacks[resSn] = null
                successfulResponses[resSn] = 1
                break
            case CSMessageResult.FAILURE:
            default:
                assert req.rollback != null
                req.enabled = true
                rollbacks[resSn] = req
                emit RollbackMessage(resSn)
}
```

#### Message Execution

```
external function executeCall(Integer _reqId, byte[] _data) {
        req = proxyReqs[_reqId]
        assert req != null
        proxyReqs[_reqId] == null

        assert hash(_data) == req.data
        executeMessage(_reqId, r)
    }

```

Method where  message specific execution logic is handled.
```
internal function executeMessage(int reqId, CallRequest req) {
    switch (req.type) {
        case CallMessage.Type:
            tryExecute(reqId, req.from, req.data, req.protocols)
        case CallMessageWithRollback.Type:
            replyState = req
            code = tryExecute(reqId, req.from, req.data, req.protocols)
            replyState = null
            result = new CSMessageResult(req.sn, code, callReply)
            msg = CSMessage(CSMessage.RESULT, result.toBytes())
            callReply = null
            sn = req.sn.negate()
            if req.protocols == []:
                protocol = defaultConnection[from.net()]
                protocol->sendMessage(from.net(), "xcall-multi", sn, msg.toBytes())
            else:
                for (String protocol : req.protocols):
                    protocol->sendMessage(from.net(), "xcall-multi", sn, msg.toBytes())
        default:
            revert
    }
}
```

```
external function executeRollback(Integer _sn) {
    req = rollbacks.get(_sn)
    assert req != null
    assert req.enabled
    rollbacks[_sn] = null

    if req.protocols == []:
        req.to->handleCallMessage(getNetworkAddress(), req.rollback, req.protocols)
    else:
        req.to->handleCallMessage(getNetworkAddress(), req.rollback, req.protocols)

    emit RollbackExecuted(_sn)
}

```
```
internal function tryExecuteCall(int id, String from, byte[] data, String[] protocols) returns String {
    try:
        executeCall(id, from, data, protocols)
    catch Error as e:
         emit CallExecuted(id, CSMessageResult.FAILURE, e.message)
         return CSMessageResult.FAILURE

    return CSMessageResult.SUCCESS
}
```

```
internal function executeCall(int id, String from, byte[] data, String[] protocols) {
    if req.protocols == []:
        req.to->handleCallMessage(from, data)
    else:
        req.to->handleCallMessage(from, data, protocols)

    emit CallExecuted(id, CSMessageResult.SUCCESS, "")
}
```

```
internal function isReply(String netId, String[] sources) {
    if replyState != null:
       return replyState.fromNid == netid && replyState.protocols.equals(sources)

    return false
}
```

### Admin methods

```javascript
adminOnly function setAdmin(Address _admin){
    admin = _admin
}

adminOnly function setProtocolFeeHandler(Address _address){
    protocolFeeHandler = _address
}

adminOnly function setProtocolFee(Integer _protocolFee){
    protocolFee = _protocolFee
}

adminOnly function setDefaultConnection(String _nid, Address _connection){
    defaultConnection.set(_nid, _connection)
}
```

### Readonly methods

```
external readonly function  getNetworkAddress() returns String {
    return NetworkAddress(NID, this.address)
}
```

```
external readonly function  getNetworkId() returns String {
    return NID
}
```

```
external readonly function  getProtocolFee() returns Integer {
    return protocolFee
}
```

```
external readonly function getFee(String _net,
                                  boolean _rollback
                                  @Optional String[] _sources)
                                        returns Integer {

    if isReply(_net, sources) && !_rollback {
        return 0
    }
    fee = protocolFee
    if _sources == [] {
        return defaultConnection[_net]->getFee(_net, _rollback) + fee
    }


    for protocol in sources:
        fee += protocol->getFee(_net, _rollback)
    return fee
}
```

```
external readonly function  verifySuccess(Integer sn) returns boolean {
    return successfulResponses[sn]
}
```

## Differences from IIP52 xCall

Multi protocol xCall is based on the initial spec defined
in [IIP52](https://github.com/icon-project/IIPs/blob/master/IIPS/iip-52.md).

* Two new optional parameters are added in sendCallMessage: `_sources` and `destinations`.
  These parameters can be specified to choose the protocols to deliver the xCall message.
  If, for example, a dapp wanted to use BTP, they specify the address of BMC as the source and the address of BMC on a
  destination chain as destinations.

* Rollback guarantees.
  In IIP52, xCall rollback executions can only be tried once before removed, which can cause loss of data in case of
  failure.
  In xCall multi protocol, it can be retried until successful.

* Two-way message success verification.
  For all two-way messages, a response has to be relayed back since the fee has already been paid.
  This means that in most cases, a response with the result success is being relayed back.
  In xCall multi protocol, we store this success receipt so that it can be verified by dapps.

* BTP address has been replaced completely by Network Address.
  A BTP address is a Network Address as defined here with a `btp://` prefix.
  A Network Address in IIP52 refers to the Network ID in this document which might cause some confusion.

* The source of truth for a Network ID is now in xCall and not BMC.

* `_nsn` is removed from `CallMessageSent` event.

* Error messages are no longer relayed across chains in a response.

* `_msg` has been removed from ResponseMessage event. This is due to the removal of relaying the error messages.

* A message can now only be success or failure (1 or 0).
  In IIP52 a message can have many different error codes but was not used by dapps and the same behavior is not
  necessarily supported by all chains.

* MaxDataSize is defined on the whole payload rather than only user data
  This change was necessary to limit the size of the `_sources` and `destinations` parameters.

### Error Handling

...

## FAQs

...