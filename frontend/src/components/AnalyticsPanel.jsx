import React from "react";

const AnalyticsPanel = () => {
  return (
    <div
      style={{
        width: "681px",
        height: "504px",
        padding: "24px 12px",
        backgroundColor: "#ffffff",
        borderRadius: "8px",
        boxShadow: "0 4px 8px rgba(0, 0, 0, 0.1)",
        position: "relative",
      }}
    >
      <h2
        style={{
          fontFamily: "Poppins",
          fontWeight: 400,
          fontSize: "15px",
          lineHeight: "22.5px",
          color: "#000000",
          margin: "0 0 20px 12px",
        }}
      >
        Analytics
      </h2>
      <div
        style={{
          position: "absolute",
          top: "60px",
          left: "12px",
          width: "621.16px",
          height: "378.44px",
          background:
            "url(https://placeholder.pics/svg/621x378) no-repeat center center",
          backgroundSize: "cover",
        }}
      ></div>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          padding: "0 60px",
          position: "absolute",
          bottom: "30px",
          width: "100%",
        }}
      >
        <span
          style={{
            fontFamily: "Poppins",
            fontWeight: 600,
            fontSize: "12px",
            color: "#33323a",
          }}
        >
          2014
        </span>
        <span
          style={{
            fontFamily: "Poppins",
            fontWeight: 600,
            fontSize: "12px",
            color: "#33323a",
          }}
        >
          2016
        </span>
        <span
          style={{
            fontFamily: "Poppins",
            fontWeight: 600,
            fontSize: "12px",
            color: "#33323a",
          }}
        >
          2018
        </span>
        <span
          style={{
            fontFamily: "Poppins",
            fontWeight: 600,
            fontSize: "12px",
            color: "#33323a",
          }}
        >
          2020
        </span>
        <span
          style={{
            fontFamily: "Poppins",
            fontWeight: 600,
            fontSize: "12px",
            color: "#33323a",
          }}
        >
          2022
        </span>
      </div>
    </div>
  );
};

export default AnalyticsPanel;
