window.steam.playerName().then(name => {
    document.getElementById('name').textContent = name
})

document.getElementById('activateOverlay').addEventListener('click', function() {
    window.steam.openExampleOverlay()
})
