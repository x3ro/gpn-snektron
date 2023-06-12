// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg';`
// will work here one day as well!
//const rust = import('./pkg');

import { io } from 'socket.io-client';

var socket = io("http://127.0.0.1:4001");

socket.on("connect", () => {
  console.log(socket.id);
});


import('./pkg')
  .then(m => {
    console.log(m.start())
  })
  .catch(console.error);
