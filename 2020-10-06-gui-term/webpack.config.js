/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable no-undef */

const path = require("path")

const TARGET_DIR = path.resolve(__dirname, "target")
const TSCONFIG_PATH = path.resolve(__dirname, "tsconfig-client.json")

module.exports = {
  entry: "./src/renderer.ts",

  devtool: "source-map",

  output: {
    filename: "renderer_bundled.js",
    path: TARGET_DIR,
  },

  resolve: {
    extensions: [
      ".ts",
      ".tsx",
      ".js",
    ],
  },

  module: {
    rules: [
      {
        test: /\.tsx?$/,
        loader: "ts-loader",
        options: {
          configFile: TSCONFIG_PATH,
        },
      },
      {
        enforce: "pre",
        test: /\.js$/,
        loader: "source-map-loader",
      },
    ],
  },

  // <https://webpack.js.org/configuration/externals/>
  externals: {
    "electron": "commonjs electron",
    // "react": "React",
    // "react-dom": "ReactDOM",
  },
}
