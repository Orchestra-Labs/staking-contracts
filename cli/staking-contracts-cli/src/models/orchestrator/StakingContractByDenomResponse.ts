import { DenomUnit } from '../DenomUnit.js';

export interface StakingContractByDenomResponse {
  denom: string,
  registered_contract: RegisteredContract,
}

export interface RegisteredContract {
  address: string,
  token: DenomUnit,
}