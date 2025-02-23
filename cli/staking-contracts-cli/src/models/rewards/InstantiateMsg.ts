import { DenomUnit } from '../DenomUnit.js';

export interface RewardsDistribution {
  denom: DenomUnit;
  weight: number;
}

export interface InstantiateMsg {
  owner?: string;
  staking_orchestrator_addr: string;
  reward_token: DenomUnit;
  rewards_distribution: RewardsDistribution[];
}