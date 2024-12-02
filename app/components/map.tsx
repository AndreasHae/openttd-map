"use client";

import dynamic from "next/dynamic";
import { LoadGraphProps } from "./loadGraph";

const isBrowser = () => typeof window !== "undefined";

export const Map = ({ graph }: LoadGraphProps) => {
  if (isBrowser()) {
    const SigmaContainer = dynamic(
      import("@react-sigma/core").then((mod) => mod.SigmaContainer),
      { ssr: false },
    );
    const LoadGraph = dynamic(
      import("./loadGraph").then((mod) => mod.LoadGraph),
      { ssr: false },
    );
    return (
      <SigmaContainer>
        <LoadGraph graph={graph} />
      </SigmaContainer>
    );
  } else return <p>NOT AVAILABLE</p>;
};
