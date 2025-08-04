import Base from '../base.js';
import { Args, Flags } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

export default class Initialize extends Base {
  static description = 'Initialize vesting contract for rewards distribution';
  static examples = [];

  static args = {
    codeId: Args.integer({description: "Code ID of the contract", required: true}),
  }

  static flags = {
    label: Flags.string({description: "Label for the contract", required: true}),
    memo: Flags.string({description: "Memo to include in the transaction", default: "", required: false}),
  }

  async run(): Promise<void> {
    const { args, flags } = await this.parse(Initialize);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});
    const initMsg = {
      owner: signerAddress,
    };
    this.log("Initializing vesting rewards orchestrator with: ", initMsg);
    const instantiateResult = await client.instantiate(signerAddress, args.codeId, initMsg, flags.label, 'auto', {
      memo: flags.memo,
      admin: signerAddress,
    });
    this.log("Contract Instantiated with address: ", instantiateResult.contractAddress);
    this.log("Transaction Hash: ", instantiateResult.transactionHash);
  }
}