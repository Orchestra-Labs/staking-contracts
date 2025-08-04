import Base from '../base.js';
import { Args, Flags } from '@oclif/core';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

export default class GetVestingContract extends Base {
  static description = 'Setup rewards vesting ';

  static examples = [];

  static args = {
    contractAddress: Args.string({description: "Address of the rewards contract", required: true}),
  }

  static flags = {
    recipient: Flags.string({char: 'p', description: "Recipient Address", required: true}),
  }

  async run(): Promise<void> {
    const { args, flags } = await this.parse(GetVestingContract);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    const client = await SigningCosmWasmClient
      .connect(this.chainConfig!.rpcEndpoint);

    const queryMsg = {
      "vesting_contract": {
        "address": flags.recipient
      }
    }

    const response: {
      found: boolean,
      address: string,
      contract_address?: string
    } = await client.queryContractSmart(args.contractAddress,queryMsg);

    if (response.found) {
      this.log("Vesting contract found at address: ", response.contract_address);
    } else {
      this.log("No vesting contract found for the given address.");
    }

    const infoQueryMsg = {
      "info": {}
    };

    const infoResponse = await client.queryContractSmart(response.contract_address!, infoQueryMsg);

    this.log("Vesting Contract Info: ", infoResponse);

    const vestedQueryMsg = {
      "vested": {
        "t": undefined
      }
    }

    const vestedResponse = await client.queryContractSmart(response.contract_address!, vestedQueryMsg);

    this.log("Vested Amount: ", vestedResponse);
  }
}