import Base from '../base.js';
import { Args, Flags } from '@oclif/core';
import fs from 'fs-extra';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { InstantiateMsg, RewardsDistribution } from '../../models/rewards/InstantiateMsg.js';

export default class Initialize extends Base {
  static description = 'Initialize rewards for staking contract';
  static examples = [];

  static args = {
    codeId: Args.integer({description: "Code ID of the contract", required: true}),
  }

  static flags = {
    label: Flags.string({description: "Label for the contract", required: true}),
    memo: Flags.string({description: "Memo to include in the transaction", default: "", required: false}),
    orchestratorAddr: Flags.string({char: 'o', description: "Orchestrator address", required: true}),
    rewardsDenom: Flags.string({char: 'd', description: "Denom unit of the rewards token", required: false, default: "note"}),
    rewardsExponent: Flags.integer({char: 'e', description: "Exponent of the rewards token", required: false, default: 6}),
    distributionPath: Flags.file({char: 'p', description: "Path to the rewards distribution file", required: true}),
  }

  async run(): Promise<void> {
    const { args, flags } = await this.parse(Initialize);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();
    const rewardsDistribution: RewardsDistribution[] = await fs.readJSON(flags.distributionPath);
    this.log("Rewards Distributions: ", rewardsDistribution);

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});
    const initMsg: InstantiateMsg = {
      reward_token: {
        denom: flags.rewardsDenom,
        exponent: flags.rewardsExponent,
        aliases: [],
      },
      rewards_distribution: rewardsDistribution,
      staking_orchestrator_addr: flags.orchestratorAddr,
    };
    this.log("Initializing rewards with: ", initMsg);
    const instantiateResult = await client.instantiate(signerAddress, args.codeId, initMsg, flags.label, 'auto', {
      memo: flags.memo,
      admin: signerAddress,
    });
    this.log("Contract Instantiated with address: ", instantiateResult.contractAddress);
    this.log("Transaction Hash: ", instantiateResult.transactionHash);
  }
}