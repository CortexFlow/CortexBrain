import React from 'react';
import './Dashboard.css';

// Importing components
const SidebarMenu = () => {
  return (
    <div className="sidebar">
      <header className="sidebar-header">
        <h1 className="title">CortexFlow</h1>
        <p className="user-panel">User Panel</p>
      </header>
      <nav className="sidebar-nav">
        <ul>
          <li className="active">Dashboard</li>
          <li>Nodes</li>
          <li>Pipeline</li>
          <li>Config</li>
          <li>Roles</li>
          <li>Settings</li>
          <li>Tutorials</li>
        </ul>
      </nav>
      <div className="advice">
        <img src="https://placeholder.pics/svg/64x38" alt="Advice" />
        <div>
          <h2 className="advice-title">CortexFlow Advices</h2>
          <p className="advice-content">Create your first pipeline by clicking in the "Pipelines" section.</p>
        </div>
      </div>
      <footer className="sidebar-footer">
        <a href="#logout" className="logout">
          Logout
        </a>
      </footer>
    </div>
  );
};

const Box = ({ title, value, backgroundColor }) => {
  return (
    <div style={{
      backgroundColor: backgroundColor,
      borderRadius: '8px',
      padding: '12px',
      width: '203px',
      height: '115px',
      margin: '8px',
      textAlign: 'center',
      color: '#000',
      fontFamily: 'Poppins, sans-serif'
    }}>
      <h3 style={{
        fontSize: '15px',
        fontWeight: 400,
        lineHeight: '22.5px'
      }}>{title}</h3>
      <p style={{
        margin: '10px 0',
        fontSize: '12px'
      }}>Current Value</p>
      <h4 style={{
        fontSize: '20px',
        fontWeight: 500,
        lineHeight: '30px'
      }}>{value}</h4>
    </div>
  );
};

const ClusterStatus = () => {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '12px' }}>
      <Box title="Memory" value="10%" backgroundColor="#a6f7e2" />
      <Box title="CPU" value="50%" backgroundColor="#d5c4ff" />
      <Box title="Node Status" value="Online" backgroundColor="#ffe5a5" />
      <Box title="Deployment Status" value="70%" backgroundColor="#c7ffa5" />
    </div>
  );
};

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
          <img src="https://placeholder.pics/svg/20x20" alt="Manage icon" style={styles.manageIcon}/>
        </button>
      </div>
    </div>
  );
};

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

const AnalyticsPanel = () => {
  return (
    <div style={{
      width: '681px',
      height: '504px',
      padding: '24px 12px',
      backgroundColor: '#ffffff',
      borderRadius: '8px',
      boxShadow: '0 4px 8px rgba(0, 0, 0, 0.1)',
      position: 'relative'
    }}>
      <h2 style={{
        fontFamily: 'Poppins',
        fontWeight: 400,
        fontSize: '15px',
        lineHeight: '22.5px',
        color: '#000000',
        margin: '0 0 20px 12px'
      }}>Analytics</h2>
      <div style={{
        position: 'absolute',
        top: '60px',
        left: '12px',
        width: '621.16px',
        height: '378.44px',
        background: 'url(https://placeholder.pics/svg/621x378) no-repeat center center',
        backgroundSize: 'cover'
      }}></div>
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        padding: '0 60px',
        position: 'absolute',
        bottom: '30px',
        width: '100%'
      }}>
        <span style={{
          fontFamily: 'Poppins',
          fontWeight: 600,
          fontSize: '12px',
          color: '#33323a'
        }}>2014</span>
        <span style={{
          fontFamily: 'Poppins',
          fontWeight: 600,
          fontSize: '12px',
          color: '#33323a'
        }}>2016</span>
        <span style={{
          fontFamily: 'Poppins',
          fontWeight: 600,
          fontSize: '12px',
          color: '#33323a'
        }}>2018</span>
        <span style={{
          fontFamily: 'Poppins',
          fontWeight: 600,
          fontSize: '12px',
          color: '#33323a'
        }}>2020</span>
        <span style={{
          fontFamily: 'Poppins',
          fontWeight: 600,
          fontSize: '12px',
          color: '#33323a'
        }}>2022</span>
      </div>
    </div>
  );
};

const Dashboard = () => {
  return (
    <div style={{ display: 'flex', flexDirection: 'row', backgroundColor: '#f5f5f5' }}>
      <SidebarMenu />
      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', padding: '24px', flex: 1 }}>
        <ClusterStatus />
        <div style={{ display: 'flex', gap: '16px' }}>
          <SensorMap />
          <AnalyticsPanel />
        </div>
      </div>
    </div>
  );
};

export default Dashboard;
