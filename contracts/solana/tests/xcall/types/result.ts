import * as rlp from "rlp";

import { CSMessageType, CSResponseType } from "./message";

export class CSMessageResult {
  sequence_no: number;
  response_code: CSResponseType;
  data: Uint8Array;

  new(sequence_no: number, response_code: CSMessageType, data: Uint8Array) {
    return {
      sequence_no,
      response_code,
      data,
    };
  }

  encode() {
    let rlpInput: rlp.Input = [
      this.sequence_no,
      this.response_code,
      Buffer.from(this.data),
    ];

    return rlp.encode(rlpInput);
  }
}
