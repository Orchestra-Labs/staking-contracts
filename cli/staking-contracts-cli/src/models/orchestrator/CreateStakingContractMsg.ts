import { DenomUnit } from '../DenomUnit.js';
import { Duration } from '../Duration.js';

export interface CreateStakingContractMsg {
  code_id: number;
  denom_unit: DenomUnit;
  unbonding_period?: Duration;
  owner?: string;
}