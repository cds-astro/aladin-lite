module.exports = {
  plugins: [
    'postcss-preset-env',
    require("tailwindcss"),
    require("autoprefixer"),
    ...(process.env.NODE_ENV === "production" ? [require("cssnano")] : []),
  ],
};
