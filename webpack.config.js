const path = require('path');
//const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const TerserPlugin = require("terser-webpack-plugin");

var ROOT_PATH = path.resolve(__dirname);
var SHADER_PATH = path.resolve(ROOT_PATH, 'src/glsl');
var IMAGES_PATH = path.resolve(ROOT_PATH, 'src/img');
var CSS_PATH = path.resolve(ROOT_PATH, 'src/css');

module.exports = {
    entry: './src/js/Aladin.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'aladin.js',
        // Keep in dist/ only files used 
        clean: true,
    },
    resolve: {
        extensions: ['.js', '.glsl', '.vert', '.frag'],
    },
    experiments: {
        syncWebAssembly: true,
        asyncWebAssembly: true,
    },
    performance: {
        hints: false,
    },
    optimization: {
        //minimize: false,
        minimizer: [
            new TerserPlugin({
                terserOptions: {
                    mangle: true,
                    warnings: false,
                    compress: {},
                    safari10: true
                }
            }),
        ],
    },
    plugins: [
        // WebGL2 app
        new WasmPackPlugin({
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
            outDir: path.resolve(__dirname, 'pkg-webgl2'),

            // The same as the `--out-name` option for `wasm-pack`
            outName: "core",

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
            forceMode: "production",

            // Controls plugin output verbosity, either 'info' or 'error'.
            // Defaults to 'info'.
            pluginLogLevel: 'info'
        }),
        /*
        // Have this example work in Edge which doesn't ship `TextEncoder` or
        // `TextDecoder` at this time.
        // Maj 24/05/22: This should be supported by edge versions as of now (to be tested!)
        // This save 600kB in the project!
        new webpack.ProvidePlugin({
            TextDecoder: ['text-encoding', 'TextDecoder'],
            TextEncoder: ['text-encoding', 'TextEncoder']
        }),*/
        //new VueLoaderPlugin()
    ],
    devServer: {
        static: 'examples'
    },
    module: {
        rules: [
            {
                test: /\.m?js$/,
                exclude: /(node_modules|bower_components)/,
                use: {
                    loader: 'babel-loader',
                    options: {
                        presets: ['@babel/preset-env'],
                        plugins: ['@babel/plugin-proposal-object-rest-spread']
                    }
                }
            },
            {
                test: /\.(vert|frag|glsl)$/,
                include: SHADER_PATH,
                use: {
                    loader: 'webpack-glsl-minify',
                    options: {
                        output: 'source',
                        //preserveVariables: true,
                        //preserveUniforms: true,
                        preserveAll: true,
                    }
                }
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
                //sideEffects: true,
                include: CSS_PATH,
                use: [
                    'style-loader',
                    'css-loader',
                ]
            },
        ],
    },
    //mode: 'development',
    mode: 'development',
    //devtool: 'source-map'
};
