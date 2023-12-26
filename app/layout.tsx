import "@fontsource/source-sans-pro";
import "@fontsource/source-sans-pro/600.css";
import "../styles/globals.css";
import "@react-sigma/core/lib/react-sigma.min.css";
import { Metadata } from "next";

export const metadata: Metadata = {
  title: "OTTD Transit Map",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
