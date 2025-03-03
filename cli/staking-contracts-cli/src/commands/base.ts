import { Command } from '@oclif/core';
import { ChainConfig } from '../models/ChainConfig.js';
import path from 'path';
import fs from 'fs-extra';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';

export default abstract class extends Command {
    protected chainConfig?: ChainConfig;

    async init() {
        const configPath = path.join(this.config.configDir, 'config.json');
        if (await this.configExists(configPath)) {
            const userConfig: ChainConfig = await fs.readJson(configPath, 'utf8');
            if (userConfig.wallet) {
                this.log('User config:', configPath);
                this.chainConfig = userConfig;
            } else {
                this.warn(`Config file on path ${configPath} not initialized, run 'symphony-staking-cli config init' before using other commands`);
                this.exit(-1);
            }
        }
    }

    protected async configExists(configPath: string): Promise<boolean> {
        return await fs.pathExists(configPath);
    }

    protected async getWalletFromMnemonic() : Promise<DirectSecp256k1HdWallet> {
        return await DirectSecp256k1HdWallet.fromMnemonic(this.chainConfig!.wallet, {prefix: this.chainConfig!.prefix});
    }

    protected get gasPrice() : GasPrice {
        return GasPrice.fromString(this.chainConfig!.gasPrice);
    }
}