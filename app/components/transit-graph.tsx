import Graph from "graphology";
import { SigmaContainer, useLoadGraph } from "@react-sigma/core";
import { useEffect } from "react";

interface LoadGraphProps {
  graph: Graph;
}

const LoadGraph = ({ graph }: LoadGraphProps) => {
  const loadGraph = useLoadGraph();

  useEffect(() => {
    loadGraph(graph);
  }, [loadGraph, graph]);

  return <></>;
};

export const TransitGraph = ({ graph }: LoadGraphProps) => {
  return (
    <SigmaContainer style={{ height: "500px", width: "500px" }}>
      <LoadGraph graph={graph} />
    </SigmaContainer>
  );
};
