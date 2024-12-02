import { useLoadGraph } from "@react-sigma/core";
import { useEffect } from "react";
import Graph from "graphology";

export interface LoadGraphProps {
  graph: Graph;
}

export const LoadGraph = ({ graph }: LoadGraphProps) => {
  const loadGraph = useLoadGraph();

  useEffect(() => {
    loadGraph(graph);
  }, [loadGraph, graph]);

  return <></>;
};
