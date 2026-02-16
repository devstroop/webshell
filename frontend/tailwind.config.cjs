/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        terminal: {
          DEFAULT: '#1a1b26',
          foreground: '#d4d4d4',
        },
      },
    },
  },
  plugins: [],
}
