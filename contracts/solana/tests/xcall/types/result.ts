import * as rlp from "rlp";

import { CSMessageType, CSResponseType } from "./message";

export class CSMessageResult {
  sequence_no: number;
  response_code: CSResponseType;
  data: Uint8Array | null;

  constructor(
    sequence_no: number,
    response_code: CSResponseType,
    data: Uint8Array | null
  ) {
    this.sequence_no = sequence_no;
    this.response_code = response_code;
    this.data = data;
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
