import Base from '../base.js';
import { Args, Flags } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Coin } from '@cosmjs/proto-signing';
import { StakedBalanceAtHeightResponse } from '../../models/staking/StakedBalanceAtHeightResponse.js';

export default class Stake extends Base {
  static description = 'Stake tokens to a staking contract';
  static examples = [];

  static flags = {
    amount: Flags.string({char: 'a', description: "Amount of tokens to stake", required: true}),
    denom: Flags.string({char: 'd', description: "Denom unit of the staking token", required: true}),
  }

  static args = {
    contractAddress: Args.string({name: "contractAddress", description: "Contract address of the staking contract", required: true}),
  }

  async run(): Promise<void> {
    const {args, flags} = await this.parse(Stake);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});
    const executeMsg = {
      "stake": {}
    }
    const funds = [{ denom: flags.denom, amount: flags.amount }] as Coin[];
    const executeResult = await client.execute(signerAddress, args.contractAddress, executeMsg, "auto", `Stake ${flags.amount} ${flags.denom}`, funds);
    this.log("Transaction Hash: ", executeResult.transactionHash);
    const queryMsg = {
      "staked_balance_at_height": {
        address: signerAddress,
      }
    }

    const stakedBalance: StakedBalanceAtHeightResponse = await client.queryContractSmart(args.contractAddress, queryMsg);
    this.log("Staked Balance: ", stakedBalance.balance);
  }
}