import type { NextPage } from "next";
import Head from "next/head";
import { FilePicker } from "./file-picker";
import { useEffect, useState } from "react";
import * as wasm from "../savegame-reader/pkg";
import { SigmaContainer, useLoadGraph } from "@react-sigma/core";
import Graph from "graphology";
import "@react-sigma/core/lib/react-sigma.min.css";
import { LinkGraph } from "../model/savefile-model";
import { deserializeCoordinates, transposeCoordinates } from "../model/coordinates";

const Home: NextPage = () => {
  const [file, setFile] = useState<File | undefined>();
  const [graph, setGraph] = useState<Graph | undefined>();

  async function loadGraph(file: File): Promise<void> {
    const graph = new Graph({ type: "directed", multi: false, allowSelfLoops: false });

    const buf = new Uint8Array(await file.arrayBuffer());
    const allGraphs: LinkGraph[] = JSON.parse(wasm.load_file(buf));
    const passengerGraphs = allGraphs.filter((graph) => graph.cargo === 0);

    for (const nodes of passengerGraphs.map((graph) => graph.nodes)) {
      for (const node of nodes) {
        // TODO read from savefile
        const mapSizeX = 1024;
        const mapSizeY = 1024;

        const coords = deserializeCoordinates(node, mapSizeX);
        graph.mergeNode(node.station, transposeCoordinates(mapSizeX, coords, mapSizeY));

        for (const edge of node.edges) {
          if (edge.next_edge === 65535) break;

          const destination = nodes[edge.next_edge];
          graph.mergeNode(destination.station);
          graph.addEdge(node.station, destination.station);
        }
      }
    }

    setGraph(graph);
  }

  return (
    <>
      <Head>
        <title>OTTD Transit Map</title>
      </Head>

      <main className="absolute inset-0 p-4 text-center bg-neutral-50 flex flex-col items-center justify-center">
        <h1 className="text-7xl md:text-8xl font-semibold">Hey there!</h1>
        <h2 className="text-3xl md:text-4xl mt-6">More coming soon 🚧👷🏻‍♂️</h2>
        <FilePicker onFileChanged={setFile} />
        <button
          className="bg-red-300 p-1 border rounded mt-2"
          onClick={async () => {
            if (file) {
              await loadGraph(file);
            }
          }}
        >
          Do the thing
        </button>
        {graph && <DisplayGraph graph={graph} />}
      </main>
    </>
  );
};

interface LoadGraphProps {
  graph: Graph;
}

export const LoadGraph = ({ graph }: LoadGraphProps) => {
  const loadGraph = useLoadGraph();

  useEffect(() => {
    loadGraph(graph);
  }, [loadGraph, graph]);

  return null;
};

export const DisplayGraph = ({ graph }: LoadGraphProps) => {
  return (
    <SigmaContainer style={{ height: "500px", width: "500px" }}>
      <LoadGraph graph={graph} />
    </SigmaContainer>
  );
};

export default Home;
