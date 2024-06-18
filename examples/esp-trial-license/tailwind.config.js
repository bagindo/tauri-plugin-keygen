/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        "app-red": "#ED4D45",
        "app-white": "#FCFAF6",
      },
      fontFamily: {
        syne: ["Syne"],
      },
    },
  },
  plugins: [],
};
