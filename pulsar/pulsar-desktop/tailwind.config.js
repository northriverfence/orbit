/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        sidebar: {
          bg: '#F7F8FA',
          hover: '#E8EAED',
          active: '#D1D5DB',
        },
        accent: {
          primary: '#10B981',         // Green for active items
          'primary-dark': '#059669',  // Darker green for hover states
          secondary: '#8B5CF6',       // Purple for badges
          'secondary-dark': '#7C3AED', // Darker purple for hover states
        }
      }
    },
  },
  plugins: [],
}
