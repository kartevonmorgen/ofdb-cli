module.exports = {
  content: [
     'index.html',
     './src/**/*.rs',
  ],
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography')
  ],
}
