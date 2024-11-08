import React from "react";

const Box = ({ title, value, backgroundColor }) => {
  return (
    <div
      style={{
        backgroundColor: backgroundColor,
        borderRadius: "8px",
        padding: "12px",
        width: "203px",
        height: "115px",
        margin: "8px",
        textAlign: "center",
        color: "#000",
        fontFamily: "Poppins, sans-serif",
      }}
    >
      <h3
        style={{
          fontSize: "15px",
          fontWeight: 400,
          lineHeight: "22.5px",
        }}
      >
        {title}
      </h3>
      <p
        style={{
          margin: "10px 0",
          fontSize: "12px",
        }}
      >
        Current Value
      </p>
      <h4
        style={{
          fontSize: "20px",
          fontWeight: 500,
          lineHeight: "30px",
        }}
      >
        {value}
      </h4>
    </div>
  );
};

export default Box;
