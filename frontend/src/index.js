const { app, BrowserWindow } = require('electron');
const path = require('path');

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
  app.quit();
}

const createWindow = () => {
  // Create the browser window.
  const mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      nodeIntegration: false, // Meglio disabilitarlo e usare il preload.js per maggiore sicurezza
      contextIsolation: true,  // Mantiene l'isolamento del contesto tra Electron e frontend
      enableRemoteModule: false
    },
  });

  // Load the React build output (che si trova nella cartella dist dopo la build di Webpack)
  mainWindow.loadFile(path.join(__dirname, 'dist', 'index.html'));

  // Open the DevTools (opzionale).
  mainWindow.webContents.openDevTools();
};

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
app.whenReady().then(() => {
  createWindow();

  // On macOS, it's common to re-create a window when the
  // dock icon is clicked and there are no open windows.
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

// Quit when all windows are closed, except on macOS.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
