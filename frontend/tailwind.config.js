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
        /* Light Theme */
        titleColor:'#2C2C2C',
        base:'#FFFFFF',
        sibarFontPrimaryColor:'#84828A',
        sidebarHoover: '#6425FE',
        sidebarBtnHoover:'#EFE9FF',
        sidebarRedHoover:'#FA0505',
        BoxTextColor:'#836E6E',
        BoxTextColor2:'#7B6A6A',
        AdviceBoxColor:'#838383',
        /* DarkTheme */
        titleColorDark:'#F7F7F7',
        baseDark:'#2C2C2C',
        sibarFontPrimaryColorDark:'#C9C9C9',
        sidebarHooverDark:'#6425FE',
        sidebarBtnHooverDark:'#EFE9FF',
        sidebarRedHooverDark:'#FA0505',
        BoxTextColorDark:'#836E6E',
        BoxTextColor2Dark:'#7B6A6A',
        AdviceBoxColorDark:'#838383',
        
      },
      fontFamily: {
        poppins: ["Poppins", "sans-serif"],
      },
    }, // Puoi aggiungere personalizzazioni qui
  },
  plugins: [],
};
