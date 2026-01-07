/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
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
