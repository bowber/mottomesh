import { createSignal, onCleanup, onMount } from 'solid-js'
import './App.css'
import { decodeTestData, encodeTestData, type TestData } from '@motto/schema'
import { MottomeshClient } from '@mottomesh/client'

// Helper function to generate a demo JWT token
// In production, this would come from your auth server
async function getDemoJwtToken(): Promise<string> {
  // This is a demo token that matches what the gateway expects
  // In production, get this from your authentication server
  const header = btoa(JSON.stringify({ alg: 'HS256', typ: 'JWT' }));
  const payload = btoa(JSON.stringify({
    sub: 'demo-user',
    exp: Math.floor(Date.now() / 1000) + 3600, // 1 hour from now
    iat: Math.floor(Date.now() / 1000),
    permissions: ['publish', 'subscribe', 'request'],
    allowed_subjects: ['messages', 'messages.*', 'messages.>'],
    deny_subjects: [],
  }));
  // Note: This signature is fake - in production, the server signs with the secret
  const signature = 'demo-signature';
  return `${header}.${payload}.${signature}`;
}

function App() {
  const [count, setCount] = createSignal(0)
  const [connectedLog, setConnectedLog] = createSignal("")
  const [rxLogs, setRxLogs] = createSignal("")
  const [client, setClient] = createSignal<MottomeshClient | null>(null)
  const [transportType, setTransportType] = createSignal<string>("")

  const data = (): TestData => ({
    id: count(),
    name: `t${count()}`,
    inner_data: {
      id: Array.from({ length: 1000 }, () => count()),
      name: Array.from({ length: 1000 }, () => `t${count()}`),
    },
  })

  const createConnection = async () => {
    console.log("Creating connection...");
    setConnectedLog("Connecting...");
    
    try {
      // Get JWT token (in production, this comes from your auth server)
      const token = await getDemoJwtToken();
      
      const clientInstance = new MottomeshClient({
        url: 'https://localhost:4433',
        token,
        transport: 'auto', // Will try WebTransport first, fall back to WebSocket
        reconnect: true,
      });

      // Set up event handlers
      clientInstance.on('connect', () => {
        console.log('Connected to gateway');
        setConnectedLog(`Connected (Session: ${clientInstance.getSessionId()})`);
      });

      clientInstance.on('disconnect', (reason) => {
        console.log('Disconnected:', reason);
        setConnectedLog(`Disconnected: ${reason}`);
      });

      clientInstance.on('error', (error) => {
        console.error('Client error:', error);
      });

      clientInstance.on('auth', (data: any) => {
        console.log('Authenticated:', data);
        setTransportType(clientInstance.isConnected() ? 'Connected' : 'Disconnected');
      });

      await clientInstance.connect();
      setClient(clientInstance);

      // Subscribe to messages
      const sub = clientInstance.subscribe('messages', async (msg) => {
        const start = performance.now();
        const decompressedData = await decompressData(msg.payload);
        const d = decodeTestData(new Uint8Array(decompressedData));
        const logMsg = `Rx: id:${d.id} name: ${d.name} compressed:${msg.payload.byteLength} bytes, decompressed:${decompressedData.byteLength} bytes, took:${(performance.now() - start).toFixed(2)} ms`;
        console.log(logMsg);
        setRxLogs((prev) => prev + `\n${logMsg}`);
      });

      onCleanup(async () => {
        await sub.unsubscribe();
        await clientInstance.disconnect();
      });

    } catch (error) {
      console.error('Connection failed:', error);
      setConnectedLog(`Connection failed: ${error}`);
    }
  }

  onMount(() => {
    createConnection();
  });

  const handleSend = async () => {
    const clientInstance = client();
    setCount(count() + 1);
    if (!clientInstance || !clientInstance.isConnected()) return;
    
    const d = data()
    const encodedData = encodeTestData(d);
    const compressedData = await compressData(encodedData);
    await clientInstance.publish("messages", new Uint8Array(compressedData));
    console.log(`Tx: id:${d.id} name:${d.name} compressed:${compressedData.byteLength} bytes, original:${encodedData.length} bytes`);
  }

  return (
    <>
      <h1>Mottomesh Example</h1>
      <div class="card">
        <p>
          {connectedLog()}
        </p>
        <p class="transport-type">
          Transport: {transportType()}
        </p>
        <button onClick={handleSend}>Send Message</button>
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
const decompressData = (data: Uint8Array) => {
  const stream = new DecompressionStream("gzip");
  const writer = stream.writable.getWriter();
  writer.write(new Uint8Array(data));
  writer.close();
  return new Response(stream.readable).arrayBuffer()
}

export default App
