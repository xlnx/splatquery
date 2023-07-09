const plugin = require("tailwindcss/plugin");
const colors = require('tailwindcss/colors');

module.exports = {
  content: [
    './index.html',
    './src/**/*.{vue,js,mjs}',
    "./node_modules/flowbite/**/*.js",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      borderWidth: {
        '3': '3px',
      },
      brightness: {
        15: '.15',
        25: '.25',
        75: '.75',
      },
      sepia: {
        50: '.50',
        75: '.75',
      },
      fontFamily: {
        splatoon1: 'var(--font-family-s1)',
        splatoon2: 'var(--font-family-s2)',
      },
    },
  },
  variants: {
    extend: {
      opacity: ['disabled'],
    }
  },
  plugins: [
    require('flowbite/plugin'),
    plugin(function({ addVariant }) {
      addVariant('mobile', 'body.is-mobile &');
      addVariant('ss', 'body.for-screenshots &');
    })
  ]
}
