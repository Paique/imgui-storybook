// Smoke test for the rust host WebSocket: mirrors the protocol of packages/client.
// Usage: node rust-host/examples/ws-smoke.mjs [port]
const port = process.argv[2] || "8765";
const ws = new WebSocket(`ws://localhost:${port}`);

const seen = [];
let frames = 0;
let stage = "connect";

const timer = setTimeout(() => {
  console.error("TIMEOUT at stage:", stage, "seen:", seen.join(","));
  process.exit(1);
}, 15000);

ws.onopen = () => {
  stage = "hello";
};

ws.onmessage = async (ev) => {
  const msg = JSON.parse(typeof ev.data === "string" ? ev.data : await ev.data.text());
  seen.push(msg.type);

  if (msg.type === "hello") {
    console.log("hello:", msg.host, msg.hostVersion, "dearImgui:", msg.dearImgui, `${msg.width}x${msg.height}`);
    stage = "select";
    ws.send(JSON.stringify({ type: "select", storyId: "inputs-slider" }));
    return;
  }
  if (msg.type === "selected") {
    console.log("selected:", msg.storyId, "args:", JSON.stringify(msg.args));
    stage = "view";
    ws.send(JSON.stringify({ type: "setView", view: { theme: "light", scale: 1.5, backdrop: "checker", canvasMode: "inline" } }));
    return;
  }
  if (msg.type === "view") {
    console.log("view:", JSON.stringify(msg.view));
    if (msg.view.theme === "light" && msg.view.scale === 1.5) {
      stage = "frame";
      ws.send(JSON.stringify({ type: "setView", view: { theme: "dark", scale: 1, backdrop: "neutral-dark", canvasMode: "windowed" } }));
      ws.send(JSON.stringify({ type: "setArgs", args: { value: 77.5 } }));
      ws.send(JSON.stringify({ type: "ping" }));
    }
    return;
  }
  if (msg.type === "frame") {
    frames++;
    if (frames === 1) console.log("first frame: seq", msg.seq, `${msg.width}x${msg.height}`, msg.mime, "bytes:", Math.round(msg.data.length * 0.75));
    if (frames >= 15) {
      // sample a JPEG frame to check base64 validity
      const buf = Buffer.from(msg.data, "base64");
      const ok = buf[0] === 0xff && buf[1] === 0xd8;
      console.log(`frames: ${frames}, last seq ${msg.seq}, jpeg SOI ok: ${ok}, size: ${buf.length}`);
      if (!ok) { clearTimeout(timer); process.exit(1); }
      clearTimeout(timer);
      console.log("WS SMOKE OK");
      process.exit(0);
    }
  }
  if (msg.type === "pong") {
    console.log("pong received");
  }
  if (msg.type === "error") {
    console.error("host error:", msg.message);
  }
};

ws.onclose = () => {
  if (stage !== "done") {
    console.error("connection closed early at", stage);
    process.exit(1);
  }
};
