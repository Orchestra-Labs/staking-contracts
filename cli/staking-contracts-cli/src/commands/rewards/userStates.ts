import Base from '../base.js';
import { Args } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

export default class UserStates extends Base {
  static description = 'Get the rewards states of all users';

  static examples = [];

  static args = {
    contractAddress: Args.string({description: "Address of the rewards contract", required: true}),
  }

  async run(): Promise<void> {
    const { args } = await this.parse(UserStates);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});

    const queryMsg = {
      "all_user_states": {}
    }
    const userStates = await client.queryContractSmart(args.contractAddress, queryMsg);
    this.log("User States: ", userStates);
  }

}