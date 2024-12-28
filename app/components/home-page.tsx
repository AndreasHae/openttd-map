"use client";

import { useState } from "react";
import Graph from "graphology";
import { LinkGraph } from "../model/savefile-model";
import { deserializeCoordinates, transposeCoordinates } from "../model/coordinates";
import { Map } from "./map";
import dynamic from "next/dynamic";
import { useDropzone } from "react-dropzone";
import init, { load_file } from "../../savegame-reader/pkg";

export const HomePage = dynamic(
  async () => {
    await init();
    return () => {
      const [graph, setGraph] = useState<Graph | undefined>();

      const { getRootProps, getInputProps, isDragActive } = useDropzone({
        onDropAccepted: async (droppedFiles) => {
          await loadGraph(droppedFiles[0]);
        },
      });

      async function loadGraph(file: File): Promise<void> {
        const graph = new Graph({ type: "directed", multi: false, allowSelfLoops: false });

        const buf = new Uint8Array(await file.arrayBuffer());
        const allGraphs: LinkGraph[] = JSON.parse(load_file(buf));
        const passengerGraphs = allGraphs.filter((graph) => graph.cargo === 0);

        for (const nodes of passengerGraphs.map((graph) => graph.nodes)) {
          for (const node of nodes) {
            // TODO read from savefile
            const mapSizeX = 1024;
            const mapSizeY = 1024;

            const coords = deserializeCoordinates(node, mapSizeX);
            graph.mergeNode(node.station, transposeCoordinates(mapSizeX, coords, mapSizeY));

            for (const edge of node.edges) {
              if ((edge.next_edge ?? edge.dest_node) === 65535) break;

              const destination = nodes[edge.next_edge ?? edge.dest_node];
              graph.mergeNode(destination.station);
              graph.addEdge(node.station, destination.station);
            }
          }
        }

        setGraph(graph);
      }
      return (
        <main className={"w-screen h-screen" + (graph ? "" : " p-2")}>
          {graph ? (
            <Map graph={graph} />
          ) : (
            <div
              {...getRootProps()}
              className={
                "w-full h-full border-2 border-dashed flex flex-col items-center justify-center gap-4 cursor-pointer" +
                (isDragActive ? " border-blue-500" : "")
              }
            >
              <h1 className="text-5xl font-bold">OpenTTD Map Visualizer</h1>
              <p className="text-xl">
                Drag and drop your <code>.sav</code> file here or click anywhere to select file
              </p>
              <input {...getInputProps()} />
            </div>
          )}
        </main>
      );
    };
  },
  { ssr: false },
);
