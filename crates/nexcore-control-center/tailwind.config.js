/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: ["*.html", "./src/**/*.rs"],
  },
  theme: {
    extend: {
      colors: {
        // NexCore brand colors
        nexcore: {
          blue: '#3B82F6',
          dark: '#1F2937',
          darker: '#111827',
        }
      }
    },
  },
  plugins: [],
}
