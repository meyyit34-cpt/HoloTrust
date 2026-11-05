import { AppWebsocket } from "https://esm.sh/@holochain/client@0.21.0";
import { decode, encode } from "https://esm.sh/@msgpack/msgpack@3.1.2";

let connectionPromise = null;

const ROLE_NAME = "identity";
const ZOME_NAME = "identity_coordinator";

const ALLOWED_FUNCTIONS = new Set([
  "create_profile",
  "get_my_profile",
  "get_profile",
  "update_profile",
  "create_review",
  "get_reviews",
  "get_reputation_summary"
]);

// Geliştirme proxy'si (node dev/dev-server.mjs) varsa onu kullanır.
async function tryDevProxy() {
  try {
    const response = await fetch("/hc/info", { cache: "no-store" });
    if (!response.ok) return null;

    const type = response.headers.get("content-type") || "";
    if (!type.includes("application/msgpack")) return null;

    const info = decode(new Uint8Array(await response.arrayBuffer()));
    if (!(info?.agent_pub_key instanceof Uint8Array)) return null;

    return { mode: "dev-proxy", agentPubKey: info.agent_pub_key };
  } catch {
    return null;
  }
}

// Holochain Launcher / Moss / Kangaroo gibi bir kabuğun içinde doğrudan bağlantı.
async function connectDirect() {
  const client = await AppWebsocket.connect();
  const appInfo = await client.appInfo();

  const cells = appInfo?.cell_info?.[ROLE_NAME];
  if (!cells?.length) {
    throw new Error("Identity hücresi bulunamadı.");
  }

  const cell = cells.find(item => item.value?.cell_id);
  if (!cell) {
    throw new Error("Identity hücresi henüz hazır değil.");
  }

  if (!appInfo.agent_pub_key) {
    throw new Error("Agent public key alınamadı.");
  }

  return {
    mode: "direct",
    client,
    appInfo,
    cellId: cell.value.cell_id,
    agentPubKey: appInfo.agent_pub_key
  };
}

export async function connectHolochain() {
  if (!connectionPromise) {
    connectionPromise = (async () => {
      const proxy = await tryDevProxy();
      if (proxy) return proxy;

      try {
        return await connectDirect();
      } catch (error) {
        throw new Error(
          "Holochain bağlantısı kurulamadı. Bu sayfa bir Holochain kabuğu " +
            "(Launcher/Moss) içinde ya da geliştirme proxy'si " +
            "(node dev/dev-server.mjs) ile açılmalı. Ayrıntı: " +
            (error?.message ?? error)
        );
      }
    })().catch(error => {
      connectionPromise = null;
      throw error;
    });
  }

  return connectionPromise;
}

export async function callIdentity(fnName, payload = null) {
  if (!ALLOWED_FUNCTIONS.has(fnName)) {
    throw new Error("İzin verilmeyen zome fonksiyonu.");
  }

  const connection = await connectHolochain();

  if (connection.mode === "dev-proxy") {
    const response = await fetch("/hc/call/" + fnName, {
      method: "POST",
      headers: { "Content-Type": "application/msgpack" },
      body: encode(payload)
    });

    const body = decode(new Uint8Array(await response.arrayBuffer()));

    if (!response.ok) {
      throw new Error(body?.error || "Zome çağrısı başarısız oldu.");
    }

    return body.result;
  }

  return connection.client.callZome({
    cell_id: connection.cellId,
    zome_name: ZOME_NAME,
    fn_name: fnName,
    provenance: connection.agentPubKey,
    payload
  });
}

export async function disconnectHolochain() {
  if (!connectionPromise) return;

  const connection = await connectionPromise;

  if (
    connection.mode === "direct" &&
    typeof connection.client.client?.close === "function"
  ) {
    await connection.client.client.close();
  }

  connectionPromise = null;
}

// END OF FILE
