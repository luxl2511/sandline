/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        'track-confidence-5': '#22c55e',
        'track-confidence-4': '#84cc16',
        'track-confidence-3': '#eab308',
        'track-confidence-2': '#f97316',
        'track-confidence-1': '#ef4444',
      },
    },
  },
  plugins: [],
}
