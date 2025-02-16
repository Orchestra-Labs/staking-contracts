import Base from '../base.js';
import { Args, Flags } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { InstantiateMsg } from '../../models/orchestrator/InstantiateMsg.js';

export default class Instantiate extends Base {
  static description = 'Instantiate a new Staking contract';

  static examples = [];

  static args = {
    codeId: Args.integer({description: "Code ID of the contract", required: true}),
  }

  static flags = {
    label: Flags.string({description: "Label for the contract", required: true}),
    memo: Flags.string({description: "Memo to include in the transaction", default: "", required: false}),
  }

  async run(): Promise<void> {
    const {args, flags} = await this.parse(Instantiate);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});
    const initMsg: InstantiateMsg = {};
    const instantiateResult = await client.instantiate(signerAddress, args.codeId, initMsg, flags.label, 'auto', {
      memo: flags.memo,
      admin: signerAddress,
    });
    this.log("Contract Instantiated with address: ", instantiateResult.contractAddress);
    this.log("Transaction Hash: ", instantiateResult.transactionHash);
  }
}