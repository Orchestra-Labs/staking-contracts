import Base from './base.js';
import { Flags } from '@oclif/core';
import { getSigningSymphonyClient, symphony } from '@orchestra-labs/symphonyjs';

const { swapSend } = symphony.market.v1beta1.MessageComposer.withTypeUrl;

export default class Swap extends Base {
  static description = 'Swap tokens';

  static examples = [];

  static flags = {
    amount: Flags.string({char: 'a', description: "Amount of tokens to swap", required: true}),
    sourceDenom: Flags.string({char: 's', description: "Denom unit of the source token", required: true}),
    targetDenom: Flags.string({char: 't', description: "Denom unit of the target token", required: true}),
  }

  static args = {}

  async run(): Promise<void> {
    const {flags} = await this.parse(Swap);
    const wallet = await this.getWalletFromMnemonic();
    const [{ address: signerAddress }] = await wallet.getAccounts();

    this.log("Signer Address: ", signerAddress);

    const client = await getSigningSymphonyClient({
      rpcEndpoint: this.chainConfig!.rpcEndpoint,
      signer: wallet,
    });

    const swapMsg = swapSend({
      fromAddress: signerAddress,
      toAddress: signerAddress,
      offerCoin: {
        denom: flags.sourceDenom,
        amount: flags.amount,
      },
      askDenom: flags.targetDenom,
    });

    const signAndBroadcastPromise = await client.signAndBroadcast(
      signerAddress!,
      [swapMsg],
      {
        amount: [{ denom: 'note', amount: '1000000' }],
        gas: '100000',
      },
    );

    this.log("Transaction Hash: ", signAndBroadcastPromise.transactionHash);
    this.log("signAndBroadcastPromise: ", signAndBroadcastPromise);
  }
}