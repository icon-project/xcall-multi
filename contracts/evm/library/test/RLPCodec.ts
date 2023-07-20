import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { assert } from "chai";
import { ethers } from "hardhat";

const RLP_ENCODED_BYTES = [
    '0x9080000000000000000000000000000000',
    '0x7f',
    '0x820080',
    '0x8180',
    '0x00',
];

const RLP_DECODED_BYTES = [
    "0x80000000000000000000000000000000",
    "0x7f",
    "0x0080",
    "0x80",
    "0x00",
];

const RLP_DECODED_INT = [
    "-0x80000000000000000000000000000000",
    "0x7f",
    "0x80",
    "-0x80",
    "0x00",
];

const IntToBinary_Cases = [
    "-0x80000000000000000000000000000000",
    "0x7f",
    "0x80",
    "-0x80",
    "0x00",
    "-0x1",
];

const IntToBinary_Result = [
    "0x80000000000000000000000000000000",
    "0x7f",
    "0x0080",
    "0x80",
    "0x00",
    "0xff",
];

const UintToBinary_Cases = [
    "0x80000000000000000000000000000000",
    "0x7f",
    "0x80",
    "0x00",
    "0x1",
];

const UintToBinary_Result = [
    "0x0080000000000000000000000000000000",
    "0x7f",
    "0x0080",
    "0x00",
    "0x01",
];

describe('RLPn', () => {
    async function deployRLPCodecMock() {
        const RLPCodecMock = await ethers.getContractFactory("RLPCodecMock");
        const rlpCodec = await RLPCodecMock.deploy();
        return { rlpCodec };
    }

    describe( "rlp to int and int to rlp", () => {
        RLP_ENCODED_BYTES.forEach((item, idx) => {
            it ('int decode&encode for '+item, async () => {
                const { rlpCodec } = await loadFixture(deployRLPCodecMock);
                let v1 = await rlpCodec.rlpToInt(item);
                assert.equal(v1.toHexString(), RLP_DECODED_INT[idx]);
                let b2 = await rlpCodec.intToRLP(v1);
                assert.equal(b2, item);
            });
        });
    });

    describe( "intToBytes", () => {
        IntToBinary_Cases.forEach((item, idx) => {
            it ('intToBytes for '+item, async () => {
                const { rlpCodec } = await loadFixture(deployRLPCodecMock);
                let v1 = await rlpCodec.intToBytes(item);
                assert.equal(v1.toString(), IntToBinary_Result[idx]);
            });
        });
    });

    describe( "uintToBytes", () => {
        UintToBinary_Cases.forEach((item, idx) => {
            it ('uintToBytes for '+item, async () => {
                const { rlpCodec } = await loadFixture(deployRLPCodecMock);
                let v1 = await rlpCodec.uintToBytes(item);
                assert.equal(v1.toString(), UintToBinary_Result[idx]);
            });
        });
    });
});
