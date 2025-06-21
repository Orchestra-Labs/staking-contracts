import Base from '../base.js';
import { Args, Flags } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Coin } from '@cosmjs/proto-signing';
import fs from 'fs-extra';
import { VestingData } from '../../models/Vesting/VestingData.js';

export default class SetupVesting extends Base {
  static description = 'Setup rewards vesting ';

  static examples = [];

  static args = {
    contractAddress: Args.string({description: "Address of the rewards contract", required: true}),
  }

  static flags = {
    vestingPath: Flags.string({char: 'p', description: "Path to the vesting file", required: true}),
  }

  async run(): Promise<void> {
    const { args, flags } = await this.parse(SetupVesting);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});

    const rewardsSetup: VestingData = await fs.readJSON(flags.vestingPath);
    this.log(rewardsSetup.configuration.start_time);

    rewardsSetup.configuration.start_time = (Date.parse(rewardsSetup.configuration.start_time) * 1_000_000).toString(); // Convert to nanoseconds

    this.log("Vesting configuration: ", rewardsSetup.configuration);

    const executeMsg = {
      "batch_vesting": rewardsSetup
    };
    const totalAmount: BigInt = rewardsSetup.records.map(x => BigInt(x.amount)).reduce((acc, element) => {
      return acc + BigInt(element);
    });
    const funds = [{ denom: rewardsSetup.configuration.denom, amount: totalAmount.toString() }] as Coin[];
    this.log("Fund: ", funds);
    const executeResult = await client.execute(signerAddress, args.contractAddress, executeMsg, "auto", `Distribute ${flags.amount} ${flags.denom}`, funds);
    this.log("Transaction Hash: ", executeResult.transactionHash);
  }
}