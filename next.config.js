/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  transpilePackages: ["@react-sigma/core"],
  webpack: (config) => {
    config.experiments.asyncWebAssembly = true;
    config.experiments.topLevelAwait = true;
    return config;
  },
};

module.exports = nextConfig;
