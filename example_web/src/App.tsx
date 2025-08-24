import { createSignal, onCleanup } from 'solid-js'
import './App.css'
import { TestData } from 'mottomesh'
import { wsconnect } from '@nats-io/nats-core'

function App() {
  const [count, setCount] = createSignal(0)
  const [connectedLog, setConnectedLog] = createSignal("")
  const [rxLogs, setRxLogs] = createSignal("")
  const [nc, setNc] = createSignal<any>(null)

  const data = () => new TestData(count(), 't' + count())

  const createConnection = async () => {
    console.log("Creating connection...");
    setConnectedLog("Connecting...");
    const ncInstance = await wsconnect({ servers: "ws://localhost:4333", tls: null });
    setNc(ncInstance)
    setConnectedLog(`Connected: ${ncInstance.getServer()}`);
    onCleanup(() => {
      ncInstance.close();
    });
    console.log(`connected: ${ncInstance.getServer()}`);
    const sub = ncInstance.subscribe("messages", {
      callback: (err, msg) => {
        if (err) {
          console.log(`error: ${err.message}`);
          return;
        }
        if (msg) {
          (async () => {
            const start = performance.now();
            const decompressedData = await decompressData(msg.data);
            const d = TestData.decode(new Uint8Array(decompressedData));
            const logMsg = `Rx: id:${d.id()} name: ${d.name()} compressed:${msg.data.byteLength} bytes, decompressed:${decompressedData.byteLength} bytes, took:${(performance.now() - start)} ms`;
            console.log(logMsg);
            setRxLogs((prev) => prev + `\n${logMsg}`);
          })();
        }
      }
    });
    onCleanup(() => {
      sub.unsubscribe();
    });
  }
  // Only run once
  createConnection();

  const handleSend = async () => {
    const ncInstance = nc();
    setCount(count() + 1);
    if (!ncInstance) return;
    const d = data()
    const encodedData = d.encode();
    const compressedData = await compressData(encodedData);
    ncInstance.publish("messages", new Uint8Array(compressedData));
    console.log(`Tx: id:${d.id()} name:${d.name()} compressed:${compressedData.byteLength} bytes, original:${encodedData.length} bytes`);
  }

  return (
    <>
      <div class="card">
        <p>
          {connectedLog()}
        </p>
        <button onClick={handleSend}>Send</button>
      </div>
      <pre class="rx-logs">
        {rxLogs()}
      </pre>
    </>
  )
}

// After encoding with TestData.encode(), use this function to compress data before sending it over the network
const compressData = (data: Uint8Array) => {
  // Using CompressionStream API
  const stream = new CompressionStream("gzip");
  const writer = stream.writable.getWriter();
  writer.write(new Uint8Array(data));
  writer.close();
  return new Response(stream.readable).arrayBuffer()
}
// Use this function to decompress data received over the network, then you can decode it with TestData.decode
const decompressData = (data: Uint8Array<ArrayBufferLike>) => {
  const stream = new DecompressionStream("gzip");
  const writer = stream.writable.getWriter();
  writer.write(new Uint8Array(data));
  writer.close();
  return new Response(stream.readable).arrayBuffer()
}

export default App
