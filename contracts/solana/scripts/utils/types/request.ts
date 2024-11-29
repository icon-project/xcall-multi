import * as rlp from "rlp";

import { MessageType } from ".";

export class CSMessageRequest {
  from: string;
  to: string;
  sequence_no: number;
  msg_type: MessageType;
  data: Uint8Array;
  protocols: string[];

  constructor(
    from: string,
    to: string,
    sequence_no: number,
    msg_type: MessageType,
    data: Uint8Array,
    protocols: string[]
  ) {
    this.from = from;
    this.to = to;
    this.sequence_no = sequence_no;
    this.msg_type = msg_type;
    this.data = data;
    this.protocols = protocols;
  }

  encode() {
    let rlpInput: rlp.Input = [
      this.from,
      this.to,
      this.sequence_no,
      this.msg_type,
      Buffer.from(this.data),
      this.protocols,
    ];

    return rlp.encode(rlpInput);
  }
}
