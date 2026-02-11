/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'charcoal': '#1a1a1a',
        'amber': '#ffbf00',
        'orange': '#ff6600',
      },
      fontFamily: {
        'mono': ['Monaco', 'Menlo', 'monospace'],
      },
    },
  },
  plugins: [],
}
