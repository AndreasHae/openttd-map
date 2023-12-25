import type { NextPage } from "next";
import Head from "next/head";
import { FilePicker } from "./file-picker";
import { useState } from "react";
import * as wasm from "../savegame-reader/pkg";

const Home: NextPage = () => {
  const [file, setFile] = useState<File | undefined>();

  async function load(file: File): Promise<void> {
    const buf = new Uint8Array(await file.arrayBuffer());
    console.log(wasm.load_file(buf));
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
              await load(file);
            }
          }}
        >
          Do the thing
        </button>
      </main>
    </>
  );
};

export default Home;
