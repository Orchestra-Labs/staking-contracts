import Base from '../base.js';
import { Args, Flags } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { CreateStakingContractMsg } from '../../models/orchestrator/CreateStakingContractMsg.js';
import { QueryStakingContractByDenom } from '../../models/orchestrator/QueryStakingContractByDenom.js';
import { StakingContractByDenomResponse } from '../../models/orchestrator/StakingContractByDenomResponse.js';

export default class CreateStakingContract extends Base {
  static description = 'Create a new staking contract';

  static examples = [];

  static args = {
    contractAddress: Args.string({name: "contractAddress", description: "Contract address of the orchestrator contract", required: true})
  }

  static flags = {
    stakingContractCodeId: Flags.integer({char: 'c', description: "Staking contract codeID", required: true}),
    denom: Flags.string({char: 'd', description: "Denom unit of the staking token", required: true}),
    tokenExponent: Flags.integer({char: 'e', description: "Exponent of the staking token", required: true}),
    unbondingPeriod: Flags.integer({char: 'u', description: "Unbounding period in seconds", required: false}),
  }

  async run(): Promise<void> {
    const {args, flags} = await this.parse(CreateStakingContract);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connectWithSigner(this.chainConfig!.rpcEndpoint, wallet, {"gasPrice": this.gasPrice});
    const createStakingContractMsg: CreateStakingContractMsg = {
      code_id: flags.stakingContractCodeId,
      denom_unit: {
        denom: flags.denom,
        exponent: flags.tokenExponent,
        aliases: [],
      },
      unbonding_period: undefined,
      owner: signerAddress,
    };
    const executeMsg = {
      "create_staking_contract": createStakingContractMsg,
    }

    const executeResult = await client.execute(signerAddress, args.contractAddress, executeMsg, "auto", `Symphony Native Staking Contract ${flags.denom}`, []);
    this.log("Transaction Hash: ", executeResult.transactionHash);
    const queryMsg = {
      "staking_contract_by_denom": {
        denom: flags.denom,
      } as QueryStakingContractByDenom
    }
    const stakingContractResult: StakingContractByDenomResponse = await client.queryContractSmart(args.contractAddress, queryMsg);
    this.log("Staking Contract Instantiated with address: ", stakingContractResult.registered_contract.address);
  }

}