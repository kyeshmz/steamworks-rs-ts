const { init } = require('../index.js')

const client = init(480)

client.overlay.activateToWebPage('https://www.example.com/');
