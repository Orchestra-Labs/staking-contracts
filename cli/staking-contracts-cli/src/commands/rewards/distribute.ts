import Base from '../base.js';
import { Args, Flags } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Coin } from '@cosmjs/proto-signing';

export default class Distribute extends Base {
  static description = 'Distribute rewards to stakers';

  static examples = [];

  static args = {
    contractAddress: Args.string({description: "Address of the rewards contract", required: true}),
  }

  static flags = {
    rewardsDenom: Flags.string({char: 'd', description: "Denom unit of the rewards token", required: false, default: "note"}),
    rewardsExponent: Flags.integer({char: 'e', description: "Exponent of the rewards token", required: false, default: 6}),
    amount: Flags.string({char: 'a', description: "Amount of tokens to stake", required: true}),
  }

  async run(): Promise<void> {
    const { args, flags } = await this.parse(Distribute);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});

    const executeMsg = {
      "distribute_rewards": {}
    };
    const funds = [{ denom: flags.rewardsDenom, amount: flags.amount }] as Coin[];
    this.log("Fund: ", funds);
    const executeResult = await client.execute(signerAddress, args.contractAddress, executeMsg, "auto", `Distribute ${flags.amount} ${flags.denom}`, funds);
    this.log("Transaction Hash: ", executeResult.transactionHash);
  }
}