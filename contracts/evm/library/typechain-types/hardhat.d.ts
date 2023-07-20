/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */

import { ethers } from "ethers";
import {
  FactoryOptions,
  HardhatEthersHelpers as HardhatEthersHelpersBase,
} from "@nomiclabs/hardhat-ethers/types";

import * as Contracts from ".";

declare module "hardhat/types/runtime" {
  interface HardhatEthersHelpers extends HardhatEthersHelpersBase {
    getContractFactory(
      name: "IBMC",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.IBMC__factory>;
    getContractFactory(
      name: "IBMV",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.IBMV__factory>;
    getContractFactory(
      name: "IBSH",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.IBSH__factory>;
    getContractFactory(
      name: "ICallService",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.ICallService__factory>;
    getContractFactory(
      name: "ICallServiceReceiver",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.ICallServiceReceiver__factory>;
    getContractFactory(
      name: "IDefaultCallServiceReceiver",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.IDefaultCallServiceReceiver__factory>;
    getContractFactory(
      name: "IConnection",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.IConnection__factory>;
    getContractFactory(
      name: "IDefaultCallServiceReceiver",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.IDefaultCallServiceReceiver__factory>;
    getContractFactory(
      name: "RLPCodecMock",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.RLPCodecMock__factory>;
    getContractFactory(
      name: "Integers",
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<Contracts.Integers__factory>;

    getContractAt(
      name: "IBMC",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.IBMC>;
    getContractAt(
      name: "IBMV",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.IBMV>;
    getContractAt(
      name: "IBSH",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.IBSH>;
    getContractAt(
      name: "ICallService",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.ICallService>;
    getContractAt(
      name: "ICallServiceReceiver",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.ICallServiceReceiver>;
    getContractAt(
      name: "IDefaultCallServiceReceiver",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.IDefaultCallServiceReceiver>;
    getContractAt(
      name: "IConnection",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.IConnection>;
    getContractAt(
      name: "IDefaultCallServiceReceiver",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.IDefaultCallServiceReceiver>;
    getContractAt(
      name: "RLPCodecMock",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.RLPCodecMock>;
    getContractAt(
      name: "Integers",
      address: string,
      signer?: ethers.Signer
    ): Promise<Contracts.Integers>;

    // default types
    getContractFactory(
      name: string,
      signerOrOptions?: ethers.Signer | FactoryOptions
    ): Promise<ethers.ContractFactory>;
    getContractFactory(
      abi: any[],
      bytecode: ethers.utils.BytesLike,
      signer?: ethers.Signer
    ): Promise<ethers.ContractFactory>;
    getContractAt(
      nameOrAbi: string | any[],
      address: string,
      signer?: ethers.Signer
    ): Promise<ethers.Contract>;
  }
}
