import React from "react";

const Box = ({ title, value, backgroundColor }) => {
  return (
    <div
      className="rounded-lg p-6 w-[220px] h-[130px] m-4 text-center text-gray-800 shadow-lg transform transition-all duration-300 hover:scale-105"
      style={{ backgroundColor: backgroundColor }}
    >
      <h3 className="text-xl font-semibold leading-[24px] text-gray-700">{title}</h3>
      <p className="my-2 text-sm text-gray-500">Current Value</p>
      <h4 className="text-2xl font-bold leading-[32px] text-gray-900">{value}</h4>
    </div>
  );
};

export default Box;
