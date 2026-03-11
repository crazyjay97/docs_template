// server.js
const http = require("http");
const fs = require("fs");
const path = require("path");

const PORT = 8080;
const ROOT = process.cwd();

const server = http.createServer((req, res) => {
  let filePath = path.join(ROOT, req.url === "/" ? "index.html" : req.url);

  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404);
      res.end("Not Found");
      return;
    }

    res.writeHead(200);
    res.end(data);
  });
});

server.listen(PORT, () => {
  console.log(`http://localhost:${PORT}`);
  console.log(`Serving: ${ROOT}`);
});
