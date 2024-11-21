import { Cl } from "@stacks/transactions";
import { describe, expect, it } from "vitest";

const accounts = simnet.getAccounts();
const address1 = accounts.get("wallet_1")!;

describe("RLP Encoding Tests", () => {
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

  it("encodes string with smaller bytes length", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii("soroban-rlp")],
      address1
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex("8b736f726f62616e2d726c70").buffer
    );
  });

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
      Cl.bufferFromHex("b90097" +
        Buffer.from(str).toString('hex')).buffer
    );
  });

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
      Cl.bufferFromHex("ca85616c69636583626f62").buffer
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer.length).toEqual(11);
  });

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

  it("encodes list with smaller bytes", () => {
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      [Cl.list([
        Cl.buffer(simnet.callReadOnlyFn(
          "rlp-encode",
          "encode-uint",
          [Cl.uint(4294967295)],
          address1
          // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        ).result.buffer),
        Cl.buffer(simnet.callReadOnlyFn(
          "rlp-encode",
          "encode-string",
          [Cl.stringAscii("soroban-rlp")],
          address1
          // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        ).result.buffer)
      ])],
      address1
    );

    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex("d2" +
        "8500ffffffff" +
        "8b736f726f62616e2d726c70"
      ).buffer
    );
  });

  it("encodes list with longer bytes", () => {
    const u8Value = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(245)],
      address1
    );
  
    const u32Value = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(24196199)],
      address1
    );
  
    const u64Value = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(103921887687475199n)],
      address1
    );
  
    const u128Value = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(180593171625979951495805181356371083263n)],
      address1
    );
  
    const strings = [
      "Integer quis auctor elit sed vulputate mi sit.",
      "Tincidunt nunc pulvinar sapien et ligula"
    ];
    
    const encodedStrings = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      [Cl.list(strings.map(str => 
        Cl.buffer(simnet.callReadOnlyFn(
          "rlp-encode",
          "encode-string",
          [Cl.stringAscii(str)],
          address1
        // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        ).result.buffer)
      ))],
      address1
    );
  
    const lastString = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii("Sed adipiscing diam donec adipiscing tristique")],
      address1
    );
  
    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      [Cl.list([
        // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        Cl.buffer(u8Value.result.buffer),
        // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        Cl.buffer(u32Value.result.buffer),
        // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        Cl.buffer(u64Value.result.buffer),
        // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        Cl.buffer(u128Value.result.buffer),
        // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        Cl.buffer(encodedStrings.result.buffer),
        // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
        Cl.buffer(lastString.result.buffer)
      ])],
      address1
    );
  
    const expectedHex = 
      "f900ae" +
      "81f5" +
      "850001713467" +
      "89001713467ffff" + "ffff" +
      "910087dcfacd87982736cdefcdefff" + "ffffff" +
      "f90058" +
      "ae" + Buffer.from(strings[0]).toString('hex') +
      "a8" + Buffer.from(strings[1]).toString('hex') +
      "ae" + Buffer.from("Sed adipiscing diam donec adipiscing tristique").toString('hex');
  
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex(expectedHex).buffer
    );
    // @ts-ignore: Property 'buffer' does not exist on type 'ClarityValue'.
    expect(result.result.buffer.length).toEqual(177);
  });

  it("encodes cross chain message", () => {
    const sourceContract = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii("0x1.icon/cxc5d40fd74995bed473e5d1b259dbc6015273ffc5")],
      address1
    );

    const destAddress = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii("archway1rv84n8yczcug4rpwx028tkvwm5gerluzs28uhn")],
      address1
    );

    const sn = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(11580)],
      address1
    );

    const msgType = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-uint",
      [Cl.uint(0)],
      address1
    );

    const emptyData = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-string",
      [Cl.stringAscii("")],
      address1
    );

    const destList = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      [Cl.list([
        Cl.buffer(simnet.callReadOnlyFn(
          "rlp-encode",
          "encode-string",
          [Cl.stringAscii("archway1lvmx2u6f47n8yr0dg7fangur2l72nwxxklasqyal2fhtpyw9uxfqmudel8")],
          address1
          // @ts-ignore
        ).result.buffer)
      ])],
      address1
    );

    const innerMessage = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      [Cl.list([
        // @ts-ignore
        Cl.buffer(sourceContract.result.buffer),
        // @ts-ignore
        Cl.buffer(destAddress.result.buffer),
        // @ts-ignore
        Cl.buffer(sn.result.buffer),
        // @ts-ignore
        Cl.buffer(msgType.result.buffer),
        // @ts-ignore
        Cl.buffer(emptyData.result.buffer),
        // @ts-ignore
        Cl.buffer(destList.result.buffer)
      ])],
      address1
    );

    const result = simnet.callReadOnlyFn(
      "rlp-encode",
      "encode-arr",
      [Cl.list([
        Cl.buffer(simnet.callReadOnlyFn(
          "rlp-encode",
          "encode-uint",
          [Cl.uint(1)],
          address1
        // @ts-ignore
        ).result.buffer),
        // @ts-ignore
        Cl.buffer(innerMessage.result.buffer)
      ])],
      address1
    );

    // @ts-ignore
    expect(result.result.buffer).toEqual(
      Cl.bufferFromHex("f8b301b8b0f8aeb33078312e69636f6e2f637863356434306664373439393562656434373365356431623235396462633630313532373366666335ae6172636877617931727638346e3879637a6375673472707778303238746b76776d356765726c757a73323875686e822d3c0080f844b84261726368776179316c766d783275366634376e3879723064673766616e677572326c37326e7778786b6c61737179616c3266687470797739757866716d7564656c38").buffer
    );
  });
});