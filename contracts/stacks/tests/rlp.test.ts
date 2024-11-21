import { Cl } from "@stacks/transactions";
import { describe, expect, it } from "vitest";

const accounts = simnet.getAccounts();
const address1 = accounts.get("wallet_1")!;

describe("RLP Soroban Compatibility Tests", () => {
  // test_encode_u8()
  it("encodes u8", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(100)],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(Cl.bufferFromHex("64").buffer);
  });

  // test_encode_u32()
  it("encodes u32", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(2000022458)],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(Cl.bufferFromHex("85007735EBBA").buffer);
  });

  // test_encode_u64()
  it("encodes u64", () => {
    let result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(1999999999999999999n)],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(Cl.bufferFromHex("89001BC16D674EC7FFFF").buffer);

    result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(199999999)],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(Cl.bufferFromHex("85000BEBC1FF").buffer);
  });

  // test_encode_u128()
  it("encodes u128", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint", 
      [Cl.uint(199999999999999999999999999999999999999n)],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex("910096769950B50D88F41314447FFFFFFFFF").buffer
    );
  });

  // test_encode_string_with_smaller_bytes_length()
  it("encodes string with smaller bytes length", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii("soroban-rlp")],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex("8b736f726f62616e2d726c70").buffer // 8b = 139
    );
  });

  // test_encode_string_with_larger_bytes_length()
  it("encodes string with larger bytes length", () => {
    const str = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s";
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii(str)],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex("b90097" + // 185 = 0xb9, followed by length bytes 0x00 0x97
        Buffer.from(str).toString('hex')).buffer
    );
  });

  // test_encode_list_empty()
  it("encodes empty list", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode", 
      "encode-arr",
      [Cl.list([])],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(Cl.bufferFromHex("c0").buffer);
  });

  // test_encode_strings()
  it("encodes strings", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      [Cl.list([
        Cl.buffer(simnet.callReadOnlyFn(
          "rlp-encode",
          "encode-string",
          [Cl.stringAscii("alice")],
          address1
          // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        ).result.buffer),
        Cl.buffer(simnet.callReadOnlyFn(
          "rlp-encode",
          "encode-string",
          [Cl.stringAscii("bob")],
          address1
          // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        ).result.buffer)
      ])],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex("ca85616c69636583626f62").buffer // Length prefix 0xca = 192 + 10
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer.length).toEqual(11);
  });

  // test_encode_strings_with_longer_bytes()
  it("encodes strings with longer bytes", () => {
    const strings = [
      "It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout.",
      "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
      "Egestas maecenas pharetra convallis posuere morbi. Velit laoreet id donec ultrices tincidunt arcu non sodales neque."
    ];

    const encodedStrings = strings.map(str => 
      simnet.callReadOnlyFn(
        "rlp-encode",
        "encode-string",
        [Cl.stringAscii(str)],
        address1
      )
    );

    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
      [Cl.list(encodedStrings.map(e => Cl.buffer(e.result.buffer)))],
      address1
    );

    // From Soroban test:
    // rlp_byte = 0xf7 + 3
    // Followed by length bytes [0x00, 0x01, 0x74]
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex("fa000174" + 
        "b9007c" + Buffer.from(strings[0]).toString('hex') +
        "b9007b" + Buffer.from(strings[1]).toString('hex') +
        "b90074" + Buffer.from(strings[2]).toString('hex')
      ).buffer
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer.length).toEqual(376);
  });
});