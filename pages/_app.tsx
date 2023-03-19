import "@fontsource/source-sans-pro";
import "@fontsource/source-sans-pro/600.css";
import "../styles/globals.css";
import type { AppProps } from "next/app";

function MyApp({ Component, pageProps }: AppProps) {
  return <Component {...pageProps} />;
}

export default MyApp;
