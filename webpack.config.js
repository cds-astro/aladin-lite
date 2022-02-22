const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
//const { VueLoaderPlugin } = require('vue-loader')

var ROOT_PATH = path.resolve(__dirname);
var SHADER_PATH = path.resolve(ROOT_PATH, 'src/core/src/shaders');
var IMAGES_PATH = path.resolve(ROOT_PATH, 'src/core/img');

module.exports = {
    entry: './src/js/Aladin.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'aladin.js',
    },
    experiments: {
        syncWebAssembly: true,
    },
    performance: {
        hints: false
    },
    plugins: [
        //new HtmlWebpackPlugin(),
        // WebGL1 app
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "src/core"),
            // Check https://rustwasm.github.io/wasm-pack/book/commands/build.html for
            // the available set of arguments.
            //
            // Optional space delimited arguments to appear before the wasm-pack
            // command. Default arguments are `--verbose`.
            args: '',
            // Default arguments are `--typescript --target browser --mode normal`.
            extraArgs: '--no-typescript -- --features webgl1',

            // Optional array of absolute paths to directories, changes to which
            // will trigger the build.
            // watchDirectories: [
            //   path.resolve(__dirname, "another-crate/src")
            // ],

            // The same as the `--out-dir` option for `wasm-pack`
            outDir: "pkg-webgl1",

            // The same as the `--out-name` option for `wasm-pack`
            // outName: "index",

            // If defined, `forceWatch` will force activate/deactivate watch mode for
            // `.rs` files.
            //
            // The default (not set) aligns watch mode for `.rs` files to Webpack's
            // watch mode.
            // forceWatch: true,

            // If defined, `forceMode` will force the compilation mode for `wasm-pack`
            //
            // Possible values are `development` and `production`.
            //
            // the mode `development` makes `wasm-pack` build in `debug` mode.
            // the mode `production` makes `wasm-pack` build in `release` mode.
            // forceMode: "development",

            // Controls plugin output verbosity, either 'info' or 'error'.
            // Defaults to 'info'.
            // pluginLogLevel: 'info'
        }),
        // WebGL2 app
        /*new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "src/core"),
            // Check https://rustwasm.github.io/wasm-pack/book/commands/build.html for
            // the available set of arguments.
            //
            // Optional space delimited arguments to appear before the wasm-pack
            // command. Default arguments are `--verbose`.
            args: '',
            // Default arguments are `--typescript --target browser --mode normal`.
            extraArgs: '--no-typescript -- --features webgl2',

            // Optional array of absolute paths to directories, changes to which
            // will trigger the build.
            // watchDirectories: [
            //   path.resolve(__dirname, "another-crate/src")
            // ],

            // The same as the `--out-dir` option for `wasm-pack`
            outDir: "pkg-webgl2",

            // The same as the `--out-name` option for `wasm-pack`
            // outName: "index",

            // If defined, `forceWatch` will force activate/deactivate watch mode for
            // `.rs` files.
            //
            // The default (not set) aligns watch mode for `.rs` files to Webpack's
            // watch mode.
            // forceWatch: true,

            // If defined, `forceMode` will force the compilation mode for `wasm-pack`
            //
            // Possible values are `development` and `production`.
            //
            // the mode `development` makes `wasm-pack` build in `debug` mode.
            // the mode `production` makes `wasm-pack` build in `release` mode.
            // forceMode: "development",

            // Controls plugin output verbosity, either 'info' or 'error'.
            // Defaults to 'info'.
            // pluginLogLevel: 'info'
        }),*/
        // Have this example work in Edge which doesn't ship `TextEncoder` or
        // `TextDecoder` at this time.
        new webpack.ProvidePlugin({
          TextDecoder: ['text-encoding', 'TextDecoder'],
          TextEncoder: ['text-encoding', 'TextEncoder']
        }),
        //new VueLoaderPlugin()
    ],
    devServer:{
        static: 'dist'
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
            /*{
                test: /\.vue$/,
                loader: 'vue-loader'
            },*/
        ],
    },
    mode: 'development',
    devtool: 'source-map'
};
