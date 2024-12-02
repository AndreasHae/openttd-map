// SLE_VAR(Edge, capacity,                 SLE_UINT32),
// SLE_VAR(Edge, usage,                    SLE_UINT32),
// SLE_CONDVAR(Edge, travel_time_sum,          SLE_UINT64, SLV_LINKGRAPH_TRAVEL_TIME, SL_MAX_VERSION),
// SLE_VAR(Edge, last_unrestricted_update, SLE_INT32),
// SLE_CONDVAR(Edge, last_restricted_update,   SLE_INT32, SLV_187, SL_MAX_VERSION),
// SLE_VAR(Edge, dest_node,                SLE_UINT16),
// SLE_CONDVARNAME(Edge, dest_node, "next_edge", SLE_UINT16, SL_MIN_VERSION, SLV_LINKGRAPH_EDGES),
export interface LinkEdge {
  capacity: number;
  usage: number;
  travel_time_sum: number;
  last_unrestricted_update: number;
  last_restricted_update: number;
  next_edge?: number;
  dest_node: number;
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