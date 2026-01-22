/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // PYRAX Brand Colors - Orange/Rust Gradient
        pyrax: {
          50: '#fff7ed',
          100: '#ffedd5',
          200: '#fed7aa',
          300: '#fdba74',
          400: '#fb923c',
          500: '#f97316',
          600: '#ea580c',
          700: '#c2410c',
          800: '#9a3412',
          900: '#7c2d12',
          950: '#431407',
        },
        // Dark Mode Background
        dark: {
          900: '#0c0a09',
          800: '#1c1917',
          700: '#292524',
          600: '#44403c',
        },
        // Legacy gray support
        gray: {
          650: '#4a5568',
          750: '#374151',
          850: '#1e2533',
        },
      },
    },
  },
  plugins: [],
};
