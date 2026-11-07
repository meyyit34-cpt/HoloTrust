import { AppWebsocket } from "https://esm.sh/@holochain/client@0.21.1";
import { decode, encode } from "https://esm.sh/@msgpack/msgpack@3.1.2";

let connectionPromise = null;

const ROLE_NAME = "identity";

const ALLOWED_FUNCTIONS = {
  identity_coordinator: new Set([
    "create_profile",
    "get_my_profile",
    "get_profile",
    "update_profile",
    "create_review",
    "get_reviews",
    "get_reputation_summary"
  ]),
  reputation_coordinator: new Set([
    "create_contextual_review",
    "get_contextual_reviews",
    "get_contextual_reputation_summary"
  ])
};

async function tryDevProxy() {
  try {
    const response = await fetch("/hc/info", { cache: "no-store" });
    if (!response.ok) return null;

    const type = response.headers.get("content-type") || "";
    if (!type.includes("application/msgpack")) return null;

    const info = decode(new Uint8Array(await response.arrayBuffer()));
    if (!(info?.agent_pub_key instanceof Uint8Array)) return null;

    return {
      mode: "dev-proxy",
      agentPubKey: info.agent_pub_key
    };
  } catch {
    return null;
  }
}

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
          "Holochain bağlantısı kurulamadı. Sayfa Launcher/Moss içinde " +
          "veya geliştirme proxy'si ile açılmalı. Ayrıntı: " +
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

export async function callZome(zomeName, fnName, payload = null) {
  const allowed = ALLOWED_FUNCTIONS[zomeName];

  if (!allowed || !allowed.has(fnName)) {
    throw new Error("İzin verilmeyen zome fonksiyonu.");
  }

  const connection = await connectHolochain();

  if (connection.mode === "dev-proxy") {
    const response = await fetch(
      `/hc/call/${encodeURIComponent(zomeName)}/${encodeURIComponent(fnName)}`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/msgpack"
        },
        body: encode(payload)
      }
    );

    const body = decode(
      new Uint8Array(await response.arrayBuffer())
    );

    if (!response.ok) {
      throw new Error(body?.error || "Zome çağrısı başarısız oldu.");
    }

    return body.result;
  }

  return connection.client.callZome({
    cell_id: connection.cellId,
    zome_name: zomeName,
    fn_name: fnName,
    provenance: connection.agentPubKey,
    payload
  });
}

export async function callIdentity(fnName, payload = null) {
  return callZome("identity_coordinator", fnName, payload);
}

export async function callReputation(fnName, payload = null) {
  return callZome("reputation_coordinator", fnName, payload);
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
