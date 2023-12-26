export interface LinkEdge {
  capacity: number;
  usage: number;
  travel_time_sum: number;
  last_unrestricted_update: number;
  last_restricted_update: number;
  next_edge: number;
}

export interface LinkNode {
  xy: number;
  supply: number;
  demand: number;
  station: number;
  last_update: number;
  edges: LinkEdge[];
}

export interface LinkGraph {
  cargo: number;
  nodes: LinkNode[];
}