const { app, BrowserWindow, Menu } = require('electron');
const path = require('path');

function createWindow() {
  const win = new BrowserWindow({
    width: 1920,
    height: 1080,
    backgroundColor: '#1e222a',
    title: 'DroxIDE',
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    }
  });

  win.loadFile('index.html');

  const template = [
    { label: 'File', submenu: [{ label: 'Open Folder...', accelerator: 'CmdOrCtrl+O' }] },
    { label: 'View', submenu: [] },
    { label: 'Swarm', submenu: [{ label: 'Run Swarm', accelerator: 'CmdOrCtrl+Shift+R' }] },
    { label: 'Help', submenu: [{ label: 'About' }] }
  ];

  Menu.setApplicationMenu(Menu.buildFromTemplate(template));
}

app.whenReady().then(createWindow);

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});