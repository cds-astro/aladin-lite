const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const VueLoaderPlugin = require('vue-loader/lib/plugin');

var ROOT_PATH = path.resolve(__dirname);
var SHADER_PATH = path.resolve(ROOT_PATH, 'src/core/src/shaders');
var IMAGES_PATH = path.resolve(ROOT_PATH, 'src/core/img');

module.exports = {
    entry: './src/js/Aladin.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'aladin.js',
    },
    plugins: [
        //new HtmlWebpackPlugin(),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "src/core")
        }),
        // Have this example work in Edge which doesn't ship `TextEncoder` or
        // `TextDecoder` at this time.
        new webpack.ProvidePlugin({
          TextDecoder: ['text-encoding', 'TextDecoder'],
          TextEncoder: ['text-encoding', 'TextEncoder']
        }),
        new VueLoaderPlugin()
    ],
    devServer:{
        contentBase: 'dist'
    },
    module: {
        rules: [
            {
                test: /\.(vert|frag|glsl)$/,
                include: SHADER_PATH,
                loader: 'webpack-glsl-loader',
            },
            {
                test: /\.(png|svg|jpg|gif)$/,
                include: IMAGES_PATH,
                use: [
                    'file-loader',
                ],
            },
            {
                test: /.css$/,
                use: [
                  'css-loader',
                ]
            },
            {
                test: /\.vue$/,
                loader: 'vue-loader'
            },
        ],
    },
    mode: 'development',
    //mode: 'production',
    devtool: 'source-map'
};
