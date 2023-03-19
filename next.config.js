/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  webpack: (config) => {
    return { ...config, experiments: { asyncWebAssembly: true } };
  },
};

module.exports = nextConfig;
