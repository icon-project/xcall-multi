import { Cl } from "@stacks/transactions";
import { describe, expect, it } from "vitest";

const accounts = simnet.getAccounts();
const address1 = accounts.get("wallet_1")!;

describe("RLP Encoding Tests", () => {
  it("encodes an empty string", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii("")],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'. Property 'buffer' does not exist on type 'TrueCV'.
    expect(result.result.buffer).toEqual(Cl.bufferFromHex("80").buffer);
  });

  it("encodes short strings", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii("dog")],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'. Property 'buffer' does not exist on type 'TrueCV'.
    expect(result.result.buffer).toEqual(Cl.bufferFromHex("83646f67").buffer);
  });

  it("encodes long strings", () => {
    const str = "Lorem ipsum dolor sit amet, consectetur adipisicing elit";
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii(str)],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'. Property 'buffer' does not exist on type 'TrueCV'.
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex("b8384c6f72656d20697073756d20646f6c6f722073697420616d65742c20636f6e7365637465747572206164697069736963696e6720656c6974").buffer
    );
  });

  it("encodes integers", () => {
    const tests = [
      { input: 0, expected: "80" },
      { input: 1, expected: "01" },
      { input: 16, expected: "10" },
      { input: 79, expected: "4f" },
      { input: 127, expected: "7f" },
      { input: 128, expected: "8180" },
      { input: 1000, expected: "8203e8" },
      { input: 100000, expected: "830186a0" }
    ];

    for (const test of tests) {
      const result = simnet.callReadOnlyFn(
        "rlp-encode",
        "encode-uint",
        [Cl.uint(test.input)],
        address1
      );
      // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'. Property 'buffer' does not exist on type 'TrueCV'.
      expect(result.result.buffer).toEqual(Cl.bufferFromHex(test.expected).buffer);
    }
  });

  it("encodes an empty list", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      [Cl.list([])],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'. Property 'buffer' does not exist on type 'TrueCV'.
    expect(result.result.buffer).toEqual(Cl.bufferFromHex("c0").buffer);
  });

  it("encodes string lists", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      [Cl.list([
        Cl.bufferFromAscii("dog"),
        Cl.bufferFromAscii("god"),
        Cl.bufferFromAscii("cat")
      ])],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'. Property 'buffer' does not exist on type 'TrueCV'.
    expect(result.result.buffer).toEqual(Cl.bufferFromHex("cc83646f6783676f6483636174").buffer);
  });
});