export type Schedule =
    | "saturating_linear"
  | { piecewise_linear: [string, string][] };

export type Timestamp = string;

export interface VestingConfiguration {
  contract_code_id: number; // uint64
  denom: string;
  description_prefix: string;
  duration: number; // uint64
  schedule: Schedule;
  start_time: Timestamp;
  title_prefix: string;
}

export interface VestingRecord {
  address: string;
  amount: string;
}

export interface VestingData {
  configuration: VestingConfiguration;
  records: VestingRecord[];
}