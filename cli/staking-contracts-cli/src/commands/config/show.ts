import Base from '../base.js';

export default class Show extends Base {
  static description = 'Show current configuration';

  async run() {
    if (!this.chainConfig) {
      this.warn("Configuration not initialized");
      return;
    }
    this.log("Current configuration:");
    this.log("Wallet: ", this.chainConfig.wallet);
    this.log("Gas Price: ", this.chainConfig.gasPrice);
    this.log("RPC Endpoint: ", this.chainConfig.rpcEndpoint);
    this.log("Prefix: ", this.chainConfig.prefix);
  }
}