import React from "react";

const SensorMap = () => {
  return (
    <div style={styles.container}>
      <div style={styles.mapContainer}>
        <h2 style={styles.header}>Your sensor map</h2>
        <img
          src="https://placeholder.pics/svg/680x432"
          alt="Map with sensor points"
          style={styles.map}
        />
      </div>
      <div style={styles.devicesContainer}>
        <div style={styles.devicesHeaderContainer}>
          <p style={styles.devicesHeader}>Devices</p>
          <button style={styles.addButton}>+</button>
        </div>
        <ul style={styles.deviceList}>
          {[1, 2, 3, 4, 5].map((device) => (
            <li key={device} style={styles.deviceItem}>
              <img
                src="https://placeholder.pics/svg/26x26"
                alt="Device icon"
                style={styles.deviceIcon}
              />
              <div style={styles.deviceTextContainer}>
                <p style={styles.deviceName}>Device {device}</p>
                <p style={styles.deviceProtocol}>Protocol: MQTT</p>
              </div>
            </li>
          ))}
        </ul>
        <button style={styles.manageButton}>
          <span>Manage</span>
          <img
            src="https://placeholder.pics/svg/20x20"
            alt="Manage icon"
            style={styles.manageIcon}
          />
        </button>
      </div>
    </div>
  );
};

export default SensorMap;


const styles = {
    container: {
      display: 'flex',
      flexDirection: 'row',
      padding: '24px',
      gap: '20px',
      backgroundColor: '#ffffff',
      borderRadius: '8px',
    },
    mapContainer: {
      width: '705px',
    },
    header: {
      fontFamily: 'Poppins, sans-serif',
      fontSize: '15px',
      fontWeight: 400,
      lineHeight: '22.5px',
      color: '#000000',
      marginBottom: '24px',
    },
    map: {
      width: '680px',
      height: '432px',
    },
    devicesContainer: {
      width: '316px',
      padding: '20px 12px',
      backgroundColor: '#ffffff',
      borderRadius: '8px',
    },
    devicesHeaderContainer: {
      display: 'flex',
      justifyContent: 'space-between',
      alignItems: 'center',
      marginBottom: '20px',
    },
    devicesHeader: {
      fontFamily: 'Poppins, sans-serif',
      fontSize: '15px',
      fontWeight: 400,
      lineHeight: '22.5px',
    },
    addButton: {
      width: '24px',
      height: '24px',
      display: 'flex',
      justifyContent: 'center',
      alignItems: 'center',
      backgroundColor: '#6425fe',
      color: '#ffffff',
      border: 'none',
      borderRadius: '4.36px',
      cursor: 'pointer',
    },
    deviceList: {
      listStyleType: 'none',
      padding: 0,
    },
    deviceItem: {
      display: 'flex',
      alignItems: 'center',
      gap: '16px',
      marginBottom: '16px',
    },
    deviceIcon: {
      width: '26px',
      height: '26px',
    },
    deviceTextContainer: {
      display: 'flex',
      flexDirection: 'column',
    },
    deviceName: {
      fontFamily: 'Poppins, sans-serif',
      fontSize: '15px',
      fontWeight: 400,
      lineHeight: '22.5px',
      color: '#2c2c2c',
    },
    deviceProtocol: {
      fontFamily: 'Poppins, sans-serif',
      fontSize: '15px',
      fontWeight: 400,
      lineHeight: '22.5px',
      color: '#2c2c2c',
    },
    manageButton: {
      marginTop: '20px',
      padding: '10px 24px',
      borderRadius: '100px',
      backgroundColor: '#6425fe',
      color: 'white',
      border: 'none',
      cursor: 'pointer',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      gap: '8px',
    },
    manageIcon: {
      width: '20px',
      height: '20px',
    },
  };
  