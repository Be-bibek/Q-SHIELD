/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        space: {
          900: "#070b19",
          800: "#0d142d",
          700: "#152046",
          600: "#1f2f64",
        },
        cyber: {
          cyan: "#00f0ff",
          green: "#00ff88",
          purple: "#9d4edd",
          amber: "#ffaa00",
        }
      }
    },
  },
  plugins: [],
}
