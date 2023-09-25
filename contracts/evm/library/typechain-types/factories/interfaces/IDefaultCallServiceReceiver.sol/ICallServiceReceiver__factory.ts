/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */

import { Contract, Signer, utils } from "ethers";
import type { Provider } from "@ethersproject/providers";
import type {
  ICallServiceReceiver,
  ICallServiceReceiverInterface,
} from "../../../interfaces/IDefaultCallServiceReceiver.sol/ICallServiceReceiver";

const _abi = [
  {
    inputs: [
      {
        internalType: "string",
        name: "_from",
        type: "string",
      },
      {
        internalType: "bytes",
        name: "_data",
        type: "bytes",
      },
    ],
    name: "handleCallMessage",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
] as const;

export class ICallServiceReceiver__factory {
  static readonly abi = _abi;
  static createInterface(): ICallServiceReceiverInterface {
    return new utils.Interface(_abi) as ICallServiceReceiverInterface;
  }
  static connect(
    address: string,
    signerOrProvider: Signer | Provider
  ): ICallServiceReceiver {
    return new Contract(
      address,
      _abi,
      signerOrProvider
    ) as ICallServiceReceiver;
  }
}