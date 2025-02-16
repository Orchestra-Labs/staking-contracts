import Base from '../base.js';
import { Args } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import fs from 'fs';

export default class Upload extends Base {
  static description = 'Upload a new Staking WASM artifact';

  static examples = [];

  static args = {
    artifact: Args.string({name: "artifact", description: "Path to the artifact wasm file of the Staking contract", required: true})
  }

  static flags = {}

  async run(): Promise<void> {
    const {args} = await this.parse(Upload);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});
    const wasm = fs.readFileSync(args.artifact);
    const uploadResult = await client.upload(signerAddress, wasm, 'auto', "Staking Contract");
    this.log("Contract WASM Uploaded with code ID: ", uploadResult.codeId);
    this.log("Transaction Hash: ", uploadResult.transactionHash);
  }
}