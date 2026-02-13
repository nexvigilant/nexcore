/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      colors: {
        nexcore: {
          black: "#020617",
          dark: "#0f172a",
          gold: "#fbbf24",
          cyan: "#22d3ee",
          amber: "#f59e0b",
        }
      },
      backgroundImage: {
        'glass-gradient': 'linear-gradient(to bottom right, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0))',
      },
      boxShadow: {
        'glow-cyan': '0 0 15px -3px rgba(34, 211, 238, 0.4)',
        'glow-amber': '0 0 15px -3px rgba(245, 158, 11, 0.4)',
      }
    },
  },
  plugins: [],
}
