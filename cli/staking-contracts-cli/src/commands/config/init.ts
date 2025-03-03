import { Args, Command, Flags } from '@oclif/core';
import path from 'path';
import { ChainConfig } from '../../models/ChainConfig.js';
import fs from 'fs-extra';

export default class Init extends Command {
  static description = 'Initialize cli configuration';

  static examples = [
    `$ symphony-staking-cli config init "word1 word2 ..." --gasPrice 0.1note --rpcEndpoint "localhost:443" --prefix symphony
`,
  ];

  static args = {
    mnemonic: Args.string({name: "mnemonic", description: "Mnemonic of your wallet", required: true})
  };

  static flags = {
    gasPrice: Flags.string({char: "g", description: "Gas price string value (e.g.: 0.025ujuno)", required: true}),
    rpcEndpoint: Flags.string({char: "e", description: "RPC endpoint of the chain", required: true}),
    prefix: Flags.string({char: "p", description: "Addresses prefix (e.g.: juno)", required: true}),
  }

  async run(): Promise<void> {
    const {args, flags} = await this.parse(Init);

    const configPath = path.join(this.config.configDir, 'config.json');

    const chainConfig: ChainConfig = {
      wallet: args.mnemonic,
      gasPrice: flags.gasPrice,
      rpcEndpoint: flags.rpcEndpoint,
      prefix: flags.prefix,
    }
    await fs.ensureFile(configPath);
    await fs.writeJson(configPath, chainConfig, {spaces: 2, encoding: 'utf8'});
    this.log("Configuration saved to", configPath);
  }
}