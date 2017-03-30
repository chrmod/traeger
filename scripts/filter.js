var block = false;

function shouldBlock(message) {
  try {
    block = JSON.parse(message).block;
  } catch(e) {
  }
  return String(block);
}
