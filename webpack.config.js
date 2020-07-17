const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './src/js/Aladin.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'aladin.js',
    },
    plugins: [
        //new HtmlWebpackPlugin(),
        //new WasmPackPlugin({
            //crateDirectory: path.resolve(__dirname, ".")
        //}),
        // Have this example work in Edge which doesn't ship `TextEncoder` or
        // `TextDecoder` at this time.
        new webpack.ProvidePlugin({
          TextDecoder: ['text-encoding', 'TextDecoder'],
          TextEncoder: ['text-encoding', 'TextEncoder']
        })
    ],
    devServer:{
        contentBase: 'dist'
    },
    mode: 'development',
    //mode: 'production',
    devtool: 'source-map'
};
