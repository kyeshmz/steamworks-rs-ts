const button = document.querySelector('#copy')
const command = document.querySelector('#command')
const status = document.querySelector('#status')

button.hidden = false
button.addEventListener('click', async () => {
  const text = command.textContent
  try {
    await navigator.clipboard.writeText(text)
    status.textContent = 'Install command copied.'
  } catch {
    const range = document.createRange()
    range.selectNodeContents(command)
    const selection = window.getSelection()
    selection.removeAllRanges()
    selection.addRange(range)
    status.textContent = 'Command selected. Use your device’s Copy action.'
  }
})
