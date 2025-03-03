import Base from '../base.js';
import { Args } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

export default class Info extends Base {
  static description = 'Get information about a deployed contract';

  static examples = [];

  static args = {
    codeId: Args.string({description: "Code ID of the contract", required: true}),
  }

  async run(): Promise<void> {
    const {args} = await this.parse(Info);
    const wallet = await this.getWalletFromMnemonic();
    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});
    const contractInfo = await client.getCodeDetails(parseInt(args.codeId));
    this.log("Contract Info: ", contractInfo);
  }
}