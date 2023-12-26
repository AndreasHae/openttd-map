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

export const Map = ({ graph }: LoadGraphProps) => {
  return (
    <SigmaContainer className="w-full h-full">
      <LoadGraph graph={graph} />
    </SigmaContainer>
  );
};
