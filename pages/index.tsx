import type { NextPage } from "next";
import Head from "next/head";
import { add } from "../savegame-reader/pkg";

const Home: NextPage = () => {
  return (
    <>
      <Head>
        <title>OTTD Transit Map</title>
      </Head>

      <main className="absolute inset-0 p-4 text-center bg-neutral-50 flex flex-col items-center justify-center">
        <h1 className="text-7xl md:text-8xl font-semibold">Hey there!</h1>
        <h2 className="text-3xl md:text-4xl mt-6">More coming soon 🚧👷🏻‍♂️</h2>
        <p className="text-xl mt-24">
          2 + 2 = {add(2, 2)} from WASM!
        </p>
        <p className="text-xl mt-24">
          Meanwhile, you can check out my <a href="https://github.com/AndreasHae">Github</a> or my{" "}
          <a href="https://www.linkedin.com/in/andreas-h%C3%A4ssler-603b41172/">LinkedIn</a>
        </p>
      </main>
    </>
  );
};

export default Home;
