var block = false;
var port = -1;

var actions = {
  getBlock: function(ip) {
    return block;
  },
  setBlock: function (b) {
    block = Boolean(b);
  },
  getPort: function () {
    return port;
  },
  setPort: function (p) {
    port = Number(p);
  },
}

onmessage = function(message) {
  log(`js received: ${message}`);

  var response;
  var responseId;
  try {
    var msg = JSON.parse(message);
    var action = msg.action;
    var args = msg.args || [];
    requestId = msg.requestId;
    var responseVal = actions[action].apply(null, args);
    response = {
      requestId: requestId,
      response: responseVal
    };
  } catch(e) {
    response = {
      responseId: responseId,
      error: e.toString()
    };
  }
  return JSON.stringify(response);
}
