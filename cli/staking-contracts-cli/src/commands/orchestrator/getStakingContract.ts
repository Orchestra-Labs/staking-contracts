import Base from '../base.js';
import { Args, Flags } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { QueryStakingContractByDenom } from '../../models/orchestrator/QueryStakingContractByDenom.js';
import { StakingContractByDenomResponse } from '../../models/orchestrator/StakingContractByDenomResponse.js';

export default class GetStakingContract extends Base {
  static description = 'Get a staking contract by denom';

  static examples = [];

  static flags = {
    denom: Flags.string({char: 'd', description: "Denom unit of the staking token", required: true}),
  }

  static args = {
    contractAddress: Args.string({name: "contractAddress", description: "Contract address of the orchestrator contract", required: true})
  }

  async run(): Promise<void> {
    const {args, flags} = await this.parse(GetStakingContract);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});
    const queryMsg = {
      "staking_contract_by_denom": {
        denom: flags.denom,
      } as QueryStakingContractByDenom
    }
    const stakingContractResult: StakingContractByDenomResponse = await client.queryContractSmart(args.contractAddress, queryMsg);
    this.log("Staking Contract address: ", stakingContractResult.registered_contract.address);
  }
}