/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
    "./src/components/*.{js,jsx,ts,tsx}",
    "./src/pages/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    extend: {
      colors:{
        titleColor:'#2C2C2C',
        base:'#FFFFFF',
        sibarFontPrimaryColor:'#84828A',
        sidebarHoover: '#6425FE',
        sidebarBtnHoover:'#EFE9FF',
        sidebarRedHoover:'#FA0505',
        BoxTextColor:'#836E6E',
        BoxTextColor2:'#7B6A6A'
      },
      fontFamily: {
        poppins: ["Poppins", "sans-serif"],
      },
    }, // Puoi aggiungere personalizzazioni qui
  },
  plugins: [],
};
